//! Firmware Virtualisation
use core::ptr::{read_volatile, write_volatile};

use mirage_core::abi;

use crate::arch::mstatus::{MPP_FILTER, MPP_OFFSET};
use crate::arch::{
    mie, misa, mstatus, parse_mpp_return_mode, satp, Arch, Architecture, Csr, HardwareCapability,
    MCause, Mode, Register, TrapInfo,
};
use crate::decoder::{decode, Instr};
use crate::device::{Device, DeviceAccess};
use crate::host::MirageContext;
use crate::platform::{Plat, Platform};
use crate::{debug, device, utils};

/// The execution mode, either virtualized firmware or native payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Virtualized virmware, running in U-mode.
    Firmware,
    /// Native payload, running in U or S-mode.
    Payload,
}

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
    pub(crate) csr: VirtCsr,
    /// Current privilege mode
    pub(crate) mode: Mode,
    /// Number of virtual PMPs
    pub(crate) nb_pmp: usize,
    /// Hart ID
    pub(crate) hart_id: usize,
    /// Number of exists to Mirage
    pub(crate) nb_exits: usize,
}

impl VirtContext {
    pub fn new(hart_id: usize, nb_pmp: usize) -> Self {
        assert!(nb_pmp <= 64, "Too many PMP registers");

        VirtContext {
            host_stack: 0,
            regs: Default::default(),
            csr: Default::default(),
            pc: 0,
            mode: Mode::M,
            trap_info: Default::default(),
            nb_exits: 0,
            hart_id,
            nb_pmp,
        }
    }
}

/// Control and Status Registers (CSR) for a virtual firmware.
#[derive(Debug)]
#[repr(C)]
pub struct VirtCsr {
    pub misa: usize,
    pub mie: usize,
    pub mip: usize,
    pub mtvec: usize,
    pub mvendorid: usize,
    pub marchid: usize,
    pub mimpid: usize,
    pub mcycle: usize,
    pub minstret: usize,
    pub mscratch: usize,
    pub mcountinhibit: usize,
    pub mcounteren: usize,
    pub menvcfg: usize,
    pub mseccfg: usize,
    pub mcause: usize,
    pub mepc: usize,
    pub mtval: usize,
    pub mstatus: usize,
    pub mtinst: usize,
    pub mconfigptr: usize,
    pub stvec: usize,
    pub scounteren: usize,
    pub senvcfg: usize,
    pub sscratch: usize,
    pub sepc: usize,
    pub scause: usize,
    pub stval: usize,
    pub satp: usize,
    pub scontext: usize,
    pub medeleg: usize,
    pub mideleg: usize,
    pub pmpcfg: [usize; 8],
    pub pmpaddr: [usize; 64],
    pub mhpmcounter: [usize; 29],
    pub mhpmevent: [usize; 29],
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
            mcycle: 0,
            minstret: 0,
            mcountinhibit: 0,
            mcounteren: 0,
            menvcfg: 0,
            mseccfg: 0,
            mcause: 0,
            mepc: 0,
            mtval: 0,
            mstatus: 0,
            mtinst: 0,
            mconfigptr: 0,
            stvec: 0,
            scounteren: 0,
            senvcfg: 0,
            sscratch: 0,
            sepc: 0,
            scause: 0,
            stval: 0,
            satp: 0,
            scontext: 0,
            medeleg: 0,
            mideleg: 0,
            pmpcfg: [0; 8],
            pmpaddr: [0; 64],
            mhpmcounter: [0; 29],
            mhpmevent: [0; 29],
        }
    }
}

impl VirtCsr {
    pub fn set_csr_field(csr: &mut usize, offset: usize, filter: usize, value: usize) {
        debug_assert_eq!((value << offset) & !filter, 0, "Invalid set_csr_field");
        // Clear field
        *csr &= !filter;
        // Set field
        *csr |= value << offset;
    }

    /// Returns the mask of valid bit for the given PMP configuration register.
    pub fn get_pmp_cfg_filter(pmp_csr_idx: usize, nbr_valid_pmps: usize) -> usize {
        if pmp_csr_idx == nbr_valid_pmps / 8 {
            // We are in the correct csr to filter out
            let to_filter_out: usize = ((nbr_valid_pmps / 8) + 1) * 8 - nbr_valid_pmps;

            return !0b0 >> (to_filter_out * 8);
        }
        !0b0
    }
}

// —————————————————————————— Handle Payload Traps —————————————————————————— //

