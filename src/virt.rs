//! Firmware Virtualisation

use mirage_core::abi;

use crate::arch::{misa, mstatus, Arch, Architecture, Csr, MCause, Register, TrapInfo};
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
    sie: usize,
    stvec: usize,
    scounteren: usize,
    senvcfg: usize,
    sscratch: usize,
    sepc: usize,
    scause: usize,
    stval: usize,
    sip: usize,
    satp: usize,
    scontext: usize,
    medeleg: usize,
    mideleg: usize,
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
            sie: 0,
            stvec: 0,
            scounteren: 0,
            senvcfg: 0,
            sscratch: 0,
            sepc: 0,
            scause: 0,
            stval: 0,
            sip: 0,
            satp: 0,
            scontext: 0,
            medeleg: 0,
            mideleg: 0,
        }
    }
}

impl VirtCsr {
    pub fn set_mstatus_field(csr: &mut usize, offset: usize, filter: usize, value: usize) {
        //Clear field
        *csr = *csr & !(filter << offset);
        //Set field
        *csr = *csr | (value << offset);
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
                if ((self.csr.mstatus >> mstatus::MPP_OFFSET) & mstatus::MPP_FILTER) != 3 {
                    panic!(
                        "MRET is not going to M mode: {} with MPP {}",
                        self.csr.mstatus,
                        ((self.csr.mstatus >> mstatus::MPP_OFFSET) & mstatus::MPP_FILTER)
                    );
                }
                // Modify mstatus
                // ONLY WITH HYPERVISOR EXTENSION : MPV = 0,
                if false {
                    VirtCsr::set_mstatus_field(
                        &mut self.csr.mstatus,
                        mstatus::MPV_OFFSET,
                        mstatus::MPV_FILTER,
                        0,
                    );
                }

                // MPP = 0, MIE= MPIE, MPIE = 1, MPRV = 0
                let mpie = mstatus::MPIE_FILTER & (self.csr.mstatus >> mstatus::MPIE_OFFSET);

                VirtCsr::set_mstatus_field(
                    &mut self.csr.mstatus,
                    mstatus::MPIE_OFFSET,
                    mstatus::MPIE_FILTER,
                    1,
                );
                VirtCsr::set_mstatus_field(
                    &mut self.csr.mstatus,
                    mstatus::MIE_OFFSET,
                    mstatus::MIE_FILTER,
                    mpie,
                );
                VirtCsr::set_mstatus_field(
                    &mut self.csr.mstatus,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    0,
                );

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
        self.csr.mstatus = self.trap_info.mstatus;
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
            MCause::EcallFromUMode if self.get(Register::X17) == abi::MIRAGE_EID => {
                let fid = self.get(Register::X16);
                match fid {
                    abi::MIRAGE_FAILURE_FID => {
                        log::error!("Payload panicked!");
                        log::error!("  pc:    0x{:x}", self.pc);
                        log::error!("  exits: {}", self.nb_exits);
                        unsafe { debug::log_stack_usage() };
                        Plat::exit_failure();
                    }
                    abi::MIRAGE_SUCCESS_FID => {
                        log::info!("Success!");
                        log::info!("Number of payload exits: {}", self.nb_exits);
                        unsafe { debug::log_stack_usage() };
                        Plat::exit_success();
                    }
                    abi::MIRAGE_LOG_FID => {
                        let log_level = self.get(Register::X10);
                        let addr = self.get(Register::X11);
                        let size = self.get(Register::X12);

                        // TODO: add proper validation that this memory range belongs to the
                        // payload
                        let bytes = unsafe { core::slice::from_raw_parts(addr as *const u8, size) };
                        let message = match core::str::from_utf8(bytes) {
                            Ok(msg) => msg,
                            Err(_) => "note: invalid message, not utf-8",
                        };
                        match log_level {
                            abi::log::MIRAGE_ERROR => log::error!("> {}", message),
                            abi::log::MIRAGE_WARN => log::warn!("> {}", message),
                            abi::log::MIRAGE_INFO => log::info!("> {}", message),
                            abi::log::MIRAGE_DEBUG => log::debug!("> {}", message),
                            abi::log::MIRAGE_TRACE => log::trace!("> {}", message),
                            _ => {
                                log::info!("Mirage log SBI call with invalid level: {}", log_level)
                            }
                        }

                        // For now we don't return error code or the lenght written
                        self.set(Register::X10, 0);
                        self.set(Register::X11, 0);
                        self.pc += 4;
                    }
                    _ => panic!("Invalid Mirage FID: 0x{:x}", fid),
                }
            }
            MCause::EcallFromUMode => {
                todo!("ecall is not yet supported for EID other than Mirage ABI");
            }
            MCause::EcallFromSMode => {
                todo!("ecall from smode is not yet supported")
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
            Csr::Mstatus => self.csr.mstatus & mstatus::MSTATUS_FILTER,
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
            Csr::Medeleg => self.csr.medeleg,
            Csr::Mideleg => self.csr.mideleg,
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
            Csr::Tselect => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION : read debug-mode specification
            Csr::Mepc => self.csr.mepc,
            Csr::Mcause => self.csr.mcause,
            Csr::Mtval => self.csr.mtval,
            //Supervisor-level CSRs
            Csr::Sstatus => {
                let mstatus: usize = self.get(Csr::Mstatus);
                return mstatus & mstatus::SSTATUS_FILTER;
            }
            Csr::Sie => self.csr.sie,
            Csr::Stvec => self.csr.stvec,
            Csr::Scounteren => self.csr.scounteren,
            Csr::Senvcfg => self.csr.senvcfg,
            Csr::Sscratch => self.csr.sscratch,
            Csr::Sepc => self.csr.sepc,
            Csr::Scause => self.csr.scause,
            Csr::Stval => self.csr.stval,
            Csr::Sip => self.csr.sip,
            Csr::Satp => self.csr.satp,
            Csr::Scontext => self.csr.scontext,
            // Unknown
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
        }
    }

