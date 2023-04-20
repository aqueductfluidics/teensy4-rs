#![no_std]
#![no_main]

mod systick;
mod usb_io;

use teensy4_panic as _;

use cortex_m_rt::entry;
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

    let mut wdog1 = peripherals.wdog1.clock(&mut peripherals.ccm.handle);

    loop {
        systick.delay_ms(1000);
        led.toggle();
        wdog1.feed();
    }
}