impl VirtContext {
    fn emulate_privileged_instr(&mut self, instr: &Instr, hw: &HardwareCapability) {
        match instr {
            Instr::Wfi => {
                // NOTE: for now there is no safeguard which guarantees that we will eventually get
                // an interrupt, so the firmware might be able to put the core in perpetual sleep
                // state.

                // Set mie to csr.mie, even if mstatus.MIE bit is cleared.
                unsafe {
                    Arch::write_mie(self.csr.mie);
                }

                Arch::wfi();
                self.pc += 4;
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
                self.set_csr(csr, self.get(rs1), hw);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrs { csr, rd, rs1 } => {
                let tmp = self.get(csr);
                self.set_csr(csr, tmp | self.get(rs1), hw);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrwi { csr, rd, uimm } => {
                self.set(rd, self.get(csr));
                self.set_csr(csr, *uimm, hw);
                self.pc += 4;
            }
            Instr::Csrrsi { csr, rd, uimm } => {
                let tmp = self.get(csr);
                self.set_csr(csr, tmp | uimm, hw);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrc { csr, rd, rs1 } => {
                let tmp = self.get(csr);
                self.set_csr(csr, tmp & !self.get(rs1), hw);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrci { csr, rd, uimm } => {
                let tmp = self.get(csr);
                self.set_csr(csr, tmp & !uimm, hw);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Mret => {
                match parse_mpp_return_mode(self.csr.mstatus) {
                    Mode::M => {
                        log::trace!("mret to m-mode");
                        // Mret is jumping back to machine mode, do nothing
                    }
                    Mode::S if Plat::HAS_S_MODE => {
                        log::trace!("mret to s-mode with MPP");
                        // Mret is jumping to supervisor mode, the runner is the guest OS
                        self.mode = Mode::S;

                        VirtCsr::set_csr_field(
                            &mut self.csr.mstatus,
                            mstatus::MPRV_OFFSET,
                            mstatus::MPRV_FILTER,
                            0,
                        );
                    }
                    Mode::U => {
                        log::trace!("mret to u-mode with MPP");
                        // Mret is jumping to user mode, the runner is the guest OS
                        self.mode = Mode::U;

                        VirtCsr::set_csr_field(
                            &mut self.csr.mstatus,
                            mstatus::MPRV_OFFSET,
                            mstatus::MPRV_FILTER,
                            0,
                        );
                    }
                    _ => {
                        panic!(
                            "MRET is not going to M/S/U mode: {} with MPP {:x}",
                            self.csr.mstatus,
                            (self.csr.mstatus & mstatus::MPP_FILTER) >> mstatus::MPP_OFFSET
                        );
                    }
                }
                // Modify mstatus
                // ONLY WITH HYPERVISOR EXTENSION : MPV = 0,
                if false {
                    VirtCsr::set_csr_field(
                        &mut self.csr.mstatus,
                        mstatus::MPV_OFFSET,
                        mstatus::MPV_FILTER,
                        0,
                    );
                }

                // MIE = MPIE, MPIE = 1, MPRV = 0
                let mpie = (self.csr.mstatus & mstatus::MPIE_FILTER) >> mstatus::MPIE_OFFSET;

                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MPIE_OFFSET,
                    mstatus::MPIE_FILTER,
                    1,
                );
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MIE_OFFSET,
                    mstatus::MIE_FILTER,
                    mpie,
                );
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    0,
                );

                // Jump back to firmware
                self.pc = self.csr.mepc;
            }
            Instr::Vfencevma => unsafe {
                Arch::sfence_vma();
                self.pc += 4;
            },
            _ => todo!("Instruction not yet implemented: {:?}", instr),
        }
    }

    pub fn handle_device_access_fault(&mut self, instr: &Instr, device: &Device) {
        match instr {
            Instr::Ld { rd, rs1, imm } => {
                let offset = utils::calculate_offset(self.get(rs1), *imm);
                if let Some(device_interface) = &device.device_interface {
                    match device_interface.lock().read_device(offset) {
                        Ok(value) => {
                            self.set(rd, value);
                            self.pc += 4;
                        }
                        Err(err) => {
                            panic!("Error reading {}: {}", device.name, err);
                        }
                    }
                } else {
                    panic!("No device interface for {}", device.name);
                }
            }
            Instr::CSd { rs1, rs2, imm } => {
                let offset = self.get(rs1) + imm * 8;
                let data = self.get(rs2);
                if let Some(device_interface) = &device.device_interface {
                    match device_interface.lock().write_device(offset, data) {
                        Ok(()) => {
                            self.set(rs1, data);
                            self.pc += 2;
                        }
                        Err(err) => {
                            panic!("Error writing {}: {}", device.name, err);
                        }
                    }
                } else {
                    panic!("No device interface for {}", device.name);
                }
            }
            Instr::CSw { rs1, rs2, imm } => {
                let offset = self.get(rs1) + imm * 4;
                let data = self.get(rs2);
                if let Some(device_interface) = &device.device_interface {
                    match device_interface.lock().write_device(offset, data) {
                        Ok(()) => {
                            self.set(rs1, data);
                            self.pc += 2;
                        }
                        Err(err) => {
                            panic!("Error writing {}: {}", device.name, err);
                        }
                    }
                } else {
                    panic!("No device interface for {}", device.name);
                }
            }
            _ => todo!("Instruction not yet implemented: {:?}", instr),
        }
    }

    pub fn emulate_jump_trap_handler(&mut self) {
        // We are now emulating a trap, registers need to be updated
        log::trace!("Emulating jump to trap handler");
        self.csr.mcause = self.trap_info.mcause;
        self.csr.mstatus = self.trap_info.mstatus;
        self.csr.mtval = self.trap_info.mtval;
        self.csr.mip = self.trap_info.mip;
        self.csr.mepc = self.trap_info.mepc;

        match self.mode {
            Mode::M => {
                // Modify mstatus: previous privilege mode is machine = 3
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    Mode::M.to_bits(),
                );
            }
            _ => {
                // No need to modify mstatus: MPP is correct
                self.mode = Mode::M;
            }
        }

        // Go to firmware trap handler
        assert!(
            self.csr.mtvec & 0b11 == 0,
            "Only direct mode is supported for mtvec"
        );
        self.pc = self.csr.mtvec
    }

    /// Handle the trap coming from the firmware
    pub fn handle_firmware_trap(&mut self, mctx: &MirageContext) {
        let hw = &mctx.hw;

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
                        let message = core::str::from_utf8(bytes)
                            .unwrap_or("note: invalid message, not utf-8");
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
                self.emulate_privileged_instr(&instr, hw);
            }
            MCause::Breakpoint => {
                self.emulate_jump_trap_handler();
            }
            MCause::StoreAccessFault | MCause::LoadAccessFault | MCause::InstrAccessFault => {
                // PMP faults
                if let Some(device) =
                    device::find_matching_device(self.trap_info.mtval, &mctx.devices)
                {
                    let instr = unsafe { Arch::get_raw_faulting_instr(&self.trap_info) };

                    let instr = decode(instr);

                    log::trace!("Accessed device: {} | With instr: {:?}", device.name, instr);

                    self.handle_device_access_fault(&instr, &device);
                } else {
                    log::trace!("No matching device found for address: {}", self.csr.mtval);

                    self.emulate_jump_trap_handler();
                }
            }
            MCause::MachineTimerInt => {
                // Set mtimecmp > mtime to clear mip.mtip
                const MTIMECMP: *mut u64 = 0x2004000 as *mut u64;
                const MTIME: *mut u64 = 0x200BFF8 as *mut u64;
                unsafe {
                    let mtime = read_volatile(MTIME);
                    write_volatile(MTIMECMP, mtime + 1000_000); // TODO : what value ?
                }

                self.emulate_jump_trap_handler();
            }
            _ => {
                if cause.is_interrupt() {
                    // TODO : For now, only care for MTIP bit
                    todo!("Other interrupt are not yet implemented {:?}", cause);
                } else {
                    // TODO : Need to match other traps
                    todo!("Other traps are not yet implemented");
                }
            }
        }
    }
}

