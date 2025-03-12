//! Passthrough device
//!
//! This device intercepts the reads and writes for a specific range and executes the read and writes in M-mode.

use crate::device::{DeviceAccess, Width};
use crate::virt::VirtContext;
#[derive(Debug, Default)]
pub struct PassThroughDevice {
    // The driver is stateless
}

impl PassThroughDevice {
    pub(crate) const fn new() -> Self {
        PassThroughDevice {}
    }
}

pub const PASSTHROUGH_BASE_ADDRESS: usize = 0x0;

// All values before 0x8000_0000 should be accessed in M-mode
pub const PASSTHROUGH_SIZE: usize = 0x8000_0000;
impl DeviceAccess for PassThroughDevice {
    fn read_device(
        &self,
        offset: usize,
        r_width: Width,
        _ctx: &mut VirtContext,
    ) -> Result<usize, &'static str> {
        let ptr = PASSTHROUGH_BASE_ADDRESS + offset;

        unsafe {
            match r_width {
                Width::Byte => (ptr as *const u8).read_volatile() as usize,
                Width::Byte2 => (ptr as *const u16).read_volatile() as usize,
                Width::Byte4 => (ptr as *const u32).read_volatile() as usize,
                Width::Byte8 => (ptr as *const u64).read_volatile() as usize,
            };
        }

        Ok(0)
    }

    fn write_device(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        _ctx: &mut VirtContext,
    ) -> Result<(), &'static str> {
        let ptr = PASSTHROUGH_BASE_ADDRESS + offset;

        unsafe {
            match w_width {
                Width::Byte => (ptr as *mut u8).write_volatile(value as u8),
                Width::Byte2 => (ptr as *mut u16).write_volatile(value as u16),
                Width::Byte4 => (ptr as *mut u32).write_volatile(value as u32),
                Width::Byte8 => (ptr as *mut u64).write_volatile(value as u64),
            };
        }

        Ok(())
    }
}
