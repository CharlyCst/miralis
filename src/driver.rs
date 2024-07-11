//! Base driver class

use core::ptr;

pub struct ClintDriver;

impl ClintDriver {

    pub fn read_mtime(address: usize) -> usize {
        unsafe { ptr::read_volatile(address as *const usize) }
    }

    pub fn write_mtime(address: usize, value: usize) {
        unsafe { ptr::write_volatile(address as *mut usize, value) };
    }

    pub fn read_msip(address: usize) -> u32 {
        unsafe { ptr::read_volatile(address as *const u32) }
    }

    pub fn write_msip(address: usize, value: u32) {
        unsafe { ptr::write_volatile(address as *mut u32, value) };
    }

    }
