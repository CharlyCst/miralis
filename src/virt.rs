//! Firmware Virtualisation
use core::usize;

use miralis_core::abi;

use crate::arch::pmp::pmpcfg;
use crate::arch::{
    mie, misa, mstatus, parse_mpp_return_mode, satp, Arch, Architecture, Csr, HardwareCapability,
    MCause, Mode, Register, TrapInfo,
};
use crate::benchmark::Benchmark;
use crate::decoder::{decode, Instr};
use crate::device::VirtDevice;
use crate::host::MiralisContext;
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
    /// Number of exists to Miralis
    pub(crate) nb_exits: usize,
}

impl VirtContext {
    pub const fn new(hart_id: usize, nb_pmp: usize) -> Self {
        assert!(nb_pmp <= 64, "Too many PMP registers");

        VirtContext {
            host_stack: 0,
            regs: [0; 32],
            csr: VirtCsr {
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
            },
            pc: 0,
            mode: Mode::M,
            trap_info: TrapInfo {
                mepc: 0,
                mstatus: 0,
                mcause: 0,
                mip: 0,
                mtval: 0,
            },
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

impl VirtContext {
    fn emulate_privileged_instr(&mut self, instr: &Instr, hw: &HardwareCapability) {
        match instr {
            Instr::Wfi => {
                // NOTE: for now there is no safeguard which guarantees that we will eventually get
                // an interrupt, so the firmware might be able to put the core in perpetual sleep
                // state.

                // Set mie to csr.mie, even if mstatus.MIE bit is cleared.
                unsafe {
                    Arch::write_csr(Csr::Mie, self.csr.mie);
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

    /// Handles a compressed load instruction.
    ///
    /// Calculates the memory address and reads a zero-extended value
    /// from the device (after applying a mask to prevent memory leak).
    /// - Compressed load&store instructions are 2 bytes long.
    /// - The immediate (`imm`) value is always positive and has a multiplier applied.
    fn handle_compressed_load(
        &mut self,
        device: &VirtDevice,
        rd: Register,
        rs1: Register,
        imm: usize,
        length: usize,
    ) {
        let multiplier = length / 8;
        let address = self.get(rs1) + imm * multiplier;
        let offset = address - device.start_addr;
        match device.device_interface.read_device(offset, length.into()) {
            Ok(mut value) => {
                let mask = if length < usize::BITS as usize {
                    (1 << length) - 1
                } else {
                    usize::MAX
                };
                value &= mask;

                self.set(rd, value);
                self.pc += 2;
            }
            Err(err) => panic!("Error reading {}: {}", device.name, err),
        }
    }

    /// Handles a compressed store instruction.
    ///
    /// Calculates the memory address and writes the value
    /// to the device (after applying a mask to prevent overflow).
    fn handle_compressed_store(
        &mut self,
        device: &VirtDevice,
        rs1: Register,
        rs2: Register,
        imm: usize,
        length: usize,
    ) {
        let multiplier = length / 8;
        let address = self.get(rs1) + imm * multiplier;
        let offset = address - device.start_addr;
        let mut value = self.get(rs2);

        let mask = if length < usize::BITS as usize {
            (1 << length) - 1
        } else {
            usize::MAX
        };

        if value > mask {
            log::warn!(
                "Value {} exceeds allowed length {}. Trimming to fit.",
                value,
                length
            );
            value &= mask;
        }
        match device
            .device_interface
            .write_device(offset, length.into(), value)
        {
            Ok(()) => {
                self.pc += 2;
            }
            Err(err) => panic!("Error writing {}: {}", device.name, err),
        }
    }

    /// Handles a load instruction.
    ///
    /// Calculates the memory address, reads the value from the device,
    /// sign-extends it to 32 bits if necessary (for 8 and 16-bit instructions),
    /// applies a mask and writes the value to the device.
    ///
    /// - Normal load instructions are 4 bytes long.
    /// - The immediate (`imm`) value can be positive or negative, but it has no multiplier.
    fn handle_load(
        &mut self,
        device: &VirtDevice,
        rd: Register,
        rs1: Register,
        imm: isize,
        length: usize,
    ) {
        let address = utils::calculate_addr(self.get(rs1), imm);
        let offset = address - device.start_addr;
        match device.device_interface.read_device(offset, length.into()) {
            Ok(mut value) => {
                // Sign-extend to 32 bits
                if length < 32 && value < (1 << 31) {
                    value = (value as i32) as usize;
                }
                let mask = if length < usize::BITS as usize {
                    (1 << length) - 1
                } else {
                    usize::MAX
                };

                value &= mask;

                self.set(rd, value);
                self.pc += 4;
            }
            Err(err) => panic!("Error reading {}: {}", device.name, err),
        }
    }

    /// Handles a store instruction.
    ///
    /// Calculates the memory address and writes the value
    /// to the device (after applying a mask to prevent overflow).
    fn handle_store(
        &mut self,
        device: &VirtDevice,
        rs1: Register,
        rs2: Register,
        imm: isize,
        length: usize,
    ) {
        let address = utils::calculate_addr(self.get(rs1), imm);
        let offset = address - device.start_addr;
        let mut value = self.get(rs2);

        let mask = if length < usize::BITS as usize {
            (1 << length) - 1
        } else {
            usize::MAX
        };

        if value > mask {
            log::warn!(
                "Value {} exceeds allowed length {}. Trimming to fit.",
                value,
                length
            );
            value &= mask;
        }

        match device
            .device_interface
            .write_device(offset, length.into(), value)
        {
            Ok(()) => {
                self.pc += 4;
            }
            Err(err) => panic!("Error writing {}: {}", device.name, err),
        }
    }

    pub fn handle_device_access_fault(&mut self, instr: &Instr, device: &VirtDevice) {
        match instr {
            Instr::Ld { rd, rs1, imm } => self.handle_load(device, *rd, *rs1, *imm, 64),
            Instr::Lw { rd, rs1, imm } => self.handle_load(device, *rd, *rs1, *imm, 32),
            Instr::Lh { rd, rs1, imm } => self.handle_load(device, *rd, *rs1, *imm, 16),
            Instr::Lb { rd, rs1, imm } => self.handle_load(device, *rd, *rs1, *imm, 8),
            Instr::Sd { rs1, rs2, imm } => self.handle_store(device, *rs1, *rs2, *imm, 64),
            Instr::Sw { rs1, rs2, imm } => self.handle_store(device, *rs1, *rs2, *imm, 32),
            Instr::Sh { rs1, rs2, imm } => self.handle_store(device, *rs1, *rs2, *imm, 16),
            Instr::Sb { rs1, rs2, imm } => self.handle_store(device, *rs1, *rs2, *imm, 8),
            Instr::CLd { rd, rs1, imm } => self.handle_compressed_load(device, *rd, *rs1, *imm, 64),
            Instr::CLw { rd, rs1, imm } => self.handle_compressed_load(device, *rd, *rs1, *imm, 32),
            Instr::CSd { rs1, rs2, imm } => {
                self.handle_compressed_store(device, *rs1, *rs2, *imm, 64)
            }
            Instr::CSw { rs1, rs2, imm } => {
                self.handle_compressed_store(device, *rs1, *rs2, *imm, 32)
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
        self.csr.mepc = self.trap_info.mepc;

        // Real mip.SEIE bit should not be different from virtual mip.SEIE as it is read-only in S-Mode or U-Mode.
        // But csrr is modified for SEIE and return the logical-OR of SEIE and the interrupt signal from interrupt
        // controller. (refer to documentation for further detail).
        self.csr.mip = self.trap_info.mip & !mie::SEIE_FILTER | self.csr.mip & mie::SEIE_FILTER;

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
    pub fn handle_firmware_trap(&mut self, mctx: &MiralisContext) {
        let hw = &mctx.hw;

        let cause = self.trap_info.get_cause();
        match cause {
            MCause::EcallFromUMode if self.get(Register::X17) == abi::MIRALIS_EID => {
                self.handle_ecall()
            }
            MCause::EcallFromUMode => {
                todo!("ecall is not yet supported for EID other than Miralis ABI");
            }
            MCause::EcallFromSMode => {
                panic!("Firware should not be able to come from S-mode");
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
            MCause::StoreAccessFault | MCause::LoadAccessFault => {
                // PMP faults
                if let Some(device) =
                    device::find_matching_device(self.trap_info.mtval, &mctx.devices)
                {
                    let instr = unsafe { Arch::get_raw_faulting_instr(&self.trap_info) };
                    let instr = decode(instr);
                    log::trace!("Accessed device: {} | With instr: {:?}", device.name, instr);
                    self.handle_device_access_fault(&instr, &device);
                } else {
                    log::trace!("No matching device found for address: {:x}", self.csr.mtval);
                    self.emulate_jump_trap_handler();
                }
            }
            MCause::InstrAccessFault => {
                self.emulate_jump_trap_handler();
            }
            MCause::MachineTimerInt => {
                // Set mtimecmp > mtime to clear mip.mtip
                // We set a deadline as far as possible in the future for now, the firmware can
                // re-write mtimecmp through the virtual CLINT to trigger an interrupt earlier.
                let mut clint = Plat::get_clint().lock();
                clint
                    .write_mtimecmp(mctx.hw.hart, usize::MAX)
                    .expect("Failed to write mtimecmp");
                self.emulate_jump_trap_handler();
            }
            _ => {
                if cause.is_interrupt() {
                    // TODO : For now, only care for MTIP bit
                    todo!(
                        "Other interrupts are not yet implemented {:?} at {:x}",
                        cause,
                        self.trap_info.mepc
                    );
                } else {
                    // TODO : Need to match other traps
                    todo!(
                        "Other traps are not yet implemented {:?} at {:x}",
                        cause,
                        self.trap_info.mepc
                    );
                }
            }
        }
    }

    /// Handle the trap coming from the payload
    pub fn handle_payload_trap(&mut self) {
        let cause = self.trap_info.get_cause();

        // We only care about ecalls.
        match cause {
            MCause::EcallFromSMode if self.get(Register::X17) == abi::MIRALIS_EID => {
                self.handle_ecall()
            }
            _ => self.emulate_jump_trap_handler(),
        }
    }

    /// Ecalls may come from firmware or payload, resulting in different handling.
    fn handle_ecall(&mut self) {
        let fid = self.get(Register::X16);
        match fid {
            abi::MIRALIS_FAILURE_FID if self.mode == Mode::M => {
                log::error!("Payload panicked!");
                log::error!("  pc:    0x{:x}", self.pc);
                log::error!("  exits: {}", self.nb_exits);
                unsafe { debug::log_stack_usage() };
                Plat::exit_failure();
            }
            abi::MIRALIS_SUCCESS_FID if self.mode == Mode::M => {
                log::info!("Success!");
                log::info!("Number of payload exits: {}", self.nb_exits);
                unsafe { debug::log_stack_usage() };
                Plat::exit_success();
            }
            abi::MIRALIS_LOG_FID if self.mode == Mode::M => {
                let log_level = self.get(Register::X10);
                let addr = self.get(Register::X11);
                let size = self.get(Register::X12);

                // TODO: add proper validation that this memory range belongs to the
                // payload
                let bytes = unsafe { core::slice::from_raw_parts(addr as *const u8, size) };
                let message =
                    core::str::from_utf8(bytes).unwrap_or("note: invalid message, not utf-8");
                match log_level {
                    abi::log::MIRALIS_ERROR => log::error!("> {}", message),
                    abi::log::MIRALIS_WARN => log::warn!("> {}", message),
                    abi::log::MIRALIS_INFO => log::info!("> {}", message),
                    abi::log::MIRALIS_DEBUG => log::debug!("> {}", message),
                    abi::log::MIRALIS_TRACE => log::trace!("> {}", message),
                    _ => {
                        log::info!("Miralis log SBI call with invalid level: {}", log_level)
                    }
                }

                // For now we don't return error code or the lenght written
                self.set(Register::X10, 0);
                self.set(Register::X11, 0);
                self.pc += 4;
            }
            abi::MIRALIS_BENCHMARK_FID => {
                Benchmark::record_counters();
                Plat::exit_success();
            }
            _ => panic!("Invalid Miralis FID: 0x{:x}", fid),
        }
    }

    /// Loads the S-mode CSR registers into the physical registers configures M-mode registers for
    /// payload execution.
    pub unsafe fn switch_from_firmware_to_payload(&mut self, mctx: &mut MiralisContext) {
        // First, restore S-mode registers

        Arch::write_csr(Csr::Stvec, self.csr.stvec);
        Arch::write_csr(Csr::Scounteren, self.csr.scounteren);
        Arch::write_csr(Csr::Satp, self.csr.satp);
        Arch::write_csr(Csr::Sscratch, self.csr.sscratch);
        Arch::write_csr(Csr::Sepc, self.csr.sepc);
        Arch::write_csr(Csr::Scause, self.csr.scause);
        Arch::write_csr(Csr::Stval, self.csr.stval);
        Arch::write_csr(Csr::Mcounteren, self.csr.mcounteren);

        if mctx.hw.available_reg.senvcfg {
            Arch::write_csr(Csr::Senvcfg, self.csr.senvcfg);
        }

        if mctx.hw.available_reg.menvcfg {
            Arch::write_csr(Csr::Menvcfg, self.csr.menvcfg);
        }

        // Then configuring M-mode registers
        let mut mstatus = self.csr.mstatus; // We need to set the next mode bits before mret
        VirtCsr::set_csr_field(
            &mut mstatus,
            mstatus::MPP_OFFSET,
            mstatus::MPP_FILTER,
            self.mode.to_bits(),
        );
        Arch::write_csr(Csr::Mstatus, mstatus & !mstatus::MIE_FILTER);
        Arch::write_csr(Csr::Mideleg, self.csr.mideleg);
        Arch::write_csr(Csr::Medeleg, self.csr.medeleg);

        Arch::write_csr(Csr::Mie, self.csr.mie);
        Arch::write_csr(Csr::Mip, self.csr.mip);

        // Load virtual PMP registers into Miralis's own registers
        mctx.pmp.load_with_offset(
            &self.csr.pmpaddr,
            &self.csr.pmpcfg,
            mctx.virt_pmp_offset as usize,
            self.nb_pmp,
        );
        // Deny all addresses by default if at least one PMP is implemented
        if self.nb_pmp > 0 {
            let last_pmp_idx = mctx.pmp.nb_pmp as usize - 1;
            mctx.pmp.set(last_pmp_idx, usize::MAX, pmpcfg::NAPOT);
        }
        // Commit the PMP to hardware
        Arch::write_pmp(&mctx.pmp);
        Arch::sfence_vma();
    }

    /// Loads the S-mode CSR registers into the virtual context and install sensible values (mostly
    /// 0) for running the virtual firmware in U-mode.
    pub unsafe fn switch_from_payload_to_firmware(&mut self, mctx: &mut MiralisContext) {
        // Save the registers into the virtual context.

        self.csr.stvec = Arch::write_csr(Csr::Stvec, 0);
        self.csr.scounteren = Arch::write_csr(Csr::Scounteren, 0);
        self.csr.satp = Arch::write_csr(Csr::Satp, 0);

        self.csr.sscratch = Arch::write_csr(Csr::Sscratch, 0);
        self.csr.sepc = Arch::write_csr(Csr::Sepc, 0);
        self.csr.scause = Arch::write_csr(Csr::Scause, 0);

        self.csr.stval = Arch::write_csr(Csr::Stval, 0);

        if mctx.hw.available_reg.senvcfg {
            self.csr.senvcfg = Arch::write_csr(Csr::Senvcfg, 0);
        }

        if mctx.hw.available_reg.menvcfg {
            self.csr.menvcfg = Arch::write_csr(Csr::Menvcfg, 0);
        }

        self.csr.mcounteren = Arch::write_csr(Csr::Mcounteren, 0);

        // Now save M-mode registers which are (partially) exposed as S-mode registers.
        // For mstatus we read the current value and clear the two MPP bits to jump into U-mode
        // (virtual firmware) during the next mret.

        self.csr.mstatus = self.csr.mstatus & !mstatus::SSTATUS_FILTER
            | Arch::read_csr(Csr::Mstatus) & mstatus::SSTATUS_FILTER;
        Arch::set_mpp(Mode::U);
        Arch::write_csr(Csr::Mideleg, 0); // Do not delegate any interrupts
        Arch::write_csr(Csr::Medeleg, 0); // Do not delegate any exceptions

        self.csr.mie = Arch::read_csr(Csr::Mie);

        // Real mip.SEIE bit should not be different from virtual mip.SEIE as it is read-only in S-Mode or U-Mode.
        // But csrr is modified for SEIE and return the logical-OR of SEIE and the interrupt signal from interrupt
        // controller. (refer to documentation for further detail).
        self.csr.mip =
            Arch::read_csr(Csr::Mip) & (!mie::SEIE_FILTER) | self.csr.mip & mie::SEIE_FILTER;

        // Remove Firmware PMP from the hardware
        mctx.pmp
            .clear_range(mctx.virt_pmp_offset as usize, self.nb_pmp);
        // Allow all addresses by default
        let last_pmp_idx = mctx.pmp.nb_pmp as usize - 1;
        mctx.pmp
            .set(last_pmp_idx, usize::MAX, pmpcfg::RWX | pmpcfg::NAPOT);
        // Commit the PMP to hardware
        Arch::write_pmp(&mctx.pmp);
        Arch::sfence_vma();
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
            Csr::Mip => self.csr.mip | Arch::read_csr(Csr::Mip) & mie::SEIE_FILTER, // Allows to read the interrupt signal from interrupt controller.
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
            Csr::Menvcfg => self.csr.menvcfg,
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
                let mpp = (value & mstatus::MPP_FILTER) >> mstatus::MPP_OFFSET;
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
                let arch_misa: usize = Arch::read_csr(Csr::Misa);
                // Update misa to a legal value
                self.csr.misa =
                    (value & arch_misa & misa::MISA_CHANGE_FILTER & !misa::DISABLED) | misa::MXL;
            }
            Csr::Mie => self.csr.mie = value & hw.interrupts & mie::MIE_WRITE_FILTER,
            Csr::Mip => {
                let value = value & hw.interrupts & mie::MIP_WRITE_FILTER;

                // If the firmware wants to read the mip register after cleaning vmip.SEIP, and we don't sync
                // vmip.SEIP with mip.SEIP, it can't know if there is an interrupt signal from the interrupt
                // controller as the CSR read will be a logical-OR of the signal and mip.SEIP (which is one)
                // so always 1. If vmip.SEIP is 0, CSR read of mip.SEIP should return the interrupt signal.
                // Then, we need to synchronize vmip.SEIP with mip.SEIP.
                if (self.csr.mip ^ value) & mie::SEIE_FILTER != 0 {
                    if value & mie::SEIE_FILTER == 0 {
                        unsafe {
                            Arch::clear_csr_bits(Csr::Mip, mie::SEIE_FILTER);
                        }
                    } else {
                        unsafe {
                            Arch::set_csr_bits(Csr::Mip, mie::SEIE_FILTER);
                        }
                    }
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
            Csr::Menvcfg => self.csr.menvcfg = value,
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

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use core::usize;

    use crate::arch::{mie, mstatus, Arch, Architecture, Csr, Mode};
    use crate::host::MiralisContext;
    use crate::virt::VirtContext;
    use crate::HwRegisterContextSetter;

    /// We test value of mstatus.MPP.
    /// When switching from firmware to payload,
    /// virtual mstatus.MPP must to be S (because we are jumping to payload)
    /// and mstatus.MPP must be M (coming from Miralis).
    ///
    /// When switching from payload to firmware,
    /// virtual mstatus.MPP must to be S (coming from payload)
    /// and mstatus.MPP must be U (going to firmware).
    #[test]
    fn switch_context_mpp() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp);

        ctx.csr.mstatus |= Mode::S.to_bits() << mstatus::MPP_OFFSET;

        unsafe { ctx.switch_from_firmware_to_payload(&mut mctx) }

        assert_eq!(
            ctx.csr.mstatus & mstatus::MPP_FILTER,
            Mode::S.to_bits() << mstatus::MPP_OFFSET,
            "VirtContext Mstatus.MPP must be set to S mode (going to payload)"
        );

        assert_eq!(
            Arch::read_csr(Csr::Mstatus) & mstatus::MPP_FILTER,
            Mode::M.to_bits() << mstatus::MPP_OFFSET,
            "Mstatus.MPP must be set to M mode (coming from Miralis)"
        );

        // Simulate a trap
        unsafe { Arch::write_csr(Csr::Mstatus, Mode::S.to_bits() << mstatus::MPP_OFFSET) };

        unsafe { ctx.switch_from_payload_to_firmware(&mut mctx) }

        // VirtContext Mstatus.MPP has been set to M mode
        assert_eq!(
            ctx.csr.mstatus & mstatus::MPP_FILTER,
            Mode::S.to_bits() << mstatus::MPP_OFFSET,
            "VirtContext Mstatus.MPP has been set to S mode (coming from payload)"
        );

        // Mstatus.MPP has been set to U mode
        assert_eq!(
            Arch::read_csr(Csr::Mstatus) & mstatus::MPP_FILTER,
            Mode::U.to_bits() << mstatus::MPP_OFFSET,
            "Mstatus.MPP has been set to U mode (going to firmware)"
        );
    }

    /// We test value of mideleg when switching from payload to firmware.
    /// Mideleg must always be 0 when executing the firware.
    #[test]
    fn switch_to_firmware_mideleg() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp);

        unsafe { Arch::write_csr(Csr::Mideleg, usize::MAX) };

        unsafe { ctx.switch_from_payload_to_firmware(&mut mctx) }

        assert_eq!(Arch::read_csr(Csr::Mideleg), 0, "Mideleg must be 0");
    }

    /// If the firmware wants to read the `mip` register after cleaning `vmip.SEIP`,
    /// and we don't sync `vmip.SEIP` with `mip.SEIP`, it can't know if there is an interrupt
    /// signal from the interrupt controller as the CSR read will be a logical-OR of the
    /// signal and `mip.SEIP` (which is one), and so always 1.
    /// If vmip.SEIP is 0, CSR read of mip.SEIP should return the interrupt signal.
    ///
    /// Then, we need to synchronize vmip.SEIP with mip.SEIP.
    #[test]
    fn csrr_external_interrupt() {
        let hw = unsafe { Arch::detect_hardware() };
        let mctx = MiralisContext::new(hw);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp);

        // This should set mip.SEIP
        ctx.set_csr(Csr::Mip, mie::SEIE_FILTER, &mctx.hw);

        assert_eq!(
            Arch::read_csr(Csr::Mip) & mie::SEIE_FILTER,
            mie::SEIE_FILTER,
            "mip.SEIP must be 1"
        );

        // This should clear mip.SEIP
        ctx.set_csr(Csr::Mip, 0, &mctx.hw);

        assert_eq!(
            Arch::read_csr(Csr::Mip) & mie::SEIE_FILTER,
            0,
            "mip.SEIP must be 0"
        );
    }
}
