//! Base device class

use crate::driver::ClintDriver;
use crate::platform::{virt, Platform}; 

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

    pub fn calculate_offset(self_value: usize, imm: isize) -> usize {
        if imm >= 0 {
            self_value + (imm) as usize
        } else {
            self_value - (-imm) as usize
        }
    }

    pub fn read_clint(offset: usize) -> Result<usize, &'static str> {
        let offset_to_clint = offset - virt::VirtPlatform::get_clint_base();
        log::trace!("Read from CLINT address {:x}", offset_to_clint);
    
        match offset_to_clint {
            0x0 | 0x4 | 0x8 | 0xC | 0x10 => {
                let msip_value = ClintDriver::read_msip(offset);
                log::trace!("MSIP value: {}", msip_value);
                if (msip_value >> 1) != 0 {
                    log::warn!("Upper 31 bits of MSIP value are not zero!");
                }    
                Ok(msip_value.try_into().unwrap())
            }
            0x4000..=0x4028 => {
                let mtimecmp_value = ClintDriver::read_mtime(offset);
                log::trace!("MTIMECMP value: {}", mtimecmp_value);
                Ok(mtimecmp_value)
            }
            0xBFF8 => {
                let mtime_value = ClintDriver::read_mtime(offset);
                log::trace!("MTIME value: {}", mtime_value);
                Ok(mtime_value)
            }
            _ => {
                log::error!("Invalid offset: {:x}", offset_to_clint);
                Err("Invalid offset")
            }
        }
    }

    pub fn write_clint(offset: usize, value: usize) -> Result<(), ()> {
        let offset_to_clint = offset - virt::VirtPlatform::get_clint_base();
        log::trace!("Write to CLINT address {:x} with a value {:?}", offset_to_clint, value);
    
        match offset_to_clint {
            0x0 | 0x4 | 0x8 | 0xC | 0x10 => {
                let msip_value = value & 0x1;
                ClintDriver::write_msip(offset, msip_value.try_into().unwrap());
                log::trace!("MSIP value written: {}", msip_value);
                Ok(())
            }
            0x4000..=0x4028 => {
                ClintDriver::write_mtime(offset_to_clint, value);
                log::trace!("MTIMECMP value written: {}", value);
                Ok(())
            }
            0xBFF8 => {
                ClintDriver::write_mtime(offset_to_clint, value);
                log::trace!("MTIME value written: {}", value);
                Ok(())
            }
            _ => {
                log::error!("Invalid offset: {:x}", offset_to_clint);
                Err(())
            }
        }
    }  
}