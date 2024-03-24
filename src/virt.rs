//! Firmware Virtualisation

use mirage_abi::{MIRAGE_ABI_EID, MIRAGE_ABI_FAILURE_FID, MIRAGE_ABI_SUCCESS_FID};

use crate::arch::{Arch, Architecture, Csr, MCause, Register, TrapInfo};
use crate::debug;
use crate::decoder::{decode, Instr};
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
    /// Information on the trap that ocurred, used to handle traps
    pub(crate) trap_info: TrapInfo,
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
            trap_info: Default::default(),
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
    pub mstatus: usize,
    pub mtinst: usize,
    mconfigptr: usize,
    tselect: usize,
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
            mstatus: 0,
            mtinst: 0,
            mconfigptr: 0,
            tselect: 0,
        }
    }
}

// —————————————————————————— Handle Payload Traps —————————————————————————— //

impl VirtContext {
    fn emulate_instr(&mut self, instr: &Instr) {
        match instr {
            Instr::Wfi => {
                todo!("wfi is not yet supported");
            }
            Instr::Csrrw { csr, .. }
            | Instr::Csrrs { csr, .. }
            | Instr::Csrrc { csr, .. }
            | Instr::Csrrwi { csr, .. }
            | Instr::Csrrsi { csr, .. }
            | Instr::Csrrci { csr, .. }
                if csr.is_unknown() =>
            {
                self.emulate_jump_trap_handler();
            }
            Instr::Csrrw { csr, rd, rs1 } => {
                let tmp = self.get(csr);
                self.set(csr, self.get(rs1));
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrs { csr, rd, rs1 } => {
                let tmp = self.get(csr);
                self.set(csr, tmp | self.get(rs1));
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrwi { csr, rd, uimm } => {
                self.set(rd, self.get(csr));
                self.set(csr, *uimm);
                self.pc += 4;
            }
            Instr::Csrrsi { csr, rd, uimm } => {
                let tmp = self.get(csr);
                self.set(csr, tmp | uimm);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrc { csr, rd, rs1 } => {
                let tmp = self.get(csr);
                self.set(csr, tmp & !self.get(rs1));
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrci { csr, rd, uimm } => {
                let tmp = self.get(csr);
                self.set(csr, tmp & !uimm);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Mret => {
                if ((self.csr.mstatus >> 11) & 0b11) != 3 {
                    panic!(
                        "MRET is not going to M mode: {} with MPP {}",
                        self.csr.mstatus,
                        ((self.csr.mstatus >> 11) & 0b11)
                    );
                }
                // Modify mstatus
                // ONLY WITH HYPERVISOR EXTENSION : MPV = 0,
                // self.csr.mstatus = self.csr.mstatus & !(0b1 << 39);

                // MPP = 0, MIE= MPIE, MPIE = 1, MPRV = 0
                let mpie = 0b1 & (self.csr.mstatus >> 7);

                // TODO: create some constants to make it easier to understand what is going on
                // here
                self.csr.mstatus = self.csr.mstatus | 0b1 << 7;
                self.csr.mstatus = self.csr.mstatus & !(0b1 << 3);
                self.csr.mstatus = self.csr.mstatus | mpie << 3;
                self.csr.mstatus = self.csr.mstatus & !(0b11 << 11);

                // Jump back to payload
                self.pc = self.csr.mepc;
            }
            _ => todo!("Instruction not yet implemented: {:?}", instr),
        }
    }

    fn emulate_jump_trap_handler(&mut self) {
        // We are now emulating a trap, registers need to be updated
        log::trace!("Emulating jump to trap handler");
        self.csr.mcause = self.trap_info.mcause;
        self.csr.mstatus = self.trap_info.mstatus; // TODO: are we leaking information from Mirage?
        self.csr.mtval = self.trap_info.mtval;
        self.csr.mip = self.trap_info.mip;
        self.csr.mepc = self.trap_info.mepc;

        // Modify mstatus: previous privilege mode is Machine = 3
        self.csr.mstatus = self.csr.mstatus | 0b11 << 11;

        // Go to payload trap handler
        assert!(
            self.csr.mtvec & 0b11 == 0,
            "Only direct mode is supported for mtvec"
        );
        self.pc = self.csr.mtvec
    }

    /// Handle the trap coming from the payload
    pub fn handle_payload_trap(&mut self) {
        // Keep track of the number of exit
        self.nb_exits += 1;

        let cause = self.trap_info.get_cause();
        match cause {
            MCause::EcallFromUMode if self.get(Register::X17) == MIRAGE_ABI_EID => {
                let fid = self.get(Register::X16);
                match fid {
                    MIRAGE_ABI_FAILURE_FID => {
                        log::error!("Payload panicked!");
                        log::error!("  pc:    0x{:x}", self.pc);
                        log::error!("  exits: {}", self.nb_exits);
                        unsafe { debug::log_stack_usage() };
                        Plat::exit_failure();
                    }
                    MIRAGE_ABI_SUCCESS_FID => {
                        log::info!("Success!");
                        log::info!("Number of payload exits: {}", self.nb_exits);
                        unsafe { debug::log_stack_usage() };
                        Plat::exit_success();
                    }
                    _ => panic!("Invalid Mirage FID: 0x{:x}", fid),
                }
            }
            MCause::EcallFromUMode => {
                todo!("ecall is not yet supported for EID other than Mirage ABI");
            }
            MCause::IllegalInstr => {
                let instr = unsafe { Arch::get_raw_faulting_instr(&self.trap_info) };
                let instr = decode(instr);
                log::trace!("Faulting instruction: {:?}", instr);
                self.emulate_instr(&instr);
            }
            MCause::Breakpoint => {
                self.emulate_jump_trap_handler();
            }
            _ => {
                if cause.is_interrupt() {
                    // TODO : Interrupts are not yet supported
                    todo!("Interrupts are not yet implemented");
                } else {
                    // TODO : Need to match other traps
                    todo!("Other traps are not yet implemented");
                }
            }
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
            Csr::Mstatus => self.csr.mstatus,
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
            Csr::Medeleg => todo!(),                // TODO : normal read
            Csr::Mideleg => todo!(),                // TODO : normal read
            Csr::Mtinst => todo!(),                 // TODO : normal read
            Csr::Mtval2 => todo!(),                 // TODO : normal read
            Csr::Tdata1 => todo!(),                 // TODO : normal read
            Csr::Tdata2 => todo!(),                 // TODO : normal read
            Csr::Tdata3 => todo!(),                 // TODO : normal read
            Csr::Mcontext => todo!(),               // TODO : normal read
            Csr::Dcsr => todo!(),                   // TODO : normal read
            Csr::Dpc => todo!(),                    // TODO : normal read
            Csr::Dscratch0 => todo!(),              // TODO : normal read
            Csr::Dscratch1 => todo!(),              // TODO : normal read
            Csr::Mconfigptr => self.csr.mconfigptr, // Read-only
            Csr::Tselect => self.csr.tselect,       // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mepc => self.csr.mepc,
            Csr::Mcause => self.csr.mcause,
            Csr::Mtval => self.csr.mtval,
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
        }
    }

    fn set(&mut self, register: Csr, value: usize) {
        match register {
            Csr::Mhartid => (), // Read-only
            Csr::Mstatus => {
                // TODO: create some constant values
                let mut new_value = self.csr.mstatus;
                // MPP : 11 : write legal : 0,1,2,3
                let mpp = (value >> 11) & 0b11;
                if mpp == 0 || mpp == 1 || mpp == 3 {
                    // Legal values
                    new_value = new_value & !(0b11 << 11); // clear MPP
                    new_value = new_value | (mpp << 11); // set new MPP
                }
                // SXL : 34 : read-only : MX-LEN = 64
                let mxl: usize = 2;
                new_value = new_value & !(0b11 << 34); // clear SXL
                new_value = new_value | (mxl << 34); // set new SXL
                                                     // UXL : 32 : read-only : MX-LEN = 64
                new_value = new_value & !(0b11 << 32); // clear UXL
                new_value = new_value | (mxl << 32); // set new UXL

                // MPRV : 17 : write anything
                // MBE : 37 : write anything
                // SBE : 36 : read-only 0 (NO S-MODE)
                new_value = new_value & !(0b1 << 36); // clear SBE
                                                      // UBE : 6 : write anything
                                                      // TVM : 20 : read-only 0 (NO S-MODE)
                new_value = new_value & !(0b1 << 20); // clear TVM
                                                      // TW : 21 : write anything
                                                      // TSR : 22 : read-only 0 (NO S-MODE)
                new_value = new_value & !(0b11 << 22); // clear TSR
                                                       // FS : 13 : read-only 0 (NO S-MODE, F extension)
                new_value = new_value & !(0b11 << 13); // clear TSR
                                                       // VS : 9 : read-only 0 (v registers)
                new_value = new_value & !(0b11 << 9); // clear TSR
                                                      // XS : 15 : read-only 0 (NO FS nor VS)
                new_value = new_value & !(0b11 << 15); // clear TSR
                                                       // SD : 63 : read-only 0 (if NO FS/VS/XS)
                new_value = new_value & !(0b1 << 63); // clear TSR

                self.csr.mstatus = new_value;
            }
            Csr::Misa => {
                let arch_misa: usize = Arch::read_misa(); // misa shows the extensions available : we cannot have more than possible in hardware

                let mut config_misa: usize = 0x0; // Features available for payload
                config_misa = config_misa
                    | match get_payload_s_mode() {
                        None => 0b0,
                        Some(b) => (if b { 0b0 } else { 0b1 }) << 18, //Mark the ones that are not available by config
                    };
                config_misa = !config_misa;

                let mirage_misa: usize = 0x8000000000000000; // Features required by Mirage : MXLEN = 2 => 64 bits

                let change_filter: usize = 0x0000000003FFFFFF; // Filters the values that can be modified by the payload
                let mut new_misa: usize =
                    (value & config_misa & arch_misa & change_filter) | mirage_misa;

                self.csr.misa = new_misa;
            }
            Csr::Mie => self.csr.mie = value,
            Csr::Mip => {
                // TODO: handle mip emulation properly
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
            Csr::Mconfigptr => (),     // Read-only
            Csr::Medeleg => todo!(), // TODO : This register should not exist in a system without S-mode
            Csr::Mideleg => todo!(), // TODO : This register should not exist in a system without S-mode
            Csr::Mtinst => todo!(), // TODO : Can only be written automatically by the hardware on a trap, this register should not exist in a system without hypervisor extension
            Csr::Mtval2 => todo!(), // TODO : Must be able to hold 0 and may hold an arbitrary number of 2-bit-shifted guest physical addresses, written alongside mtval, this register should not exist in a system without hypervisor extension
            Csr::Tselect => (),     // Read-only 0 when no triggers are implemented
            Csr::Tdata1 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata2 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata3 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mcontext => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dcsr => todo!(),   // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dpc => todo!(),    // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dscratch0 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dscratch1 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mepc => {
                if value > Plat::get_max_valid_address() {
                    return;
                }
                self.csr.mepc = value
            }
            Csr::Mcause => {
                let cause = MCause::new(value);
                if cause.is_interrupt() {
                    // TODO : does not support interrupts
                    return;
                }
                match cause {
                    // Can only contain supported exception codes
                    MCause::UnknownException => (),
                    _ => self.csr.mcause = value,
                }
            }
            Csr::Mtval => (), // TODO : PLATFORM DEPENDANCE (if trapping writes to mtval or not) : Mtval is read-only 0 for now : must be able to contain valid address and zero
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
