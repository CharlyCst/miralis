//! Firmware Virtualisation

use crate::arch::{Arch, Architecture, Csr, Register};
use crate::platform::{Plat, Platform};

/// The context of a virtual firmware.
#[derive(Debug)]
#[repr(C)]
pub struct VirtContext {
    /// Stack pointer of the host, used to restore context on trap.
    host_stack: usize,
    /// Basic registers
    regs: [usize; 32],
    /// Virtual Control and Status Registers
    csr: VirtCsr,
    /// Number of virtual PMPs
    nbr_pmps: usize,
    /// Hart ID
    hart_id: usize,
    /// Number of exists to Mirage
    pub(crate) nb_exits: usize,
}

impl VirtContext {
    pub fn new(hart_id: usize) -> Self {
        return VirtContext {
            host_stack: 0,
            regs: Default::default(),
            csr: Default::default(),
            nb_exits: 0,
            hart_id,
            nbr_pmps : match Plat::get_nb_pmp() {
                0 => 0,
                16 => 0,
                64 => 16,
                _ => 0,
            },
        };

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
    pmp_cfg: [usize; 16],
    pmp_addr: [[usize; 32]; 2],
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
            Csr::Pmpcfg(pmp_cfg_idx) => {
                if pmp_cfg_idx % 2 == 1 {
                    // Illegal because we are in a RISCV64 setting
                    panic!("Illegal PMP_CFG {:?}", register)
                }
                if pmp_cfg_idx >= self.nbr_pmps / 8 {
                    //This PMP is not emulated
                    return 0;
                }
                self.csr.pmp_cfg[pmp_cfg_idx]
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= self.nbr_pmps {
                    //This PMP is not emulated
                    return 0;
                }
                self.csr.pmp_addr[if pmp_addr_idx < 32 { 0 } else { 1 }][if pmp_addr_idx < 32 {
                    pmp_addr_idx
                } else {
                    pmp_addr_idx - 32
                }]
            }
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
            Csr::Mvendorid => (), // Read-only
            Csr::Marchid => (),   // Read-only
            Csr::Mimpid => (),    // Read-only
            Csr::Pmpcfg(pmp_cfg_idx) => {
                let _locks = Csr::PMP_CFG_LOCK_MASK & value;

                if _locks != 0 {
                    panic!("PMP lock bits are not yet supported")
                }

                let _legal_value = Csr::PMP_CFG_LEGAL_MASK & value;
                if pmp_cfg_idx % 2 == 1 {
                    // Illegal because we are in a RISCV64 setting
                    panic!("Illegal PMP_CFG {:?}", register)
                }
                if pmp_cfg_idx >= self.nbr_pmps / 8 {
                    //This PMP is not emulated
                    return
                }
                self.csr.pmp_cfg[pmp_cfg_idx] = _legal_value
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                let _legal_value = Csr::PMP_ADDR_LEGAL_MASK & value;

                if pmp_addr_idx >= self.nbr_pmps {
                    //This PMP is not emulated
                    return
                }
                self.csr.pmp_addr[if pmp_addr_idx < 32 { 0 } else { 1 }][if pmp_addr_idx < 32 {
                    pmp_addr_idx
                } else {
                    pmp_addr_idx - 32
                }] = _legal_value
            }
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
