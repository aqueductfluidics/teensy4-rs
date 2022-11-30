//! An adaptation of the `rtic_blink.rs` example that demonstrates logging via Teensy 4 UART.
//!
//! This example requires:
//!
//! - The `rtic` feature to be enabled.
//! - a serial to USB converter (tested with CP2102). The converter should be connected to pins 14
//! and 15. Pin 14 is teensy's TX and pin 15 is teensy's RX pin.
//!
//! Success criteria:
//! - The on-board LED should blink once per second.
//! - On each blink, we receive a message from the teensy via the serial console (e.g. `screen`).
//! - When writing serial data from the console, the teensy should log when each call to the
//! interrupt hardware task occurs and prints the characters received as a utf8 string on each
//! blink.

#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;
mod isotp;
mod systick;
mod usb_io;

// Type aliases for the Queue we want to use.
type Ty = u8;
const CAP: usize = 256;
type Queue = heapless::spsc::Queue<Ty, { CAP }>;
type Producer = heapless::spsc::Producer<'static, Ty, { CAP }>;
type Consumer = heapless::spsc::Consumer<'static, Ty, { CAP }>;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [LPUART8])]
mod app {
    use crate::{isotp, systick, usb_io, Consumer, Producer, Queue};
    use dwt_systick_monotonic::{fugit::ExtU32, DwtSystick};
    use embedded_hal::serial::Read;
    use embedded_hal::{can::Frame, can::StandardId, digital::v2::OutputPin};
    use imxrt1060_hal::iomuxc::consts::{Unsigned, U1, U2};
    use teensy4_bsp as bsp;

    const BAUD: u32 = 115_200;
    const TX_FIFO_SIZE: u8 = 4;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<{ bsp::hal::ccm::PLL1::ARM_HZ }>;

    #[local]
    struct Local {
        led: bsp::Led,
        q_tx: Producer,
        q_rx: Consumer,
        blink_count: u32,
    }

    #[shared]
    struct Shared {
        isotp: isotp::IsoTP<U1>,
    }

    #[init(local = [
        queue: Queue = heapless::spsc::Queue::new(),
    ])]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        usb_io::init().unwrap();

        let mono = DwtSystick::new(&mut dcb, dwt, systick, bsp::hal::ccm::PLL1::ARM_HZ);

        cx.device.ccm.pll1.set_arm_clock(
            bsp::hal::ccm::PLL1::ARM_HZ,
            &mut cx.device.ccm.handle,
            &mut cx.device.dcdc,
        );

        let pins = bsp::pins::t40::from_pads(cx.device.iomuxc);

        let (can1_builder, _) = cx.device.can.clock(
            &mut cx.device.ccm.handle,
            bsp::hal::ccm::can::ClockSelect::Pll2,
            bsp::hal::ccm::can::PrescalarSelect::DIVIDE_1,
        );

        let mut can1 = can1_builder.build();

        let delay_ms = |ms: u32| -> () {
            let wait_until = monotonics::now() + ms.millis();
            while monotonics::now() < wait_until { /* spin */ }
        };

        let delay_us = |us: u32| -> () {
            let wait_until = monotonics::now() + us.micros();
            while monotonics::now() < wait_until { /* spin */ }
        };

        let mut isotp_builder = isotp::IsoTPBuilder::new(can1);
        let isotp = isotp_builder.build();

        // The queue used for buffering bytes.
        let (q_tx, q_rx) = cx.local.queue.split();

        // LED setup.
        let mut led = bsp::configure_led(pins.p13);
        led.set_high().unwrap();

        // Schedule the first blink.
        blink::spawn_after(1_u32.secs()).unwrap();

        can1_init::spawn_after(1_u32.secs()).unwrap();

        (
            Shared { isotp },
            Local {
                led,
                q_rx,
                q_tx,
                blink_count: 0,
            },
            init::Monotonics(mono),
        )
    }

    #[task(local = [led, q_rx, blink_count])]
    fn blink(cx: blink::Context) {
        if cx.local.q_rx.ready() {
            let mut buffer = [0u8; 256];
            for elem in buffer.iter_mut() {
                *elem = match cx.local.q_rx.dequeue() {
                    None => break,
                    Some(b) => b,
                };
            }
            let s = core::str::from_utf8(&buffer).unwrap();
            log::info!("read: {}", s);
        }

        // Toggle the LED.
        cx.local.led.toggle();

        // Schedule the following blink.
        blink::spawn_after(100_u32.millis()).unwrap();
    }

    #[task(shared = [isotp])]
    fn can1_init(mut cx: can1_init::Context) {
        cx.shared.isotp.lock(|isotp| {
            isotp.can.set_baud_rate(1_000_000);
            isotp.can.set_max_mailbox(16);
            isotp.can.enable_fifo(true);
            isotp.can.enable_fifo_interrupt(true);
            isotp.can.print_registers();
        });
        can1::spawn_after(100_u32.millis()).unwrap();
    }

    #[task(shared = [isotp], local = [q_tx])]
    fn can1(mut cx: can1::Context) {
        let q_tx = cx.local.q_tx;
        const data: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        let id = StandardId::new(0).unwrap();
        let frame = Frame::new(id, &data).unwrap();
        cx.shared.isotp.lock(|isotp| {
            for _ in 0..5 {
                let wait_until = monotonics::now() + (10 as u32).millis();
                while monotonics::now() < wait_until { /* spin */ }
                isotp.write(&frame);
                log::info!("{:?}", monotonics::now());
            }
        });
        can1::spawn_after(100_u32.millis()).unwrap();
    }

    #[task(binds = CAN1, shared = [isotp],)]
    fn can1_int(mut cx: can1_int::Context) {
        cx.shared.isotp.lock(|isotp| {
            isotp.can.handle_interrupt();
        });
    }
}
