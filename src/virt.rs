//! Firmware Virtualisation

use core::ops::{Index, IndexMut};

use crate::arch::{Csr, Register};

/// The context of a virtual firmware.
#[derive(Debug, Default)]
#[repr(C)]
pub struct VirtContext {
    /// Stack pointer of the host, used to restore context on trap.
    host_stack: usize,
    /// Basic registers
    regs: [usize; 32],
    pub csr: VirtCsr,
}

/// Control and Status Registers (CSR) for a virtual firmware.
#[derive(Debug, Default)]
pub struct VirtCsr {
    mtvec: usize,
    mscratch: usize,
}

// ————————————————————————————————— Index —————————————————————————————————— //

impl Index<Register> for VirtContext {
    type Output = usize;

    fn index(&self, index: Register) -> &Self::Output {
        &self.regs[index as usize]
    }
}

impl IndexMut<Register> for VirtContext {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.regs[index as usize]
    }
}

impl Index<Csr> for VirtContext {
    type Output = usize;

    fn index(&self, index: Csr) -> &Self::Output {
        match index {
            Csr::Mstatus => todo!("CSR not yet implemented"),
            Csr::Mtvec => &self.csr.mtvec,
            Csr::Mscratch => &self.csr.mscratch,
            Csr::Unknown => panic!("Tried to access unknown CSR"),
        }
    }
}

impl IndexMut<Csr> for VirtContext {
    fn index_mut(&mut self, index: Csr) -> &mut Self::Output {
        match index {
            Csr::Mstatus => todo!("CSR not yet implemented"),
            Csr::Mtvec => &mut self.csr.mtvec,
            Csr::Mscratch => &mut self.csr.mscratch,
            Csr::Unknown => panic!("Tried to access unknown CSR"),
        }
    }
}
