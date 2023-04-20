//! Loopback over UART
//!
//! Connect Teensy pins 14 and 15 together. We transfer
//! from one pin, and receive on the other. Demonstrates
//! the usage of the TX and RX FIFOs.
//!
//! It's not the most advanced example. The RX FIFO could
//! overrun if we're not reading fast enough.
//!
//! See the `const` configurations for settings.
//!
//! Success criteria: the loopback is reported as successful
//! over USB logging. Changing the settings below demonstrate
//! the same ideal behavior. When decreasing the `TX_FIFO_SIZE`,
//! we see an increase of blocked reads. The transfer
//! content is `0xDEADBEEF`.

#![no_std]
#![no_main]

mod systick;
mod usb_io;

use teensy4_panic as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::serial::{Read, Write};
use teensy4_bsp as bsp;

const BAUD: u32 = 9_600;
/// Change the TX FIFO sizes to see how the FIFO affects the number
/// of `WouldBlock`s that we would see. Setting this to zero disables
/// the FIFO.
const TX_FIFO_SIZE: u8 = 4;
/// Change me to affect the partity bit generation
const PARITY: Option<bsp::hal::uart::Parity> = None;
/// Change me to invert all output data, and to treat all input data as inverted
const INVERTED: bool = false;
/// The data you want to send and receive
const DATA: [u8; 10] = [0x01, 0x05, 0x05, 0x00, 0x01, 0x05, 0x05, 0x00, 0x01, 0x05];

/// Writes `bytes` to the provided `uart`. The function will count the
/// number of blocks that we hit, and will log how many blocks we
/// required to transmit `bytes`.
fn write<W: Write<u8>>(uart: &mut W, bytes: &[u8]) -> Result<(), W::Error> {
    let mut blocks = 0;
    for byte in bytes {
        loop {
            match uart.write(*byte) {
                Ok(()) => break,
                Err(nb::Error::WouldBlock) => blocks += 1,
                Err(nb::Error::Other(err)) => return Err(err),
            }
        }
    }
    log::info!("{} blocks to transmit {:?}", blocks, bytes);
    Ok(())
}

/// Reads from `uart` into `bytes`. The function will count the
/// number of blocks that we hit, and it will log how many blocks
/// we required to receive `bytes`.
fn read<R: Read<u8>>(uart: &mut R, bytes: &mut [u8]) -> Result<(), R::Error> {
    let mut blocks = 0;
    let mut n = 0;
    let mut errors = 0;
    'read: loop {
        match uart.read() {
            Ok(b) => {
                bytes[n] = b;
                if n >= bytes.len().saturating_sub(1) {
                    break;
                } else {
                    n += 1
                }
                // log::info!("{:?}", &b);
            }
            Err(nb::Error::WouldBlock) => {
                blocks += 1;
                if blocks > 1_000_000 {
                    break 'read;
                }
            }
            Err(nb::Error::Other(err)) => {
                errors += 1;
                if errors > 5 {
                    break 'read;
                }
            }
        }
    }

    log::info!("{} blocks to receive {:?}", blocks, bytes);
    Ok(())
}

#[entry]
fn main() -> ! {
    let mut peripherals = bsp::Peripherals::take().unwrap();
    let mut systick = systick::new(cortex_m::Peripherals::take().unwrap().SYST);
    let pins = bsp::pins::t41::from_pads(peripherals.iomuxc);
    usb_io::init().unwrap();

    let mut rs485_enable = bsp::hal::gpio::GPIO::new(pins.p6).output();
    rs485_enable.set_fast(true);
    rs485_enable.set_low().unwrap();

    systick.delay_ms(1_000);
    let uarts = peripherals.uart.clock(
        &mut peripherals.ccm.handle,
        bsp::hal::ccm::uart::ClockSelect::OSC,
        bsp::hal::ccm::uart::PrescalarSelect::DIVIDE_1,
    );
    let mut uart = uarts.uart4.init(pins.p8, pins.p7, BAUD).unwrap();
    let fifo_size = uart.set_tx_fifo(core::num::NonZeroU8::new(TX_FIFO_SIZE));
    log::info!("Setting TX FIFO to {}", fifo_size);
    // If this is disabled, we won't receive the four bytes from the transfer!
    uart.set_rx_fifo(true);
    uart.set_receiver_interrupt(Some(0));
    uart.set_parity(PARITY);
    uart.set_rx_inversion(INVERTED);
    uart.set_tx_inversion(INVERTED);

    let mut led = bsp::configure_led(pins.p13);
    let (mut tx, mut rx) = uart.split();
    loop {
        systick.delay_ms(1_000);
        led.toggle();
        let mut buffer = DATA;
        rs485_enable.set_high().unwrap();
        systick.delay_ms(1);
        write(&mut tx, &buffer).unwrap();
        rs485_enable.set_low().unwrap();
        buffer = [0x00; 10];
        match read(&mut rx, &mut buffer) {
            Ok(_) => continue,
            Err(err) => log::warn!("Receiver error: {:?}", err.flags),
        }
    }
}
