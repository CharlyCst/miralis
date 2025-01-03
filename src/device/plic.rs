//! Virtual PLIC device
//!
//! This modules implements a virtual PLIC device, that is the front-end of the virtual device
//! exposed to the virtual firmware.
//!
//! For the specification of the PLIC see here:
//! https://github.com/riscv/riscv-plic-spec/releases/tag/1.0.0

use spin::Mutex;

use crate::device::{DeviceAccess, Width};
use crate::driver::plic::PlicDriver;
use crate::virt::VirtContext;
use crate::{debug, logger};

// —————————————————————————————— Virtual PLIC —————————————————————————————— //

/// Size of the address space covered by the PLIC.
pub const PLIC_SIZE: usize = 0x4000000;

/// Represents a virtual PLIC (Platform-Level Interrupt Controller) device
#[derive(Debug)]
pub struct VirtPlic {
    /// A driver for the physical CLINT
    driver: &'static Mutex<PlicDriver>,
}

impl DeviceAccess for VirtPlic {
    fn read_device(
        &self,
        offset: usize,
        r_width: Width,
        _ctx: &mut VirtContext,
    ) -> Result<usize, &'static str> {
        logger::trace!("read PLIC at offset 0x{:x}", offset);
        let plic = self.driver.lock();

        // TODO: for now we don't virtualize the PLIC, but simply implement a pass-through
        // We should implement a proper virtualization.
        unsafe {
            let ptr = plic.add_base_offset(offset);
            let val = match r_width {
                Width::Byte => (ptr as *const u8).read_volatile() as usize,
                Width::Byte2 => (ptr as *const u16).read_volatile() as usize,
                Width::Byte4 => (ptr as *const u32).read_volatile() as usize,
                Width::Byte8 => (ptr as *const u64).read_volatile() as usize,
            };

            Ok(val)
        }
    }

    fn write_device(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        _ctx: &mut VirtContext,
    ) -> Result<(), &'static str> {
        // Validate the write width and alignment
        if offset % 4 != 0 && w_width != Width::Byte4 {
            debug::warn_once!(
                "Unexpected write width/alignment: offset 0x{:x}, width: {}",
                offset,
                w_width as u8
            );

            // We return early in this case, simply discarding the write
            return Ok(());
        }

        // Log some information, for debugging purpose
        match offset {
            0x000000..0x001000 => {
                logger::trace!("Setting interrupt {} to priority 0x{:x}", offset / 4, value)
            }
            0x002000..0x200000 => {
                let context = (offset - 0x002000) / 0x80;
                let source_group = ((offset - 0x002000) % 0x80) * 8;
                logger::trace!(
                    "Setting enable bits for sources {}-{} to 0x{:x} on context {}",
                    source_group,
                    source_group + 31,
                    value,
                    context
                );
            }
            0x200000..0x400000 => {
                let context = (offset - 0x200000) % 0x1000;
                match offset % 0x1000 {
                    0 => logger::trace!(
                        "Set priority threshold for context {} to 0x{:x}",
                        context,
                        value
                    ),
                    4 => logger::trace!("Complete interrupt {} on context {}", value, context),
                    _ => logger::trace!("Write to reserved area at offset 0x{:x}", offset),
                }
            }
            _ => logger::debug!("Writting to unknon PLIC region at offset 0x{:x}", offset),
        }
        let plic = self.driver.lock();

        // TODO: for now we don't virtualize the PLIC, but simply implement a pass-through
        // We should implement a proper virtualization.
        unsafe {
            let ptr = plic.add_base_offset(offset);
            match w_width {
                Width::Byte => (ptr as *mut u8).write_volatile(value as u8),
                Width::Byte2 => (ptr as *mut u16).write_volatile(value as u16),
                Width::Byte4 => (ptr as *mut u32).write_volatile(value as u32),
                Width::Byte8 => (ptr as *mut u64).write_volatile(value as u64),
            }

            Ok(())
        }
    }
}

impl VirtPlic {
    /// Creates a new virtual PLIC device backed by a physical PLIC.
    pub const fn new(driver: &'static Mutex<PlicDriver>) -> Self {
        Self { driver }
    }
}
