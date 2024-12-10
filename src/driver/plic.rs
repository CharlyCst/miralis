//! # PLIC Driver
//!
//! This module implements a driver for the RISC-V PLIC (Plaftform-Level Interrupt Controller). It
//! is inteded to be used as a back-end for the virtual PLIC device.
//!
//! For the PLIC spec see here:
//! https://github.com/riscv/riscv-plic-spec/releases/tag/1.0.0

#[derive(Clone, Debug)]
pub struct PlicDriver {
    /// The base address of the physical PLIC.
    base: usize,
}

impl PlicDriver {
    /// Creates a new PLIC driver from the base address of the PLIC device.
    ///
    /// # Safety
    ///
    /// This function assumes that the base address corresponds to the base address of a
    /// PLIC-compatible device. In addition this function assumes that a at most one [PlicDriver]
    /// is initialized with the same base address and that no other code is accessing the PLIC
    /// device.
    pub const unsafe fn new(base: usize) -> Self {
        Self { base }
    }

    /// Add an offset to the base of the PLIC and return the resulting address.
    pub fn add_base_offset(&self, offset: usize) -> usize {
        self.base.checked_add(offset).expect("Invalid offset")
    }
}
