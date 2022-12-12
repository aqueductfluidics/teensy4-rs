//! Teensy 4 EEPROM, taken from the original Teensy 4 C libraries
//!

mod bindings;

#[inline(always)]
pub fn start() {
    unsafe { bindings::eeprom_initialize() };
}

#[inline(always)]
pub fn read_eeprom<M: AsMut<[u8]>>(addr: u8, buffer: M) {
    unsafe { bindings::eeprom_read(addr, buffer) };
}


