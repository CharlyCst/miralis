//! Base driver class

use core::ptr;

use crate::platform::{virt, Platform};

#[derive(Clone, Debug)]
pub struct ClintDriver {
    pub base_address_msip: usize,
    pub base_address_mtimecmp: usize,
    pub base_address_mtime: usize,
    pub max_hart: usize,
    pub msip_width: usize,
    pub mtimecmp_width: usize,
}

impl ClintDriver {
    pub fn new(
        base_address_msip: usize,
        base_address_mtimecmp: usize,
        base_address_mtime: usize,
        max_hart: usize,
        msip_width: usize,
        mtimecmp_width: usize,
    ) -> Self {
        Self {
            base_address_msip,
            base_address_mtimecmp,
            base_address_mtime,
            max_hart,
            msip_width,
            mtimecmp_width,
        }
    }

    fn add_base_offset(offset_address: usize) -> usize {
        offset_address + virt::VirtPlatform::get_clint_base()
    }

    // Read the current value of the machine timer (mtime)
    /// SAFETY: We derive a valid memory address from a pre-defined platform constants.
    pub fn read_mtime(&self) -> usize {
        let pointer = Self::add_base_offset(self.base_address_mtime);
        let time = unsafe { ptr::read_volatile(pointer as *const usize) };
        log::trace!("MTIME value: {}", time);
        return time;
    }

    // Write a new value to the machine timer (mtime)
    /// SAFETY: We derive a valid memory address from a pre-defined platform constants.
    pub fn write_mtime(&mut self, time: usize) {
        let pointer = Self::add_base_offset(self.base_address_mtime);
        unsafe { ptr::write_volatile(pointer as *mut usize, time) };
        log::trace!("MTIME value written: {}", time);
    }

    // Read the value of the machine timer compare (mtimecmp) for a specific hart
    /// SAFETY: We derive a valid memory address from a pre-defined platform constants.
    /// Additionally, we check that the HART with the required number exists on the board.
    pub fn read_mtimecmp(&self, hart: usize) -> Result<usize, &'static str> {
        if hart >= self.max_hart {
            return Err("Out of bounds MTIMECMP read attempt");
        }
        let pointer =
            Self::add_base_offset(self.base_address_mtimecmp + hart * self.mtimecmp_width);
        let deadline = unsafe { ptr::read_volatile(pointer as *const usize) };
        log::trace!("MTIMECMP value: {}", deadline);
        Ok(deadline)
    }

    // Write a new value to the machine timer compare (mtimecmp) for a specific hart
    /// SAFETY: We derive a valid memory address from a pre-defined platform constants.
    /// Additionally, we check that the HART with the required number exists on the board.
    pub fn write_mtimecmp(&mut self, hart: usize, deadline: usize) -> Result<(), &'static str> {
        if hart >= self.max_hart {
            return Err("Out of bounds MTIMECMP write attempt");
        }

        let pointer =
            Self::add_base_offset(self.base_address_mtimecmp + hart * self.mtimecmp_width);
        unsafe { ptr::write_volatile(pointer as *mut usize, deadline) };
        log::trace!("MTIMECMP value written: {}", deadline);
        Ok(())
    }

    // Read the value of the machine software interrupt (msip) for a specific hart
    /// SAFETY: We derive a valid memory address from a pre-defined platform constants.
    /// Additionally, we check that the HART with the required number exists on the board.
    pub fn read_msip(&self, hart: usize) -> Result<usize, &'static str> {
        if hart >= self.max_hart {
            return Err("Out of bounds MSIP read attempt");
        }

        let pointer = Self::add_base_offset(self.base_address_msip + hart * self.max_hart);
        let msip = unsafe { ptr::read_volatile((pointer) as *const u32) };
        log::trace!("MSIP value: {}", msip);
        if (msip >> 1) != 0 {
            log::warn!("Upper 31 bits of MSIP value are not zero!");
        }
        Ok(msip.try_into().unwrap())
    }

    // Write a new value to the machine software interrupt (msip) for a specific hart
    /// SAFETY: We derive a valid memory address from a pre-defined platform constants.
    /// Additionally, we check that the HART with the required number exists on the board.
    pub fn write_msip(&mut self, hart: usize, msip: u32) -> Result<(), &'static str> {
        if hart >= self.max_hart {
            return Err("Out of bounds MSIP write attempt");
        }
        let msip_value = msip & 0x1;
        let pointer = Self::add_base_offset(self.base_address_msip + hart * self.max_hart);
        unsafe { ptr::write_volatile((pointer) as *mut u32, msip_value) };
        log::trace!("MSIP value written: {}", msip_value);
        Ok(())
    }
}