// ———————————————————————— Register Setters/Getters ———————————————————————— //

/// A module exposing the traits to manipulate registers of a virtual context.
///
/// To get and set registers from a virtual context, first import all the traits:
///
/// ```
/// use crate::virt::traits::*;
/// ```
pub mod traits {
    pub use super::{HwRegisterContextSetter, RegisterContextGetter, RegisterContextSetter};
}

/// A trait implemented by virtual contexts to read registers.
pub trait RegisterContextGetter<R> {
    fn get(&self, register: R) -> usize;
}

/// A trait implemented by virtual contexts to write registers.
pub trait RegisterContextSetter<R> {
    fn set(&mut self, register: R, value: usize);
}

/// A trait implemented by virtual contexts to write registers whose value depends on
/// hardware capabilities..
pub trait HwRegisterContextSetter<R> {
    fn set_csr(&mut self, register: R, value: usize, hw: &HardwareCapability);
}

impl RegisterContextGetter<Register> for VirtContext {
    fn get(&self, register: Register) -> usize {
        // NOTE: Register x0 is never set, so always keeps a value of 0
        self.regs[register as usize]
    }
}

impl RegisterContextSetter<Register> for VirtContext {
    fn set(&mut self, register: Register, value: usize) {
        // Skip register x0
        if register == Register::X0 {
            return;
        }
        self.regs[register as usize] = value;
    }
}

