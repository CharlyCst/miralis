//! Firmware Virtualisation

use crate::arch::{Arch, Architecture, Csr, Register};

/// The context of a virtual firmware.
#[derive(Debug)]
#[repr(C)]
pub struct VirtContext {
    /// Stack pointer of the host, used to restore context on trap.
    host_stack: usize,
    /// Basic registers
    regs: [usize; 32],
    /// Virtual Constrol and Status Registers
    csr: VirtCsr,
    /// Hart ID
    hart_id: usize,
}

impl VirtContext {
    pub fn new(hart_id: usize) -> Self {
        VirtContext {
            host_stack: 0,
            regs: Default::default(),
            csr: Default::default(),
            hart_id,
        }
    }
}

/// Control and Status Registers (CSR) for a virtual firmware.
#[derive(Debug, Default)]
pub struct VirtCsr {
    misa: usize,
    mie: usize,
    mip: usize,
    mtvec: usize,
    mscratch: usize,
    mvendorid: usize,
    marchid: usize,
    mimpid: usize,
}

// ———————————————————————— Register Setters/Getters ———————————————————————— //

/// A trait implemented by virtual contexts to read and write registers.
pub trait RegisterContext<R> {
    fn get(&self, register: R) -> usize;
    fn set(&mut self, register: R, value: usize);
}

impl RegisterContext<Register> for VirtContext {
    fn get(&self, register: Register) -> usize {
        // NOTE: Register x0 is never set, so always keeps a value of 0
        self.regs[register as usize]
    }

    fn set(&mut self, register: Register, value: usize) {
        // Skip register x0
        if register == Register::X0 {
            return;
        }
        self.regs[register as usize] = value;
    }
}

impl RegisterContext<Csr> for VirtContext {
    fn get(&self, register: Csr) -> usize {
        match register {
            Csr::Mhartid => self.hart_id,
            Csr::Mstatus => todo!("CSR not yet implemented"),
            Csr::Misa => self.csr.misa,
            Csr::Mie => self.csr.mie,
            Csr::Mip => self.csr.mip,
            Csr::Mtvec => self.csr.mtvec,
            Csr::Mscratch => self.csr.mscratch,
            Csr::Mvendorid => self.csr.mvendorid,
            Csr::Marchid => self.csr.marchid,
            Csr::Mimpid => self.csr.mimpid,
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
        }
    }

    fn set(&mut self, register: Csr, value: usize) {
        match register {
            Csr::Mhartid => (), // Read-only
            Csr::Mstatus => todo!("CSR not yet implemented"),
            Csr::Misa => {
                // TODO: handle misa emulation properly
                if value != Arch::read_misa() {
                    // For now we panic if the payload tries to update misa. In the future we will
                    // most likely have a mask of minimal features required by Mirage and pass the
                    // changes through.
                    panic!("misa emulation is not yet implemented");
                }
                self.csr.misa = value;
            }
            Csr::Mie => self.csr.mie = value,
            Csr::Mip => {
                // TODO: handle misa emulation properly
                if value != 0 {
                    // We only support resetting mip for now
                    panic!("mip emulation is not yet implemented");
                }
                self.csr.mip = value;
            }
            Csr::Mtvec => self.csr.mtvec = value,
            Csr::Mscratch => self.csr.mscratch = value,
            Csr::Mvendorid => (), //Read-only
            Csr::Marchid => (),   //Read-only
            Csr::Mimpid => (),    //Read-only
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
        }
    }
}

/// Forward RegisterContext implementation for register references
impl<'a, R> RegisterContext<&'a R> for VirtContext
where
    R: Copy,
    VirtContext: RegisterContext<R>,
{
    fn get(&self, register: &'a R) -> usize {
        self.get(*register)
    }

    fn set(&mut self, register: &'a R, value: usize) {
        self.set(*register, value)
    }
}