//! Base device class

use crate::driver::ClintDriver;

// Represents a memory-mapped device
pub struct Device {
    pub start_addr: usize,
    pub size: usize,
    pub name: &'static str,
}

pub fn find_matching_device(address: usize, devices: &[Device]) -> Option<&Device> {
    // Iterate through the list (slice?) of devices
    for device in devices {
        // Check if the address falls within the device's protected range
        if address >= device.start_addr && address < device.start_addr + device.size {
            log::trace!("Found matching device: {}", device.name);
            return Some(device);
        }
    }
    None // No matching device found
}

// Represents the CLINT (Core Local Interruptor) device
pub struct VirtClint;

impl VirtClint {

    pub fn read_clint(offset: usize) -> Result<usize, &'static str> {
        log::trace!("Read from CLINT register: {:x}", offset);
        let clint_driver = ClintDriver::new();
    
        match offset {
            0x4000..=0x4028 => {
                let mtimecmp_value = clint_driver.read_mtime(offset);
                log::trace!("MTIMECMP value: {}", mtimecmp_value);
                Ok(mtimecmp_value)
            }
            0xBFF8 => {
                let mtime_value = clint_driver.read_mtime(offset);
                log::trace!("MTIME value: {}", mtime_value);
                Ok(mtime_value)
            }
            _ => {
                log::error!("Invalid offset: {}", offset);
                Err("Invalid offset")
            }
        }
    }

    pub fn write_clint(offset: usize, value: usize) -> Result<(), ()> {
        log::trace!("Write to CLINT register {:x} with a value {:?}", offset, value);
        let clint_driver = ClintDriver::new();
    
        match offset {
            0x4000..=0x4028 => {
                clint_driver.write_mtime(offset, value);
                log::trace!("MTIMECMP value written: {}", value);
                Ok(())
            }
            0xBFF8 => {
                clint_driver.write_mtime(offset, value);
                log::trace!("MTIME value written: {}", value);
                Ok(())
            }
            _ => {
                log::error!("Invalid offset: {}", offset);
                Err(())
            }
        }
    }  
}