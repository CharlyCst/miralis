//! Host (Mirage) Context
//!
//! This module exposes the host context as [MirageCtx], which holds Mirage's own configuration registers.

use crate::arch::pmp::PmpGroup;

/// The Mirage Context, holding configuration registers for Mirage.
pub struct MirageContext {
    /// Configuration of the host PMP
    pub pmp: PmpGroup,
    /// The offset of the virutal PMP registers, compared to physical PMP.
    pub virt_pmp_offset: u8,
}

impl MirageContext {
    /// Creates a new Mirage context with default values.
    pub fn new(nb_pmp: usize) -> Self {
        Self {
            pmp: PmpGroup::new(nb_pmp),
            virt_pmp_offset: 0,
        }
    }
}