    fn set(&mut self, register: Csr, value: usize) {
        match register {
            Csr::Mhartid => (), // Read-only
            Csr::Mstatus => {
                // TODO: create some constant values
                let mut new_value = value & mstatus::MSTATUS_FILTER; //self.csr.mstatus;
                                                                     // MPP : 11 : write legal : 0,1,3
                let mpp = (value >> 11) & 0b11;
                VirtCsr::set_mstatus_field(
                    &mut new_value,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    if mpp == 0 || mpp == 1 || mpp == 3 {
                        mpp
                    } else {
                        0
                    },
                );
                // SXL : 34 : read-only : MX-LEN = 64
                let mxl: usize = 2;
                VirtCsr::set_mstatus_field(
                    &mut new_value,
                    mstatus::SXL_OFFSET,
                    mstatus::SXL_FILTER,
                    if Plat::HAS_S_MODE { mxl } else { 0 },
                );
                // UXL : 32 : read-only : MX-LEN = 64
                VirtCsr::set_mstatus_field(
                    &mut new_value,
                    mstatus::UXL_OFFSET,
                    mstatus::UXL_FILTER,
                    mxl,
                );

                // MPRV : 17 : write anything
                // MBE : 37 : write anything
                let mbe: usize = (self.csr.mstatus >> mstatus::MBE_OFFSET) & mstatus::MBE_FILTER;
                // SBE : 36 : equals MBE
                VirtCsr::set_mstatus_field(
                    &mut new_value,
                    mstatus::SBE_OFFSET,
                    mstatus::SBE_FILTER,
                    if Plat::HAS_S_MODE { mbe } else { 0 },
                );
                // UBE : 6 : equals MBE
                VirtCsr::set_mstatus_field(
                    &mut new_value,
                    mstatus::UBE_OFFSET,
                    mstatus::UBE_FILTER,
                    mbe,
                );

                // TVM : 20 : read-only 0 (NO S-MODE)
                new_value = new_value & !(0b1 << 20); // clear TVM
                if !Plat::HAS_S_MODE {
                    VirtCsr::set_mstatus_field(
                        &mut new_value,
                        mstatus::TVM_OFFSET,
                        mstatus::TVM_FILTER,
                        0,
                    );
                }
                // TW : 21 : write anything
                // TSR : 22 : read-only 0 (NO S-MODE)
                if !Plat::HAS_S_MODE {
                    VirtCsr::set_mstatus_field(
                        &mut new_value,
                        mstatus::TSR_OFFSET,
                        mstatus::TSR_FILTER,
                        0,
                    );
                }
                // FS : 13 : read-only 0 (NO S-MODE, F extension)
                if !Plat::HAS_S_MODE {
                    VirtCsr::set_mstatus_field(
                        &mut new_value,
                        mstatus::FS_OFFSET,
                        mstatus::FS_FILTER,
                        0,
                    );
                }
                // VS : 9 : read-only 0 (v registers)
                VirtCsr::set_mstatus_field(
                    &mut new_value,
                    mstatus::VS_OFFSET,
                    mstatus::VS_FILTER,
                    0,
                );
                // XS : 15 : read-only 0 (NO FS nor VS)
                VirtCsr::set_mstatus_field(
                    &mut new_value,
                    mstatus::XS_OFFSET,
                    mstatus::XS_FILTER,
                    0,
                );
                // SD : 63 : read-only 0 (if NO FS/VS/XS)
                VirtCsr::set_mstatus_field(
                    &mut new_value,
                    mstatus::SD_OFFSET,
                    mstatus::SD_FILTER,
                    if Plat::HAS_S_MODE {
                        let fs: usize = (value >> mstatus::FS_OFFSET) & mstatus::FS_FILTER;
                        if fs != 0 {
                            0b1
                        } else {
                            0b0
                        }
                    } else {
                        0
                    },
                );

                self.csr.mstatus = new_value;
            }
            Csr::Misa => {
                // misa shows the extensions available : we cannot have more than possible in hardware
                let arch_misa: usize = Arch::read_misa();
                // Update misa to a legal value
                self.csr.misa =
                    (value & arch_misa & mstatus::MISA_CHANGE_FILTER & !misa::DISABLED) | misa::MXL;
            }
            Csr::Mie => (), // Read-only 0 : interrupts are not yet supported : self.csr.mie = value,
            Csr::Mip => {
                // Only reset possible : interrupts are not yet supported
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
            Csr::Medeleg => (),        // Read-only 0 : do not delegate exceptions
            Csr::Mideleg => (),        // Read-only 0 : do not delegate interrupts
            Csr::Mtinst => todo!(), // TODO : Can only be written automatically by the hardware on a trap, this register should not exist in a system without hypervisor extension
            Csr::Mtval2 => todo!(), // TODO : Must be able to hold 0 and may hold an arbitrary number of 2-bit-shifted guest physical addresses, written alongside mtval, this register should not exist in a system without hypervisor extension
            Csr::Tselect => todo!(), // Read-only 0 when no triggers are implemented
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
            Csr::Mtval => self.csr.mtval = value, // TODO : PLATFORM DEPENDANCE (if trapping writes to mtval or not) : Mtval is read-only 0 for now : must be able to contain valid address and zero
            //Supervisor-level CSRs
            Csr::Sstatus => {
                self.set(Csr::Mstatus, value & mstatus::SSTATUS_FILTER);
            }
            Csr::Sie => self.csr.sie = value,
            Csr::Stvec => self.csr.stvec = value,
            Csr::Scounteren => (), // Read-only 0
            Csr::Senvcfg => self.csr.senvcfg = value,
            Csr::Sscratch => self.csr.sscratch = value,
            Csr::Sepc => {
                if value > Plat::get_max_valid_address() {
                    return;
                }
                self.csr.sepc = value
            }
            Csr::Scause => {
                let cause = MCause::new(value);
                if cause.is_interrupt() {
                    // TODO : does not support interrupts
                    return;
                }
                match cause {
                    // Can only contain supported exception codes
                    MCause::UnknownException => (),
                    _ => self.csr.scause = value,
                }
            }
            Csr::Stval => self.csr.stval = value,
            Csr::Sip => {
                // TODO: handle sip emulation properly
                if value != 0 {
                    // We only support resetting sip for now
                    panic!("sip emulation is not yet implemented");
                }
                self.csr.sip = value;
            }
            Csr::Satp => {
                self.csr.satp = value & mstatus::SATP_CHANGE_FILTER;
            }
            Csr::Scontext => todo!("No information in the specification"),
            // Unknown
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
