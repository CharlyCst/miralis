//! Host (Mirage) Context
//!
//! This module exposes the host context as [MirageCtx], which holds Mirage's own configuration registers.

use crate::arch::pmp::PmpGroup;
use crate::arch::HardwareCapability;
use crate::device;
use crate::platform::{Plat, Platform};

/// The Mirage Context, holding configuration registers for Mirage.
pub struct MirageContext {
    /// Configuration of the host PMP
    pub pmp: PmpGroup,
    /// The offset of the virutal PMP registers, compared to physical PMP.
    pub virt_pmp_offset: u8,
    /// Hardware capabilities of the core (hart).
    pub hw: HardwareCapability,
    /// List of devices with PMP
    pub devices: [device::Device; 1],
}

impl MirageContext {
    /// Creates a new Mirage context with default values.
    pub fn new(nb_pmp: usize, hw: HardwareCapability) -> Self {
        Self {
            pmp: PmpGroup::new(nb_pmp),
            virt_pmp_offset: 0,
            hw,
            devices: [Plat::create_clint_device()],
        }
    }
}
