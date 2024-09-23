use spin::Mutex;

use crate::arch::mie;
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
}

impl DeviceAccess for VirtClint {
    fn read_device(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
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
        Self { driver }
    }

    fn validate_offset(&self, offset: usize) -> Result<(), &'static str> {
        if offset >= CLINT_SIZE {
            log::warn!("Invalid CLINT offset: 0x{:x}", offset);
            Err("Invalid CLINT offset")
        } else {
            Ok(())
        }
    }

    fn clear_interrupts(&self, ctx: &mut VirtContext) {
        let driver = self.driver.lock();
        // Hart may write MSIP bit of other hart, but we have no way of clearing
        // vmip.MSIP bit of another hart, so no point in checking
        let msip = driver.read_msip(ctx.hart_id).unwrap();
        let mtimecmp = driver.read_mtimecmp(ctx.hart_id).unwrap();
        let mtime = driver.read_mtime();

        if msip == 0 {
            ctx.csr.mip &= !(mie::MSIE_FILTER);
            log::trace!("vMSIP cleared");
        }
        if mtime < mtimecmp {
            ctx.csr.mip &= !(mie::MTIE_FILTER);
            log::trace!("vMTIP cleared");
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
        let mut err = false;
        {
            let mut driver = self.driver.lock();

            match (offset, w_width) {
                (o, Width::Byte4) if (MSIP_OFFSET..MTIMECMP_OFFSET).contains(&o) => {
                    let hart = (o - MSIP_OFFSET) / MSIP_WIDTH.to_bytes();
                    let _ = driver.write_msip(hart, value as u32);
                }
                (o, Width::Byte8) if (MTIMECMP_OFFSET..MTIME_OFFSET).contains(&o) => {
                    let hart = (o - MTIMECMP_OFFSET) / MTIMECMP_WIDTH.to_bytes();
                    let _ = driver.write_mtimecmp(hart, value);
                }
                (o, Width::Byte8) if o == MTIME_OFFSET => {
                    driver.write_mtime(value);
                }
                _ => err = true,
            }
        }

        self.clear_interrupts(ctx);

        if !err {
            Ok(())
        } else {
            Err("Invalid CLINT address")
        }
    }
}
