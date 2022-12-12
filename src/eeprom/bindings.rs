//! The Teensy EEPROM modules, compiled for Rust.
//!
//! The crate is comprised of the C sources from the
//! [Teensy Core Libraries for Arduino](https://github.com/PaulStoffregen/cores).
//! We compile the EEPROM sources into the crate, then expose
//! the initialization and I/O routines. The crate is
//! intended for use in the `teensy4-bsp` Teensy 4 BSP.
//!

#[cfg_attr(target_arch = "arm", link(name = "t4eeprom"))]
extern "C" {
    pub fn eeprom_initialize();
    pub fn eeprom_read_byte(addr_ptr: *const u8) -> u8;
    pub fn eeprom_write_byte(addr_ptr: *const u8, data: u8);
}

pub unsafe fn eeprom_read<M: AsMut<[u8]>>(addr: u8, mut buffer: M) {
    let buffer = buffer.as_mut();
    for p in 0..buffer.len() {
        buffer[p] = eeprom_read_byte((p as u8 + addr) as *const u8)
    }
}

pub unsafe fn eeprom_write<B: AsMut<[u8]>>(mut addr: B, mut buffer: B) {
    let addr = addr.as_mut();
    let buffer = buffer.as_mut();
    for p in 0..buffer.len() {
        eeprom_write_byte(&addr[p], buffer[p])
    }
}
