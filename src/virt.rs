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
    /// Program Counter
    pub(crate) pc: usize,
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
        VirtContext {
            host_stack: 0,
            regs: Default::default(),
            csr: Default::default(),
            pc: 0,
            nb_exits: 0,
            hart_id,
            nbr_pmps: match Plat::get_nb_pmp() {
                0 => 0,
                16 => 0,
                64 => 16,
                _ => 0,
            },
        }
    }
}

/// Control and Status Registers (CSR) for a virtual firmware.
#[derive(Debug)]
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
    pmp_addr: [usize; 64],
    mcycle: usize,
    minstret: usize,
    mhpmcounter: [usize; 29],
    mcountinhibit: usize,
    mhpmevent: [usize; 29],
    mcounteren: usize,
    menvcfg: usize,
    mseccfg: usize,
    mcause: usize,
    mepc: usize,
    mtval: usize,
}

impl Default for VirtCsr {
    fn default() -> VirtCsr {
        VirtCsr {
            misa: 0,
            mie: 0,
            mip: 0,
            mtvec: 0,
            mscratch: 0,
            mvendorid: 0,
            marchid: 0,
            mimpid: 0,
            pmp_cfg: [0; 16],
            pmp_addr: [0; 64],
            mcycle: 0,
            minstret: 0,
            mhpmcounter: [0; 29],
            mcountinhibit: 0,
            mhpmevent: [0; 29],
            mcounteren: 0,
            menvcfg: 0,
            mseccfg: 0,
            mcause: 0,
            mepc: 0,
            mtval: 0,
        }
    }
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
                    // This PMP is not emulated
                    return 0;
                }
                self.csr.pmp_cfg[pmp_cfg_idx]
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= self.nbr_pmps {
                    // This PMP is not emulated
                    return 0;
                }
                self.csr.pmp_addr[pmp_addr_idx]
            }
            Csr::Mcycle => self.csr.mcycle,
            Csr::Minstret => self.csr.minstret,
            Csr::Mhpmcounter(n) => self.csr.mhpmcounter[n],
            Csr::Mcountinhibit => self.csr.mcountinhibit,
            Csr::Mhpmevent(n) => self.csr.mhpmevent[n],
            Csr::Mcounteren => self.csr.mcounteren,
            Csr::Menvcgf => self.csr.menvcfg,
            Csr::Mseccfg => self.csr.mseccfg,
            Csr::Mconfigptr => todo!(), // TODO : Read-only, can be read-only 0
            Csr::Medeleg => todo!(),    // TODO : normal read
            Csr::Mideleg => todo!(),    // TODO : normal read
            Csr::Mtinst => todo!(),     // TODO : normal read
            Csr::Mtval2 => todo!(),     // TODO : normal read
            Csr::Tselect => todo!(),    // TODO : normal read
            Csr::Tdata1 => todo!(),     // TODO : normal read
            Csr::Tdata2 => todo!(),     // TODO : normal read
            Csr::Tdata3 => todo!(),     // TODO : normal read
            Csr::Mcontext => todo!(),   // TODO : normal read
            Csr::Dcsr => todo!(),       // TODO : normal read
            Csr::Dpc => todo!(),        // TODO : normal read
            Csr::Dscratch0 => todo!(),  // TODO : normal read
            Csr::Dscratch1 => todo!(),  // TODO : normal read
            Csr::Mepc => self.csr.mepc,
            Csr::Mcause => self.csr.mcause,
            Csr::Mtval => self.csr.mtval,
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
        }
    }

    fn set(&mut self, register: Csr, value: usize) {
        match register {
            Csr::Mhartid => (), // Read-only
            Csr::Mstatus => todo!("CSR not yet implemented"),
            Csr::Misa => {
                // misa shows the extensions available : we cannot have more than possible in hardware
                let arch_misa: usize = Arch::read_misa();
                let mut config_misa: usize = 0x00000000000000000; // Features required by the payload

                config_misa = config_misa
                    | match get_payload_s_mode() {
                        None => 0b0,
                        Some(b) => (if b { 0b1 } else { 0b0 }) << 18,
                    };

                let mirage_misa: usize = 0x2 << 62; // Features required by Mirage : MXLEN = 2 => 64 bits
                let change_filter: usize = 0x00000000003FFFFFF; // Filters the values that can be modified by the payload
                let mut new_misa: usize =
                    ((value | config_misa) & arch_misa & change_filter) | mirage_misa;

                self.csr.misa = new_misa;
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
                if Csr::PMP_CFG_LOCK_MASK & value != 0 {
                    panic!("PMP lock bits are not yet supported")
                } else if pmp_cfg_idx % 2 == 1 {
                    // Illegal because we are in a RISCV64 setting
                    panic!("Illegal PMP_CFG {:?}", register)
                } else if pmp_cfg_idx >= self.nbr_pmps / 8 {
                    // This PMP is not emulated, ignore changes
                    return;
                }
                self.csr.pmp_cfg[pmp_cfg_idx] = Csr::PMP_CFG_LEGAL_MASK & value;
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= self.nbr_pmps {
                    // This PMP is not emulated, ignore
                    return;
                }
                self.csr.pmp_addr[pmp_addr_idx] = Csr::PMP_ADDR_LEGAL_MASK & value;
            }
            Csr::Mcycle => (),                    // Read-only 0
            Csr::Minstret => (),                  // Read-only 0
            Csr::Mhpmcounter(_counter_idx) => (), // Read-only 0
            Csr::Mcountinhibit => (),             // Read-only 0
            Csr::Mhpmevent(_event_idx) => (),     // Read-only 0
            Csr::Mcounteren => (),                // Read-only 0
            Csr::Menvcgf => self.csr.menvcfg = value,
            Csr::Mseccfg => self.csr.mseccfg = value,
            Csr::Mconfigptr => todo!(), // TODO : Read-only, can be read-only 0
            Csr::Medeleg => todo!(), // TODO : This register should not exist in a system without S-mode
            Csr::Mideleg => todo!(), // TODO : This register should not exist in a system without S-mode
            Csr::Mtinst => todo!(), // TODO : Can only be written automatically by the hardware on a trap
            Csr::Mtval2 => todo!(), // TODO : Must be able to hold 0 and may hold an arbitrary number of 2-bit-shifted guest physical addresses, written alongside mtval
            Csr::Tselect => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata1 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata2 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata3 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mcontext => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dcsr => todo!(),   // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dpc => todo!(),    // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dscratch0 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dscratch1 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mepc => todo!(),   // TODO : must contain a valid address
            Csr::Mcause => todo!(), // TODO : can only contain supported exception codes
            Csr::Mtval => todo!(),  // TODO : must contain a valid address and zero
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

pub fn get_payload_s_mode() -> Option<bool> {
    match option_env!("MIRAGE_PAYLOAD_S_MODE") {
        Some(env_var) => match env_var.parse() {
            Ok(b) => Some(b),
            Err(_) => None,
        },
        None => None,
    }
}
