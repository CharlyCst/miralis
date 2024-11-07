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
    /// Hardware capabilities of the core (hart).
    pub hw: HardwareCapability,
    /// List of device with PMP
    pub devices: [device::VirtDevice; 2],
}

impl MiralisContext {
    /// Creates a new Miralis context with default values.
    pub fn new(hw: HardwareCapability) -> Self {
        Self {
            pmp: PmpGroup::init_pmp_group(hw.available_reg.nb_pmp),
            hw,
            devices: Plat::create_virtual_devices(),
        }
    }
}
