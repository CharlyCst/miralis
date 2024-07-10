//! Base device class

use spin::Mutex;

use crate::driver::{clint, ClintDriver};

// ———————————————————————————— Virtual Devices ————————————————————————————— //

// Represents a virtual memory-mapped device
pub struct VirtDevice {
    pub start_addr: usize,
    pub size: usize,
    pub name: &'static str,
    pub device_interface: &'static Mutex<VirtClint>,
}

pub fn find_matching_device(address: usize, devices: &[VirtDevice]) -> Option<&VirtDevice> {
    devices
        .iter()
        .find(|device| address >= device.start_addr && address < device.start_addr + device.size)
        .and_then(|matching_device| {
            let device_interface_guard = matching_device.device_interface.lock();
            match device_interface_guard.into() {
                Some(_interface) => Some(matching_device),
                None => {
                    log::error!(
                        "Device {} has a missing device_interface!",
                        matching_device.name
                    );
                    None
                }
            }
        })
}

pub trait DeviceAccess: Sync + Send {
    fn read_device(&self, offset: usize) -> Result<usize, &'static str>;
    fn write_device(&mut self, offset: usize, value: usize) -> Result<(), &'static str>;
}

// ————————————————————————————— Virtual CLINT —————————————————————————————— //

pub const CLINT_SIZE: usize = 0x10000;

// Represents the CLINT (Core Local Interruptor) device
#[derive(Clone, Debug)]
pub struct VirtClint {
    clint_driver: ClintDriver,
}

impl DeviceAccess for VirtClint {
    fn read_device(&self, offset: usize) -> Result<usize, &'static str> {
        self.read_clint(offset)
    }

    fn write_device(&mut self, offset: usize, value: usize) -> Result<(), &'static str> {
        self.write_clint(offset, value)
    }
}

impl VirtClint {
    /// Creates a new virtual CLINT device backed by a physical CLINT at the provided address.
    ///
    /// SAFETY: this function assumes that the base address corresponds to the base address of a
    /// CLINT-compatible device. In addition this function assumes that a at most one [ClintDriver]
    /// is initialized with the same base address (this function also creates a [ClintDriver]) and
    /// that no other code is accessing the CLINT device.
    pub const unsafe fn new(clint_base: usize) -> Self {
        let clint_driver = ClintDriver::new(clint_base);

        Self { clint_driver }
    }

    fn validate_offset(&self, offset: usize) -> Result<(), &'static str> {
        if offset >= CLINT_SIZE {
            log::warn!("Invalid CLINT offset: 0x{:x}", offset);
            Err("Invalid CLINT offset")
        } else {
            Ok(())
        }
    }

    pub fn read_clint(&self, offset: usize) -> Result<usize, &'static str> {
        log::trace!("Read from CLINT at offset 0x{:x}", offset);
        self.validate_offset(offset)?;

        match offset {
            o if o >= clint::MSIP_OFFSET && o < clint::MTIMECMP_OFFSET => {
                let hart = (o - clint::MSIP_OFFSET) / clint::MSIP_WIDTH;
                self.clint_driver.read_msip(hart)
            }
            o if o >= clint::MTIMECMP_OFFSET && o < clint::MTIME_OFFSET => {
                let hart = (o - clint::MTIMECMP_OFFSET) / clint::MTIMECMP_WIDTH;
                self.clint_driver.read_mtimecmp(hart)
            }
            o if o == clint::MTIME_OFFSET => Ok(self.clint_driver.read_mtime()),
            _ => Err("Invalid CLINT offset"),
        }
    }

    pub fn write_clint(&mut self, offset: usize, value: usize) -> Result<(), &'static str> {
        log::trace!(
            "Write to CLINT at offset 0x{:x} with a value 0x{:x}",
            offset,
            value
        );
        self.validate_offset(offset)?;

        match offset {
            o if o >= clint::MSIP_OFFSET && o < clint::MTIMECMP_OFFSET => {
                let hart = (o - clint::MSIP_OFFSET) / clint::MSIP_WIDTH;
                self.clint_driver.write_msip(hart, value as u32)
            }
            o if o >= clint::MTIMECMP_OFFSET && o < clint::MTIME_OFFSET => {
                let hart = (o - clint::MTIMECMP_OFFSET) / clint::MTIMECMP_WIDTH;
                self.clint_driver.write_mtimecmp(hart, value)
            }
            o if o == clint::MTIME_OFFSET => {
                self.clint_driver.write_mtime(value);
                Ok(())
            }
            _ => Err("Invalid CLINT address"),
        }
    }
}
