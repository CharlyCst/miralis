//! Base driver class

use core::ptr;

use crate::platform::{virt, Platform}; 
pub struct ClintDriver{
    pub base_address: usize,
}

impl ClintDriver {

    pub fn new() -> Self {
        Self {
            base_address: virt::VirtPlatform::get_clint_base(),
        }
    }

    fn calculate_address(&self, offset: usize) -> usize {
        self.base_address + offset as usize
    }

    pub fn read_mtime(&self, address: usize) -> usize {
        let address = self.calculate_address(address);
        unsafe { ptr::read_volatile(address as *const usize) }
    }

    pub fn write_mtime(&self, address: usize, value: usize) {
        let address = self.calculate_address(address);
        unsafe { ptr::write_volatile(address as *mut usize, value) };
    }

    }