impl RegisterContextGetter<Csr> for VirtContext {
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
                if pmp_cfg_idx >= self.nb_pmp / 8 {
                    // This PMP is not emulated
                    return 0;
                }
                self.csr.pmpcfg[pmp_cfg_idx / 2]
                    & VirtCsr::get_pmp_cfg_filter(pmp_cfg_idx, self.nb_pmp)
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= self.nb_pmp {
                    // This PMP is not emulated
                    return 0;
                }
                self.csr.pmpaddr[pmp_addr_idx]
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
            Csr::Sstatus => self.get(Csr::Mstatus) & mstatus::SSTATUS_FILTER,
            Csr::Sie => self.get(Csr::Mie) & mie::SIE_FILTER,
            Csr::Stvec => self.csr.stvec,
            Csr::Scounteren => self.csr.scounteren,
            Csr::Senvcfg => self.csr.senvcfg,
            Csr::Sscratch => self.csr.sscratch,
            Csr::Sepc => self.csr.sepc,
            Csr::Scause => self.csr.scause,
            Csr::Stval => self.csr.stval,
            Csr::Sip => self.get(Csr::Mip) & mie::SIE_FILTER,
            Csr::Satp => self.csr.satp,
            Csr::Scontext => self.csr.scontext,
            // Unknown
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
        }
    }
}

impl HwRegisterContextSetter<Csr> for VirtContext {
    fn set_csr(&mut self, register: Csr, value: usize, hw: &HardwareCapability) {
        match register {
            Csr::Mhartid => (), // Read-only
            Csr::Mstatus => {
                // TODO: create some constant values
                let mut new_value = value & mstatus::MSTATUS_FILTER; //self.csr.mstatus;
                                                                     // MPP : 11 : write legal : 0,1,3
                let mpp = (value & MPP_FILTER) >> MPP_OFFSET;
                VirtCsr::set_csr_field(
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
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::SXL_OFFSET,
                    mstatus::SXL_FILTER,
                    if Plat::HAS_S_MODE { mxl } else { 0 },
                );
                // UXL : 32 : read-only : MX-LEN = 64
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::UXL_OFFSET,
                    mstatus::UXL_FILTER,
                    mxl,
                );

                // MPRV : 17 : write anything
                // MBE : 37 : write anything
                let mbe: usize = (self.csr.mstatus & mstatus::MBE_FILTER) >> mstatus::MBE_OFFSET;
                // SBE : 36 : equals MBE
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::SBE_OFFSET,
                    mstatus::SBE_FILTER,
                    if Plat::HAS_S_MODE { mbe } else { 0 },
                );
                // UBE : 6 : equals MBE
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::UBE_OFFSET,
                    mstatus::UBE_FILTER,
                    mbe,
                );

