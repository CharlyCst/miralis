//! Base device class

use spin::Mutex;

use crate::arch::Width;
use crate::driver::{clint, ClintDriver};

// ———————————————————————————— Virtual Devices ————————————————————————————— //

/// Represents a virtual memory-mapped device
pub struct VirtDevice {
    pub start_addr: usize,
    pub size: usize,
    pub name: &'static str,
    pub device_interface: &'static dyn DeviceAccess,
}

pub fn find_matching_device(address: usize, devices: &[VirtDevice]) -> Option<&VirtDevice> {
    devices
        .iter()
        .find(|device| address >= device.start_addr && address < device.start_addr + device.size)
}

pub trait DeviceAccess: Sync + Send {
    fn read_device(&self, offset: usize, r_width: Width) -> Result<usize, &'static str>;
    fn write_device(&self, offset: usize, w_width: Width, value: usize)
        -> Result<(), &'static str>;
}

// ————————————————————————————— Virtual CLINT —————————————————————————————— //

pub const CLINT_SIZE: usize = 0x10000;

/// Represents a virtual CLINT (Core Local Interruptor) device
#[derive(Clone, Debug)]
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
    ) -> Result<(), &'static str> {
        self.write_clint(offset, w_width, value)
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

    pub fn read_clint(&self, offset: usize, r_width: Width) -> Result<usize, &'static str> {
        log::trace!("Read from CLINT at offset 0x{:x}", offset);
        self.validate_offset(offset)?;
        let driver = self.driver.lock();

        match (offset, r_width) {
            (o, Width::Byte4) if o >= clint::MSIP_OFFSET && o < clint::MTIMECMP_OFFSET => {
                let hart = (o - clint::MSIP_OFFSET) / clint::MSIP_WIDTH.to_bytes();
                driver.read_msip(hart)
            }
            (o, Width::Byte8) if o >= clint::MTIMECMP_OFFSET && o < clint::MTIME_OFFSET => {
                let hart = (o - clint::MTIMECMP_OFFSET) / clint::MTIMECMP_WIDTH.to_bytes();
                driver.read_mtimecmp(hart)
            }
            (o, Width::Byte8) if o == clint::MTIME_OFFSET => Ok(driver.read_mtime()),
            _ => Err("Invalid CLINT offset"),
        }
    }

    pub fn write_clint(
        &self,
        offset: usize,
        w_width: Width,
        value: usize,
    ) -> Result<(), &'static str> {
        log::trace!(
            "Write to CLINT at offset 0x{:x} with a value 0x{:x}",
            offset,
            value
        );
        self.validate_offset(offset)?;
        let mut driver = self.driver.lock();

        match (offset, w_width) {
            (o, Width::Byte4) if o >= clint::MSIP_OFFSET && o < clint::MTIMECMP_OFFSET => {
                let hart = (o - clint::MSIP_OFFSET) / clint::MSIP_WIDTH.to_bytes();
                driver.write_msip(hart, value as u32)
            }
            (o, Width::Byte8) if o >= clint::MTIMECMP_OFFSET && o < clint::MTIME_OFFSET => {
                let hart = (o - clint::MTIMECMP_OFFSET) / clint::MTIMECMP_WIDTH.to_bytes();
                driver.write_mtimecmp(hart, value)
            }
            (o, Width::Byte8) if o == clint::MTIME_OFFSET => {
                driver.write_mtime(value);
                Ok(())
            }
            _ => Err("Invalid CLINT address"),
        }
    }
}
