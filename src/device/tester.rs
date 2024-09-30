use core::sync::atomic::{AtomicUsize, Ordering};

use crate::device::{DeviceAccess, Width};
use crate::virt::VirtContext;

// ————————————————————————————— Virtual Test Device —————————————————————————————— //

pub const TEST_DEVICE_SIZE: usize = 0x8;

/// Devices and drivers are used to communicate with the external world and therefore produce
/// non-deterministic events. This makes testing a virtual device interface harder. This structure
/// creates a determinist driver.
#[derive(Debug)]
pub struct VirtTestDevice {
    magic_value: usize,
    remote_register: AtomicUsize,
}

impl DeviceAccess for VirtTestDevice {
    fn read_device(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
        self.validate_offset(offset)?;
        self.validate_width(r_width)?;

        if offset == 0 {
            Ok(self.magic_value)
        } else {
            Ok(self.remote_register.load(Ordering::Relaxed))
        }
    }

    fn write_device(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        _ctx: &mut VirtContext,
    ) -> Result<(), &'static str> {
        self.validate_offset(offset)?;
        self.validate_width(w_width)?;

        if offset == 4 {
            self.remote_register.store(value, Ordering::Relaxed)
        }

        Ok(())
    }
}

impl VirtTestDevice {
    pub const fn new() -> Self {
        Self {
            magic_value: 0xdeadbeef,
            remote_register: AtomicUsize::new(0),
        }
    }

    fn validate_offset(&self, offset: usize) -> Result<(), &'static str> {
        if offset == 0 || offset == 4 {
            Ok(())
        } else {
            log::warn!("Invalid TestDriver offset: 0x{:x}", offset);
            Err("Invalid TestDriver offset")
        }
    }

    fn validate_width(&self, width: Width) -> Result<(), &'static str> {
        match width {
            Width::Byte4 => Ok(()),
            _ => Err("Invalid TestDriver width"),
        }
    }
}
