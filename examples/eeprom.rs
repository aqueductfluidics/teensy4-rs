//! EEPROM emulation
//!
//! Demonstrates how to read and write to an EEPROM address.
//! Success: 5 seconds after power-up, the example checks a
//! location in flash for a sentinel value. It logs if the expected
//! value is found. It reads and writes another location, incrementing
//! the value. This should persist across executions.

#![no_std]
#![no_main]

mod systick;
mod usb_io;

use teensy4_panic as _;

use cortex_m_rt as rt;
use teensy4_bsp as bsp;

const PERIOD_MS: u32 = 1_000;
const EEPROM_RW_ADDRESS: usize = 42;
const EEPROM_PERSISTENCE_ADDRESS: usize = 137;
const EEPROM_SENTINEL: u8 = 42;

#[rt::entry]
fn main() -> ! {
    let p = bsp::Peripherals::take().unwrap();
    let mut systick = systick::new(cortex_m::Peripherals::take().unwrap().SYST);
    usb_io::init().unwrap();
    systick.delay_ms(5_000);
    let pins = bsp::pins::t40::from_pads(p.iomuxc);
    let mut led = bsp::configure_led(pins.p13);

    let mut eeprom = bsp::Eeprom::new().unwrap();
    if EEPROM_SENTINEL == eeprom.read_byte(EEPROM_PERSISTENCE_ADDRESS).unwrap() {
        // Run this example at least once. Then, power cycle your Teensy4. You
        // should see this branch of code is hit (manifests as the LED turns on
        // immediately, instead of after one second.)
        led.set();
        log::info!("Observed EEPROM_SENTINEL");
    } else {
        log::warn!("No EEPROM_SENTINEL");
    }
    eeprom
        .write_byte(EEPROM_PERSISTENCE_ADDRESS, EEPROM_SENTINEL)
        .unwrap();

    loop {
        systick.delay_ms(PERIOD_MS);
        led.toggle();

        let mut rw_val = eeprom.read_byte(EEPROM_RW_ADDRESS).unwrap();
        log::info!("Incrementing {rw_val}...");
        rw_val = rw_val.wrapping_add(1);
        eeprom.write_byte(EEPROM_RW_ADDRESS, rw_val).unwrap();

        if rw_val % 7 == 0 {
            log::warn!(
                "Intentionally reading out of bounds... {:?}",
                eeprom.read_byte(0xDEADBEEF)
            );
            log::warn!(
                "Intentionally writing out of bounds... {:?}",
                eeprom.read_byte(0xDEADBEEF)
            );
        }
    }
}
