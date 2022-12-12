//! Demonstrates an I2C master. We try to read data from
//! a MPU9250 9-DOF IMU.
//!
//! Teensy pin 16 => SCL (I2C3)
//! Teensy pin 17 => SDA (I2C3)
//!
//! Success criteria:
//!
//! - The MPU correctly reports its `WHO_AM_I` address. The slave
//!   address is printed over USB logging.
//! - The clock is running at its selected bit rate; either 100KHz
//!   or 400KHz. Measure it with a logic analyzer.
//! - There's a repeated start in the `write_read` call; observable
//!   via a logic analyzer. Changing it to a `write`, followed by a
//!   `read`, should show that there is are two transactions.

#![no_std]
#![no_main]

mod systick;
mod usb_io;

use teensy4_panic as _;

use teensy4_bsp as bsp;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut peripherals = bsp::Peripherals::take().unwrap();
    let mut systick = systick::new(cortex_m::Peripherals::take().unwrap().SYST);
    let pins = bsp::pins::t40::from_pads(peripherals.iomuxc);
    usb_io::init().unwrap();
    systick.delay_ms(50);

    bsp::eeprom::start();
   
    log::info!("Starting loop...");

    let mut buffer: [u8; 100] = [0x00; 100];
    let mut counter = 0;

    loop {
        systick.delay_ms(1000);
        counter += 1;
        bsp::eeprom::read_eeprom(0x00, buffer);
        log::info!("Buffer: {:?}", &buffer);
    }
}
