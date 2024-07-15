//! Base device class

use spin::Mutex;

use crate::config;
use crate::driver::ClintDriver;
use crate::platform::{virt, Platform};

pub const CLINT_SIZE: usize = 0x10000;

// Represents a memory-mapped device
pub struct Device {
    pub start_addr: usize,
    pub size: usize,
    pub name: &'static str,
    pub device_interface: Option<&'static Mutex<VirtClint>>,
}

pub fn find_matching_device(address: usize, devices: &[Device]) -> Option<&Device> {
    devices
        .iter()
        .find(|device| address >= device.start_addr && address < device.start_addr + device.size)
        .and_then(|matching_device| {
            let device_interface_guard = matching_device.device_interface?.lock();
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
    pub fn new() -> Self {
        let clint_driver = ClintDriver::new(0x0, 0x4000, 0xBFF8, config::PLATFORM_NB_HARTS, 32, 64);

        Self { clint_driver }
    }

    fn validate_address(&self, offset: usize) -> Result<(), &'static str> {
        let clint_base = virt::VirtPlatform::get_clint_base();
        let offset_to_clint = offset - clint_base;

        if offset_to_clint < self.clint_driver.base_address_msip
            || offset_to_clint > self.clint_driver.base_address_mtime
        {
            Err("Invalid CLINT address")
        } else {
            Ok(())
        }
    }

    pub fn read_clint(&self, offset: usize) -> Result<usize, &'static str> {
        self.validate_address(offset)?;

        let offset_to_clint = offset - virt::VirtPlatform::get_clint_base();
        log::trace!("Read from CLINT address {:x}", offset_to_clint);

        match offset_to_clint {
            o if o >= self.clint_driver.base_address_msip
                && o < self.clint_driver.base_address_mtimecmp =>
            {
                let hart = (o - self.clint_driver.base_address_msip) / self.clint_driver.msip_width;
                self.clint_driver.read_msip(hart)
            }
            o if o >= self.clint_driver.base_address_mtimecmp
                && o < self.clint_driver.base_address_mtime =>
            {
                let hart = (o - self.clint_driver.base_address_mtimecmp)
                    / self.clint_driver.mtimecmp_width;
                self.clint_driver.read_mtimecmp(hart)
            }
            o if o == self.clint_driver.base_address_mtime => Ok(self.clint_driver.read_mtime()),
            _ => Err("Invalid CLINT address"),
        }
    }

    pub fn write_clint(&mut self, offset: usize, value: usize) -> Result<(), &'static str> {
        self.validate_address(offset)?;
        let offset_to_clint = offset - virt::VirtPlatform::get_clint_base();
        log::trace!(
            "Write to CLINT address {:x} with a value {:?}",
            offset_to_clint,
            value
        );

        match offset_to_clint {
            o if o >= self.clint_driver.base_address_msip
                && o < self.clint_driver.base_address_mtimecmp =>
            {
                let hart = (o - self.clint_driver.base_address_msip) / self.clint_driver.msip_width;
                self.clint_driver.write_msip(hart, value as u32)
            }
            o if o >= self.clint_driver.base_address_mtimecmp
                && o < self.clint_driver.base_address_mtime =>
            {
                let hart = (o - self.clint_driver.base_address_mtimecmp)
                    / self.clint_driver.mtimecmp_width;
                self.clint_driver.write_mtimecmp(hart, value)
            }
            o if o == self.clint_driver.base_address_mtime => {
                self.clint_driver.write_mtime(value);
                Ok(())
            }
            _ => Err("Invalid CLINT address"),
        }
    }
}
