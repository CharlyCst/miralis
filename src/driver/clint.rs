//! # CLINT Driver
//!
//! This module implements a driver for the RISC-V CLINT (Core Local Interruptor). It is inteded to
//! be used both as a back-end for the virtual CLINT device and for directly programming M-mode
//! interrupts from Miralis.

use core::ptr;

use crate::arch::Width;
use crate::config::{self, PLATFORM_NB_HARTS};
use crate::logger;

pub const MSIP_OFFSET: usize = 0x0;
pub const MTIMECMP_OFFSET: usize = 0x4000;
pub const MTIME_OFFSET: usize = 0xBFF8;

pub const MSIP_WIDTH: Width = Width::Byte4;
pub const MTIMECMP_WIDTH: Width = Width::Byte8;
pub const _MTIME_WIDTH: Width = Width::Byte8;

#[derive(Clone, Debug)]
pub struct ClintDriver {
    /// The base address of the physical CLINT.
    base: usize,
}

impl ClintDriver {
    /// Creates a new CLINT driver from the base address of the CLINT device.
    ///
    /// # Safety
    ///
    /// This function assumes that the base address corresponds to the base address of a
    /// CLINT-compatible device. In addition this function assumes that a at most one [ClintDriver]
    /// is initialized with the same base address and that no other code is accessing the CLINT
    /// device.
    pub const unsafe fn new(base: usize) -> Self {
        Self { base }
    }

    fn add_base_offset(&self, offset: usize) -> usize {
        self.base.checked_add(offset).expect("Invalid offset")
    }

    /// Read the current value of the machine timer (mtime)
    pub fn read_mtime(&self) -> usize {
        let pointer = self.add_base_offset(MTIME_OFFSET);

        // SAFETY: We derive a valid memory address assuming the base points to a valid CLINT
        // device.
        let time = unsafe { ptr::read_volatile(pointer as *const usize) };
        logger::trace!("MTIME value: 0x{:x}", time);

        time
    }

    /// Write a new value to the machine timer (mtime)
    pub fn write_mtime(&mut self, time: usize) {
        let pointer = self.add_base_offset(MTIME_OFFSET);

        // SAFETY: We derive a valid memory address assuming the base points to a valid CLINT
        // device. Moreover, we take `self` with &mut reference to enforce aliasing rules.
        unsafe { ptr::write_volatile(pointer as *mut usize, time) };
        logger::trace!("MTIME value written: 0x{:x}", time);
    }

    ///  Read the value of the machine timer compare (mtimecmp) for a specific hart
    pub fn read_mtimecmp(&self, hart: usize) -> Result<usize, &'static str> {
        if hart >= config::PLATFORM_NB_HARTS {
            log::warn!(
                "Tried to read MTIMECMP for hart {}, but only {} hart(s) are available",
                hart,
                config::PLATFORM_NB_HARTS
            );
            return Err("Out of bounds MTIMECMP read attempt");
        }
        let pointer = self.add_base_offset(MTIMECMP_OFFSET + hart * MTIMECMP_WIDTH.to_bytes());

        // SAFETY: We checked that the number of hart is within the platform limit, which ensures
        // the read is contained within the MTIMECMP area of the CLINT.
        let deadline = unsafe { ptr::read_volatile(pointer as *const usize) };
        logger::trace!("MTIMECMP value: 0x{:x}", deadline);
        Ok(deadline)
    }

    /// Write a new value to the machine timer compare (mtimecmp) for a specific hart
    pub fn write_mtimecmp(&mut self, hart: usize, deadline: usize) -> Result<(), &'static str> {
        if hart >= config::PLATFORM_NB_HARTS {
            log::warn!(
                "Tried to write MTIMECMP for hart {}, but only {} hart(s) are available",
                hart,
                config::PLATFORM_NB_HARTS
            );
            return Err("Out of bounds MTIMECMP write attempt");
        }
        let pointer = self.add_base_offset(MTIMECMP_OFFSET + hart * MTIMECMP_WIDTH.to_bytes());

        // SAFETY: We checked that the number of hart is within the platform limit, which ensures
        // the read is contained within the MTIMECMP area of the CLINT. Moreover, we take `self`
        // with a &mut reference to enforce aliasing rules.
        unsafe { ptr::write_volatile(pointer as *mut usize, deadline) };
        logger::trace!("MTIMECMP value written: 0x{:x}", deadline);
        Ok(())
    }

    /// Read the value of the machine software interrupt (msip) for a specific hart.
    pub fn read_msip(&self, hart: usize) -> Result<usize, &'static str> {
        if hart >= config::PLATFORM_NB_HARTS {
            log::warn!(
                "Tried to read MSIP for hart {}, but only {} hart(s) are available",
                hart,
                config::PLATFORM_NB_HARTS
            );
            return Err("Out of bounds MSIP read attempt");
        }
        let pointer = self.add_base_offset(MSIP_OFFSET + hart * MSIP_WIDTH.to_bytes());

        // SAFETY: We checked that the number of hart is within the platform limit, which ensures
        // the read is contained within the MSIP area of the CLINT.
        let msip = unsafe { ptr::read_volatile((pointer) as *const u32) };
        logger::trace!("MSIP value: 0x{:x}", msip);
        if (msip >> 1) != 0 {
            log::warn!("Upper 31 bits of MSIP value are not zero!");
        }
        Ok(msip.try_into().unwrap())
    }

    /// Write a new value to the machine software interrupt (msip) for a specific hart.
    pub fn write_msip(&mut self, hart: usize, msip: u32) -> Result<(), &'static str> {
        if hart >= PLATFORM_NB_HARTS {
            log::warn!(
                "Tried to write MSIP for hart {}, but only {} hart(s) are available",
                hart,
                config::PLATFORM_NB_HARTS
            );
            return Err("Out of bounds MSIP write attempt");
        }
        let msip_value = msip & 0x1;
        let pointer = self.add_base_offset(MSIP_OFFSET + hart * MSIP_WIDTH.to_bytes());

        // SAFETY: We checked that the number of hart is within the platform limit, which ensures
        // the read is contained within the MSIP area of the CLINT. Moreover, we take `self`
        // with a &mut reference to enforce aliasing rules.
        unsafe { ptr::write_volatile((pointer) as *mut u32, msip_value) };
        logger::trace!("MSIP value written: 0x{:x} for hart {hart}", msip_value);
        Ok(())
    }

    /// Create a pending MSI interrupts for each harts of the platform, including the current one.
    pub fn trigger_msi_on_all_harts(&mut self, mask: usize) {
        for i in 0..PLATFORM_NB_HARTS {
            if mask & (1 << i) != 0 {
                self.write_msip(i, 1).unwrap();
            }
        }
    }
}
