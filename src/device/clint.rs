use core::sync::atomic::{AtomicBool, Ordering};

use spin::Mutex;

use crate::arch::mie;
use crate::config::PLATFORM_NB_HARTS;
use crate::debug;
use crate::device::{DeviceAccess, Width};
use crate::driver::clint::{
    MSIP_OFFSET, MSIP_WIDTH, MTIMECMP_OFFSET, MTIMECMP_WIDTH, MTIME_OFFSET,
};
use crate::driver::ClintDriver;
use crate::virt::VirtContext;

// ————————————————————————————— Virtual CLINT —————————————————————————————— //

pub const CLINT_SIZE: usize = 0x10000;

/// Represents a virtual CLINT (Core Local Interruptor) device
#[derive(Debug)]
pub struct VirtClint {
    /// A driver for the physical CLINT
    driver: &'static Mutex<ClintDriver>,
    /// Virtual Machine Software Interrupt (MSI) map
    vmsi: [AtomicBool; PLATFORM_NB_HARTS],
}

impl DeviceAccess for VirtClint {
    fn read_device(
        &self,
        offset: usize,
        r_width: Width,
        _ctx: &mut VirtContext,
    ) -> Result<usize, &'static str> {
        self.read_clint(offset, r_width)
    }

    fn write_device(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        ctx: &mut VirtContext,
    ) -> Result<(), &'static str> {
        self.write_clint(offset, w_width, value, ctx)
    }
}

impl VirtClint {
    /// Creates a new virtual CLINT device backed by a physical CLINT.
    pub const fn new(driver: &'static Mutex<ClintDriver>) -> Self {
        Self {
            driver,
            vmsi: [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS],
        }
    }

    fn validate_offset(&self, offset: usize) -> Result<(), &'static str> {
        if offset >= CLINT_SIZE {
            log::warn!("Invalid CLINT offset: 0x{:x}", offset);
            Err("Invalid CLINT offset")
        } else {
            Ok(())
        }
    }

    pub fn read_clint(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
        log::trace!("Read from CLINT at offset 0x{:x}", offset);
        self.validate_offset(offset)?;
        let driver = self.driver.lock();

        match (offset, r_width) {
            (o, Width::Byte4) if (MSIP_OFFSET..MTIMECMP_OFFSET).contains(&o) => {
                let hart = (o - MSIP_OFFSET) / MSIP_WIDTH.to_bytes();
                driver.read_msip(hart)
            }
            (o, Width::Byte8) if (MTIMECMP_OFFSET..MTIME_OFFSET).contains(&o) => {
                let hart = (o - MTIMECMP_OFFSET) / MTIMECMP_WIDTH.to_bytes();
                driver.read_mtimecmp(hart)
            }
            (o, Width::Byte8) if o == MTIME_OFFSET => Ok(driver.read_mtime()),
            _ => Err("Invalid CLINT offset"),
        }
    }

    pub fn write_clint(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
        ctx: &mut VirtContext,
    ) -> Result<(), &'static str> {
        log::trace!(
            "Write to CLINT at offset 0x{:x} with a value 0x{:x}",
            offset,
            value
        );
        self.validate_offset(offset)?;
        let mut driver = self.driver.lock();

        match (offset, w_width) {
            (o, Width::Byte4) if (MSIP_OFFSET..MTIMECMP_OFFSET).contains(&o) => {
                let hart = (o - MSIP_OFFSET) / MSIP_WIDTH.to_bytes();
                if hart >= PLATFORM_NB_HARTS {
                    return Err("Invalid hart when writting MSIP");
                }
                match value & 0b1 {
                    0 => {
                        // Clear pending MSI
                        self.vmsi[hart].store(false, Ordering::SeqCst);
                        if hart == ctx.hart_id {
                            // On the current hart clear mip.MSIE
                            ctx.csr.mip &= !mie::MSIE_FILTER;
                            Ok(())
                        } else {
                            // On remote hart send a physical MSI
                            driver.write_msip(hart, 1)
                        }
                    }
                    1 => {
                        // Set pending MSI
                        self.vmsi[hart].store(true, Ordering::SeqCst);
                        if hart == ctx.hart_id {
                            // On the current hart set mip.MSIE
                            ctx.csr.mip |= mie::MSIE_FILTER;
                            Ok(())
                        } else {
                            // On remote hart send a physical MSI
                            driver.write_msip(hart, 1)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (o, Width::Byte8) if (MTIMECMP_OFFSET..MTIME_OFFSET).contains(&o) => {
                let mtime = driver.read_mtime();
                let hart = (o - MTIMECMP_OFFSET) / MTIMECMP_WIDTH.to_bytes();
                if hart >= PLATFORM_NB_HARTS {
                    return Err("Invalid hart when writting MSIP");
                }
                if hart != ctx.hart_id {
                    todo!("Setting mtime for another hart is not yet supported");
                }

                // Update the virtual `mip` according to the relative ordering of mtime and
                // mtimecmp.
                if mtime >= value {
                    ctx.csr.mip |= mie::MTIE_FILTER;
                } else {
                    // Register a timer to trigger the virtual interrupt once appropriate
                    driver.write_mtimecmp(hart, value)?;
                    ctx.csr.mip &= !mie::MTIE_FILTER;
                }

                Ok(())
            }
            (o, Width::Byte8) if o == MTIME_OFFSET => {
                // TODO: when updating mtime we should check on which core the timer should fire.
                // We don't do it for now so we might loose interrupts.
                debug::warn_once!(
                    "Write to mtime not yet fully supported (might cause interrupt loss)"
                );
                driver.write_mtime(value);
                Ok(())
            }
            _ => Err("Invalid CLINT address"),
        }
    }

    /// Return true if a vMSI is pending for the given hart
    pub fn get_vmsi(&self, hart: usize) -> bool {
        assert!(
            hart < PLATFORM_NB_HARTS,
            "Invalid hart ID when clearing vMSI"
        );
        self.vmsi[hart].load(Ordering::SeqCst)
    }
}
