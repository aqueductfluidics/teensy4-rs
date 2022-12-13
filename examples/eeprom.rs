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

    let mut eeprom = bsp::eeprom::Eeprom::new().unwrap();
   
    log::info!("Starting loop...");

    let mut counter = 0;
    let mut buffer: [u8; 100] = [0x00; 100];

    // for i in 0..100 {
    //     eeprom.write_byte(i, i as u8);
    // }

    loop {
        systick.delay_ms(5000);
        counter += 1;
        for i in 0..100 {
            buffer[i] = eeprom.read_byte(i);
        }
        log::info!("{:?}", &buffer);
    }
}
