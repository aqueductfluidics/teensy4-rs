//! Demonstrates how to use a SPI master.
//! Similar to the I2C example, we try to
//! read the WHO_AM_I register from an MPU9250.
//!
//! Pinout:
//!
//! - Teensy 4 Pin 13 (SCK) to MPU's SCL (Note that we lose the LED here)
//! - Teensy 4 Pin 11 (MOSI) to MPU's SDA/SDI
//! - Teensy 4 Pin 12 (MISO) to MPU's AD0/SDO
//! - Teensy 4 Pin 10 (PSC0) to MPU's NCS
//!
//! By default, the example utilizes the SPI's internal chip select pin, rather
//! than relying on an arbitrary GPIO to control the chip select. However, you
//! may consider using an arbitrary GPIO for chip select. The example supports
//! that as well.
//!
//! Success criteria: the SPI clock rate is 1MHz. We can read both the MPU9250's
//! `WHO_AM_I` register (returns 0x71), and also the AK8963's `WHO_AM_I` register
//! (returns 0x48).

#![no_std]
#![no_main]

mod systick;
mod usb_io;

use core::time::Duration;

use teensy4_panic as _;

use cortex_m_rt::entry;
use embedded_hal::{blocking::spi::Transfer, digital::v2::OutputPin};
use teensy4_bsp as bsp;

#[entry]
fn main() -> ! {
    let mut peripherals = bsp::Peripherals::take().unwrap();
    let mut systick = systick::new(cortex_m::Peripherals::take().unwrap().SYST);
    
    usb_io::init().unwrap();
    let pins = bsp::pins::t40::from_pads(peripherals.iomuxc);

    let mut led = bsp::configure_led(pins.p13);
    led.set();

    peripherals.ccm.pll1.set_arm_clock(
        bsp::hal::ccm::PLL1::ARM_HZ,
        &mut peripherals.ccm.handle,
        &mut peripherals.dcdc,
    );

    systick.delay_ms(1000);
    log::info!("Initializing CAN clocks...");

    let (can1_builder, _) = peripherals.can.clock(
        &mut peripherals.ccm.handle,
        bsp::hal::ccm::can::ClockSelect::Pll2,
        bsp::hal::ccm::can::PrescalarSelect::DIVIDE_1,
    );

    log::info!("Building CAN1 peripheral...");
    let mut can1 = can1_builder.build();
    can1.set_baud_rate(1_000_000);



    loop {
      systick.delay_ms(100);
      led.toggle();
      can1.read_mailbox();
    }

}


