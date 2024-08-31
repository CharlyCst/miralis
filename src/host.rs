//! Host (Miralis) Context
//!
//! This module exposes the host context as [MiralisCtx], which holds Miralis's own configuration registers.

use crate::arch::pmp::PmpGroup;
use crate::arch::HardwareCapability;
use crate::device;
use crate::platform::{Plat, Platform};

/// The Miralis Context, holding configuration registers for Miralis.
pub struct MiralisContext {
    /// Configuration of the host PMP
    pub pmp: PmpGroup,
    /// The offset of the virutal PMP registers, compared to physical PMP.
    pub virt_pmp_offset: u8,
    /// Hardware capabilities of the core (hart).
    pub hw: HardwareCapability,
    /// List of device with PMP
    pub devices: [device::VirtDevice; 2],
}

impl MiralisContext {
    /// Creates a new Miralis context with default values.
    pub fn new(hw: HardwareCapability) -> Self {
        Self {
            pmp: PmpGroup::new(hw.available_reg.nb_pmp),
            virt_pmp_offset: 0,
            hw,
            devices: Plat::create_virtual_devices(),
        }
    }
}
