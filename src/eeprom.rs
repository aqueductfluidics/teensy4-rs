//! EEPROM emulation
//!
//! EEPROM emulation uses a small (~1KiB) region of flash to persist data.

use core::sync::atomic::{AtomicBool, Ordering};

#[cfg_attr(target_arch = "arm", link(name = "t4eeprom"))]
extern "C" {
    pub fn eeprom_initialize();
    pub fn eeprom_read_byte(addr_ptr: *const u8) -> u8;
    pub fn eeprom_write_byte(addr_ptr: *const u8, data: u8);
    // TODO the rest...
}

static TAKEN: AtomicBool = AtomicBool::new(false);

/// Possible errors encountered when interacting with EEPROM.
#[non_exhaustive]
#[derive(Debug)]
pub enum EepromError {
    /// The operation extends beyond the EEPROM range.
    ///
    /// See [`EEPROM_CAPACITY`] for more information.
    OutOfRange,
}

/// The EEPROM capacity, in bytes.
///
/// All values for `index` supplied to [`Eeprom`] should be less
/// than this value. Otherwise, you'll observe [`EepromError::OutOfRange`].
pub const EEPROM_CAPACITY: usize = E2END + 1;
const E2END: usize = 0x437;

type Result<T> = core::result::Result<T, EepromError>;

/// This simulates the bounds check that was implemented in
/// the official `eeprom.c` module. Unit tests demonstrate
/// that it meets the behaviors documented in [`EEPROM_CAPACITY`].
const fn bounds_check_scalar(addr: usize) -> Result<()> {
    if addr > E2END {
        Err(EepromError::OutOfRange)
    } else {
        Ok(())
    }
}

/// Provides read/write access to EEPROM
///
/// There's only one of these available in a given program.
pub struct Eeprom(());

impl Eeprom {
    // The safety of this implementation depends on
    //
    // 1. Bounds checks happening before we read / write to
    //    emulated EEPROM.
    // 2. These functions are only accessed from one execution context.
    //
    // We ensure 2 by construction of `new()`. We ensure 1 by construction
    // of I/O methods.
    //
    // Refer to this notice when you see undocumented `unsafe` code in
    // this block.

    /// Create an `Eeprom` that controls I/O with the EEPROM emulation region
    ///
    /// Returns `None` if the `Eeprom` has already been created.
    pub fn new() -> Option<Self> {
        let taken = TAKEN.swap(true, Ordering::SeqCst);
        if taken {
            None
        } else {
            unsafe { eeprom_initialize() };
            Some(Eeprom(()))
        }
    }
    /// Read a byte from the EEPROM emulation region.
    pub fn read_byte(&self, index: usize) -> Result<u8> {
        bounds_check_scalar(index)?;
        Ok(unsafe { eeprom_read_byte(index as _) })
    }
    /// Write a byte into the EEPROM emulated region.
    pub fn write_byte(&mut self, index: usize, byte: u8) -> Result<()> {
        bounds_check_scalar(index)?;
        Ok(unsafe { eeprom_write_byte(index as _, byte) })
    }
}

#[cfg(test)]
mod tests {
    use super::bounds_check_scalar;

    #[test]
    fn scalar_ok() {
        assert!(bounds_check_scalar(0).is_ok());
        assert!(bounds_check_scalar(1079).is_ok());
        assert!(bounds_check_scalar(1080).is_err());
        assert!(bounds_check_scalar(0xDEADBEEF).is_err());
    }
}
