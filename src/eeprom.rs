//! EEPROM emulation
//!
//! EEPROM emulation uses a small (~1KiB) region of flash to persist data.

use core::{
    ffi::c_void,
    sync::atomic::{AtomicBool, Ordering},
};

#[cfg_attr(target_arch = "arm", link(name = "t4eeprom"))]
extern "C" {
    pub fn eeprom_initialize();
    pub fn eeprom_read_byte(addr_ptr: *const u8) -> u8;
    pub fn eeprom_write_byte(addr_ptr: *const u8, data: u8);
    pub fn eeprom_read_block(buf: *mut c_void, addr_ptr: *const c_void, len: u32);
    pub fn eeprom_write_block(buf: *const c_void, addr_ptr: *mut c_void, len: u32);
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

fn bounds_check_slice<T>(addr: usize, slice: &[T]) -> Result<()> {
    bounds_check_scalar(addr)?;
    bounds_check_scalar((addr + slice.len()).saturating_sub(1))?;
    Ok(())
}

/// Provides read/write access to EEPROM
///
/// There's only one of these available in a given program.
/// It implements some of the basic `embedded_storage` traits;
/// consider using these for generality.
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

    /// Read the exact number of bytes required to fill `buffer`, starting from `index`.
    ///
    /// If the operation would exceed the EEPROM range, this method returns an error
    /// and does nothing. This includes the case when `buffer` is empty. Otherwise,
    /// if `buffer` is non-empty, it will be filled with the contents of storage.
    pub fn read_bytes_exact(&self, index: usize, buffer: &mut [u8]) -> Result<()> {
        bounds_check_slice(index, buffer)?;
        if !buffer.is_empty() {
            unsafe {
                eeprom_read_block(
                    buffer.as_mut_ptr() as *mut _,
                    index as _,
                    buffer.len() as u32,
                )
            }
        }
        Ok(())
    }

    /// Write all of `buffer` to storage, starting at `index`.
    ///
    /// If the operation would exceed the EEPROM range, this method returns an error
    /// and does nothing. This includes the case when `buffer` is empty. Otherwise,
    /// if `buffer` is non-empty, its contents are written to storage.
    pub fn write_bytes_exact(&mut self, index: usize, buffer: &[u8]) -> Result<()> {
        bounds_check_slice(index, buffer)?;
        if !buffer.is_empty() {
            unsafe {
                eeprom_write_block(buffer.as_ptr() as *const _, index as _, buffer.len() as u32)
            }
        }
        Ok(())
    }
}

impl embedded_storage::ReadStorage for Eeprom {
    type Error = EepromError;
    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<()> {
        self.read_bytes_exact(offset as usize, bytes)
    }
    fn capacity(&self) -> usize {
        EEPROM_CAPACITY
    }
}

impl embedded_storage::Storage for Eeprom {
    fn write(&mut self, offset: u32, bytes: &[u8]) -> core::result::Result<(), Self::Error> {
        self.write_bytes_exact(offset as usize, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::{bounds_check_scalar, bounds_check_slice};

    #[test]
    fn scalar_ok() {
        assert!(bounds_check_scalar(0).is_ok());
        assert!(bounds_check_scalar(1079).is_ok());
        assert!(bounds_check_scalar(1080).is_err());
        assert!(bounds_check_scalar(9999).is_err());
    }

    #[test]
    fn slice_ok() {
        assert!(bounds_check_slice(0, &[1]).is_ok());
        assert!(bounds_check_slice(1079, &[1]).is_ok());
        assert!(bounds_check_slice(1080, &[1]).is_err());

        let buffer = [0u8; 13];
        assert!(bounds_check_slice(1070, &buffer).is_err());
        assert!(bounds_check_slice(1068, &buffer).is_err());
        assert!(bounds_check_slice(1067, &buffer).is_ok());
        assert!(bounds_check_slice(314, &buffer).is_ok());

        static LARGE_BUFFER: [u8; 1080] = [0; 1080];
        assert!(bounds_check_slice(0, &LARGE_BUFFER).is_ok());
        assert!(bounds_check_slice(1, &LARGE_BUFFER).is_err());

        static ALMOST_LARGE_BUFFER: [u8; 1079] = [0; 1079];
        assert!(bounds_check_slice(0, &ALMOST_LARGE_BUFFER).is_ok());
        assert!(bounds_check_slice(1, &ALMOST_LARGE_BUFFER).is_ok());
        assert!(bounds_check_slice(2, &ALMOST_LARGE_BUFFER).is_err());

        // Though there's nothing to read or write, we'll still
        // signal an error.
        const EMPTY: &[()] = &[];
        assert!(bounds_check_slice(1080, EMPTY).is_err());
    }
}