                // TVM : 20 : read-only 0 (NO S-MODE)
                new_value &= !(0b1 << 20); // clear TVM
                if !Plat::HAS_S_MODE {
                    VirtCsr::set_csr_field(
                        &mut new_value,
                        mstatus::TVM_OFFSET,
                        mstatus::TVM_FILTER,
                        0,
                    );
                }
                // TW : 21 : write anything
                // TSR : 22 : read-only 0 (NO S-MODE)
                if !Plat::HAS_S_MODE {
                    VirtCsr::set_csr_field(
                        &mut new_value,
                        mstatus::TSR_OFFSET,
                        mstatus::TSR_FILTER,
                        0,
                    );
                }
                // FS : 13 : read-only 0 (NO S-MODE, F extension)
                if !Plat::HAS_S_MODE {
                    VirtCsr::set_csr_field(
                        &mut new_value,
                        mstatus::FS_OFFSET,
                        mstatus::FS_FILTER,
                        0,
                    );
                }
                // VS : 9 : read-only 0 (v registers)
                VirtCsr::set_csr_field(&mut new_value, mstatus::VS_OFFSET, mstatus::VS_FILTER, 0);
                // XS : 15 : read-only 0 (NO FS nor VS)
                VirtCsr::set_csr_field(&mut new_value, mstatus::XS_OFFSET, mstatus::XS_FILTER, 0);
                // SD : 63 : read-only 0 (if NO FS/VS/XS)
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::SD_OFFSET,
                    mstatus::SD_FILTER,
                    if Plat::HAS_S_MODE {
                        let fs: usize = (value & mstatus::FS_FILTER) >> mstatus::FS_OFFSET;
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
                    (value & arch_misa & misa::MISA_CHANGE_FILTER & !misa::DISABLED) | misa::MXL;
            }
            Csr::Mie => self.csr.mie = value & hw.interrupts,
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
                let mut value = value;
                if Csr::PMP_CFG_LOCK_MASK & value != 0 {
                    debug::warn_once!("PMP lock bits are not yet supported");
                    value &= !Csr::PMP_CFG_LOCK_MASK;
                }
                if pmp_cfg_idx % 2 == 1 {
                    // Illegal because we are in a RISCV64 setting
                    panic!("Illegal PMP_CFG {:?}", register)
                } else if pmp_cfg_idx >= self.nb_pmp / 8 {
                    // This PMP is not emulated, ignore changes
                    return;
                }
                self.csr.pmpcfg[pmp_cfg_idx / 2] = Csr::PMP_CFG_LEGAL_MASK
                    & value
                    & VirtCsr::get_pmp_cfg_filter(pmp_cfg_idx, self.nb_pmp);
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= self.nb_pmp {
                    // This PMP is not emulated, ignore
                    return;
                }
                self.csr.pmpaddr[pmp_addr_idx] = Csr::PMP_ADDR_LEGAL_MASK & value;
            }
            Csr::Mcycle => (),                                      // Read-only 0
            Csr::Minstret => (),                                    // Read-only 0
            Csr::Mhpmcounter(_counter_idx) => (),                   // Read-only 0
            Csr::Mcountinhibit => (),                               // Read-only 0
            Csr::Mhpmevent(_event_idx) => (),                       // Read-only 0
            Csr::Mcounteren => self.csr.mcounteren = value & 0b111, // Only show IR, TM and CY (for cycle, time and instret counters)
            Csr::Menvcgf => self.csr.menvcfg = value,
            Csr::Mseccfg => self.csr.mseccfg = value,
            Csr::Mconfigptr => (),                    // Read-only
            Csr::Medeleg => self.csr.medeleg = value, //TODO : some values need to be read-only 0
            Csr::Mideleg => self.csr.mideleg = value & hw.interrupts,
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
                // Clear sstatus bits
                let mstatus = self.get(Csr::Mstatus) & !mstatus::SSTATUS_FILTER;
                // Set sstatus bits to new value
                self.set_csr(
                    Csr::Mstatus,
                    mstatus | (value & mstatus::SSTATUS_FILTER),
                    hw,
                );
            }
            Csr::Sie => {
                // Clear S bits
                let mie = self.get(Csr::Mie) & !mie::SIE_FILTER;
                // Set S bits to new value
                self.set_csr(Csr::Mie, mie | (value & mie::SIE_FILTER), hw);
            }
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
                // Clear S bits
                let mip = self.get(Csr::Mip) & !mie::SIE_FILTER;
                // Set S bits to new value
                self.set_csr(Csr::Mip, mip | (value & mie::SIE_FILTER), hw);
            }
            Csr::Satp => {
                self.csr.satp = value & satp::SATP_CHANGE_FILTER;
            }
            Csr::Scontext => todo!("No information in the specification"),
            // Unknown
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
        }
    }
}

/// Forward RegisterContextGetter implementation for register references
impl<'a, R> RegisterContextGetter<&'a R> for VirtContext
where
    R: Copy,
    VirtContext: RegisterContextGetter<R>,
{
    #[inline]
    fn get(&self, register: &'a R) -> usize {
        self.get(*register)
    }
}

/// Forward RegisterContextSetter implementation for register references
impl<'a, R> RegisterContextSetter<&'a R> for VirtContext
where
    R: Copy,
    VirtContext: RegisterContextSetter<R>,
{
    #[inline]
    fn set(&mut self, register: &'a R, value: usize) {
        self.set(*register, value)
    }
}

/// Forward HwCsrRegisterContextSetter implementation for register references
impl<'a, R> HwRegisterContextSetter<&'a R> for VirtContext
where
    R: Copy,
    VirtContext: HwRegisterContextSetter<R>,
{
    #[inline]
    fn set_csr(&mut self, register: &'a R, value: usize, hw: &HardwareCapability) {
        self.set_csr(*register, value, hw)
    }
}
