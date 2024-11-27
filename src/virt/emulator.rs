//! RISC-V privileged instruction emulation

use miralis_core::abi;

use super::csr::traits::*;
use super::{VirtContext, VirtCsr};
use crate::arch::{
    mie, misa, mstatus, mtvec, parse_mpp_return_mode, Arch, Architecture, Csr, MCause, Mode,
    Register,
};
use crate::benchmark::Benchmark;
use crate::decoder::Instr;
use crate::device::VirtDevice;
use crate::host::MiralisContext;
use crate::platform::{Plat, Platform};
use crate::policy::{Policy, PolicyModule};
use crate::utils::sign_extend;
use crate::{device, utils};

/// Wether to continue execution of the virtual firmware or payload, or terminate the run loop.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ExitResult {
    /// Continue execution of the virtual firmware or payload.
    Continue,
    /// Terminate execution successfully.
    Donne,
}

impl VirtContext {
    fn emulate_privileged_instr(&mut self, instr: &Instr, mctx: &mut MiralisContext) {
        match instr {
            Instr::Wfi => self.emulate_wfi(mctx),
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
            Instr::Csrrw { csr, rd, rs1 } => self.emulate_csrrw(mctx, csr, rd, rs1),
            Instr::Csrrs { csr, rd, rs1 } => self.emulate_csrrs(mctx, csr, rd, rs1),
            Instr::Csrrc { csr, rd, rs1 } => self.emulate_csrrc(mctx, csr, rd, rs1),
            Instr::Csrrwi { csr, rd, uimm } => self.emulate_csrrwi(mctx, csr, rd, uimm),
            Instr::Csrrsi { csr, rd, uimm } => self.emulate_csrrsi(mctx, csr, rd, uimm),
            Instr::Csrrci { csr, rd, uimm } => self.emulate_csrrci(mctx, csr, rd, uimm),
            Instr::Mret => self.emulate_mret(mctx),
            Instr::Sfencevma { rs1, rs2 } => self.emulate_sfence_vma(mctx, rs1, rs2),
            Instr::Hfencegvma { rs1, rs2 } => self.emulate_hfence_gvma(mctx, rs1, rs2),
            Instr::Hfencevvma { rs1, rs2 } => self.emulate_hfence_vvma(mctx, rs1, rs2),
            _ => todo!(
                "Instruction not yet implemented: {:?} {:x} {:x}",
                instr,
                self.csr.mepc,
                self.csr.mtval
            ),
        }
    }

    /// Handles a load instruction.
    ///
    /// Calculates the memory address, reads the value from the device,
    /// sign-extends (normal load) or zero-extends (unsigned load) it to 64 bits if necessary,
    /// applies a mask and writes the value to the device.
    ///
    /// - Normal load&store instructions are 4 bytes long.
    /// - The immediate (`imm`) value can be positive or negative.
    /// - Compressed load&store instructions are 2 bytes long.
    /// - The immediate (`imm`) value is always positive.
    fn handle_load(&mut self, device: &VirtDevice, instr: &Instr) {
        match instr {
            Instr::Load {
                rd,
                rs1,
                imm,
                len,
                is_compressed,
                is_unsigned,
            } => {
                let address = utils::calculate_addr(self.get(*rs1), *imm);
                let offset = address - device.start_addr;

                match device.device_interface.read_device(offset, *len, self) {
                    Ok(value) => {
                        let value = if !is_unsigned {
                            sign_extend(value, *len)
                        } else {
                            value
                        };

                        self.set(*rd, value);
                        self.pc += if *is_compressed { 2 } else { 4 };
                    }
                    Err(err) => panic!("Error reading {}: {}", device.name, err),
                }
            }
            _ => panic!("Not a load instruction in a load handler"),
        }
    }

    /// Handles a store instruction.
    ///
    /// Calculates the memory address and writes the value
    /// to the device (after applying a mask to prevent overflow).
    fn handle_store(&mut self, device: &VirtDevice, instr: &Instr) {
        match instr {
            Instr::Store {
                rs2,
                rs1,
                imm,
                len,
                is_compressed,
            } => {
                let address = utils::calculate_addr(self.get(*rs1), *imm);
                let offset = address - device.start_addr;

                let value = self.get(*rs2);

                let mask = if len.to_bits() < usize::BITS as usize {
                    (1 << len.to_bits()) - 1
                } else {
                    usize::MAX
                };

                if value > mask {
                    log::warn!(
                        "Value {} exceeds allowed length {}. Trimming to fit.",
                        value,
                        len.to_bits()
                    );
                }

                match device
                    .device_interface
                    .write_device(offset, *len, value & mask, self)
                {
                    Ok(()) => {
                        // Update the program counter (pc) based on compression
                        self.pc += if *is_compressed { 2 } else { 4 };
                    }
                    Err(err) => panic!("Error writing {}: {}", device.name, err),
                }
            }
            _ => panic!("Not a store instruction in a store handler"),
        }
    }

    pub fn handle_device_access_fault(&mut self, instr: &Instr, device: &VirtDevice) {
        match instr {
            Instr::Load { .. } => self.handle_load(device, instr),
            Instr::Store { .. } => self.handle_store(device, instr),
            _ => todo!("Instruction not yet implemented: {:?}", instr),
        }
    }

    /// Check if an interrupt should be injected in virtual M-mode.
    ///
    /// If an interrupt is injected, jumps to the firmware trap handler.
    pub fn check_and_inject_interrupts(&mut self) {
        if self.csr.mstatus & mstatus::MIE_FILTER == 0 && self.mode == Mode::M {
            // Interrupts are disabled while in M-mode if mstatus.MIE is 0
            return;
        }
        let Some(next_int) = get_next_interrupt(self.csr.mie, self.csr.mip, self.csr.mideleg)
        else {
            // No enabled interrupt pending
            return;
        };

        // Update Mstatus to match the semantic of a trap
        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::MPP_OFFSET,
            mstatus::MPP_FILTER,
            self.mode.to_bits(),
        );
        let mpie = (self.csr.mstatus & mstatus::MIE_FILTER) >> mstatus::MIE_OFFSET;
        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::MPIE_OFFSET,
            mstatus::MPIE_FILTER,
            mpie,
        );
        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::MIE_OFFSET,
            mstatus::MIE_FILTER,
            0,
        );

        let mcause = next_int | (1 << (usize::BITS - 1));
        self.csr.mcause = mcause;
        self.csr.mepc = self.pc;
        self.csr.mtval = 0;
        self.mode = Mode::M;
        self.set_pc_to_mtvec();
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
        // MSIE and MEIE could not be set by payload so it should be 0. The real value is read from hardware when
        // the firmware want to read virtual mip.
        //
        // We also preserve the virtualized interrupt bits from the virtual mip, as those are pure
        // software and might not match the physical mip.
        let hw_mip_bits = self.trap_info.mip & !(mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO);
        let sw_mip_bits = self.csr.mip & (mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO);
        self.csr.mip = hw_mip_bits | sw_mip_bits;

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
        self.set_pc_to_mtvec();
    }

    /// Set the program counter (PC) to `mtvec`, amulating a jump to the trap handler.
    ///
    /// This function checks the `mcause` CSR to select the right entry point if `mtvec` is in
    /// vectored more. Therefore it assumes `mcause` has been configured prior to calling this
    /// function.
    fn set_pc_to_mtvec(&mut self) {
        self.pc = match mtvec::get_mode(self.csr.mtvec) {
            // If Direct mode: just jump to BASE directly
            mtvec::Mode::Direct => self.csr.mtvec & mtvec::BASE_FILTER,
            // If Vectored mode: if synchronous exception, jump to the BASE directly
            // else, jump to BASE + 4 * cause
            mtvec::Mode::Vectored => {
                if MCause::is_interrupt(MCause::new(self.csr.mcause)) {
                    (self.csr.mtvec & mtvec::BASE_FILTER)
                        + 4 * MCause::cause_number(self.csr.mcause)
                } else {
                    self.csr.mtvec & mtvec::BASE_FILTER
                }
            }
        }
    }

    /// Handles a machine timer interrupt
    ///
    /// TODO: for now we assume that all M-mode timer interrupts are issued from the
    /// firmware (in-band interrupts), so we just set the bit in `vmip`.
    /// In the future we might want to support timer interrupts for Miralis' own purpose
    /// (out-of-band interrupts). Once we add such support we should disambiguate
    /// interrupts here.
    fn handle_machine_timer_interrupt(&mut self, mctx: &mut MiralisContext) {
        let mut clint = Plat::get_clint().lock();
        clint
            .write_mtimecmp(mctx.hw.hart, usize::MAX)
            .expect("Failed to write mtimecmp");
        drop(clint); // Release the lock early

        self.csr.mip |= mie::MTIE_FILTER;
    }

    /// Handles a machine software interrupt trap
    fn handle_machine_software_interrupt(
        &mut self,
        mctx: &mut MiralisContext,
        policy: &mut Policy,
    ) {
        // Clear the interrupt
        let mut clint = Plat::get_clint().lock();
        clint
            .write_msip(mctx.hw.hart, 0)
            .expect("Failed to write msip");
        drop(clint); // Release the lock early

        // Check if a virtual MSI is pending
        let vclint = Plat::get_vclint();
        if vclint.get_vmsi(self.hart_id) {
            self.csr.mip |= mie::MSIE_FILTER;
        } else {
            self.csr.mip &= !mie::MSIE_FILTER;
        }

        // Check if a policy MSI is pending
        if vclint.get_policy_msi(self.hart_id) {
            vclint.clear_policy_msi(self.hart_id);
            policy.on_interrupt(self, mctx);
        }
    }

    /// Handle the trap coming from the firmware
    pub fn handle_firmware_trap(
        &mut self,
        mctx: &mut MiralisContext,
        policy: &mut Policy,
    ) -> ExitResult {
        if policy.trap_from_firmware(mctx, self).overwrites() {
            log::trace!("Catching trap in the policy module");
            return ExitResult::Continue;
        }

        let cause = self.trap_info.get_cause();
        match cause {
            MCause::EcallFromUMode if policy.ecall_from_firmware(mctx, self).overwrites() => {
                // Nothing to do, the policy module handles those ecalls
                log::trace!("Catching E-call from firmware in the policy module");
            }
            MCause::EcallFromUMode if self.get(Register::X17) == abi::MIRALIS_EID => {
                return self.handle_ecall();
            }
            MCause::EcallFromUMode => {
                todo!("ecall is not yet supported for EID other than Miralis ABI");
            }
            MCause::EcallFromSMode => {
                panic!("Firmware should not be able to come from S-mode");
            }
            MCause::IllegalInstr => {
                let instr = unsafe { Arch::get_raw_faulting_instr(&self.trap_info) };
                let instr = mctx.decode(instr);
                log::trace!("Faulting instruction: {:?}", instr);
                self.emulate_privileged_instr(&instr, mctx);
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
                    let instr = mctx.decode(instr);
                    log::trace!(
                        "Accessed devices: {} | With instr: {:?}",
                        device.name,
                        instr
                    );
                    self.handle_device_access_fault(&instr, device);
                } else if (self.csr.mstatus & mstatus::MPRV_FILTER) >> mstatus::MPRV_OFFSET == 1 {
                    // TODO: make sure virtual address does not get around PMP protection
                    let instr = unsafe { Arch::get_raw_faulting_instr(&self.trap_info) };
                    let instr = mctx.decode(instr);
                    log::trace!(
                        "Access fault {:x?} with a virtual address: 0x{:x}",
                        &instr,
                        self.trap_info.mtval
                    );
                    unsafe {
                        Arch::handle_virtual_load_store(instr, self);
                    }
                } else {
                    log::trace!(
                        "No matching device found for address: {:x}",
                        self.trap_info.mtval
                    );
                    self.emulate_jump_trap_handler();
                }
            }
            MCause::InstrAccessFault => {
                log::trace!("Instruction access fault: {:x?}", self.trap_info);
                self.emulate_jump_trap_handler();
            }
            MCause::MachineTimerInt => {
                self.handle_machine_timer_interrupt(mctx);
            }
            MCause::MachineSoftInt => {
                log::info!("Machine soft int");
                self.handle_machine_software_interrupt(mctx, policy);
            }
            MCause::MachineExternalInt => {
                todo!("Virtualize machine external interrupt")
            }
            MCause::LoadAddrMisaligned
            | MCause::StoreAddrMisaligned
            | MCause::InstrAddrMisaligned => self.emulate_jump_trap_handler(),
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

        ExitResult::Continue
    }

    /// Handle the trap coming from the payload
    pub fn handle_payload_trap(
        &mut self,
        mctx: &mut MiralisContext,
        policy: &mut Policy,
    ) -> ExitResult {
        // Update the current mode
        self.mode = parse_mpp_return_mode(self.trap_info.mstatus);

        if policy.trap_from_payload(mctx, self).overwrites() {
            log::trace!("Catching trap in the policy module");
            return ExitResult::Continue;
        }

        // Handle the exit.
        // We only care about ecalls and virtualized interrupts.
        match self.trap_info.get_cause() {
            MCause::EcallFromSMode if policy.ecall_from_payload(mctx, self).overwrites() => {
                // Nothing to do, the Policy module handles those ecalls
                log::trace!("Catching E-call from payload in the policy module");
            }
            MCause::EcallFromSMode if self.get(Register::X17) == abi::MIRALIS_EID => {
                return self.handle_ecall();
            }
            MCause::MachineTimerInt => {
                self.handle_machine_timer_interrupt(mctx);
            }
            MCause::MachineSoftInt => {
                self.handle_machine_software_interrupt(mctx, policy);
            }
            _ => self.emulate_jump_trap_handler(),
        }

        ExitResult::Continue
    }

    /// Ecalls may come from firmware or payload, resulting in different handling.
    fn handle_ecall(&mut self) -> ExitResult {
        let fid = self.get(Register::X16);
        match fid {
            abi::MIRALIS_FAILURE_FID => {
                log::error!("Firmware or payload panicked!");
                log::error!("  pc:    0x{:x}", self.pc);
                log::error!("  exits: {}", self.nb_exits);
                panic!();
            }
            abi::MIRALIS_SUCCESS_FID => {
                log::info!("Success!");
                log::info!("Number of exits: {}", self.nb_exits);
                // Terminate execution
                return ExitResult::Donne;
            }
            abi::MIRALIS_LOG_FID => {
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

        ExitResult::Continue
    }
}

// ——————————————————— Privileged Instructions Emulation ———————————————————— //

impl VirtContext {
    pub fn emulate_wfi(&mut self, _mctx: &mut MiralisContext) {
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

    pub fn emulate_csrrw(
        &mut self,
        mctx: &mut MiralisContext,
        csr: &Csr,
        rd: &Register,
        rs1: &Register,
    ) {
        let tmp = self.get(csr);
        self.set_csr(csr, self.get(rs1), mctx);
        self.set(rd, tmp);
        self.pc += 4;
    }

    pub fn emulate_csrrs(
        &mut self,
        mctx: &mut MiralisContext,
        csr: &Csr,
        rd: &Register,
        rs1: &Register,
    ) {
        let tmp = self.get(csr);
        self.set_csr(csr, tmp | self.get(rs1), mctx);
        self.set(rd, tmp);
        self.pc += 4;
    }

    pub fn emulate_csrrwi(
        &mut self,
        mctx: &mut MiralisContext,
        csr: &Csr,
        rd: &Register,
        uimm: &usize,
    ) {
        self.set(rd, self.get(csr));
        self.set_csr(csr, *uimm, mctx);
        self.pc += 4;
    }

    pub fn emulate_csrrsi(
        &mut self,
        mctx: &mut MiralisContext,
        csr: &Csr,
        rd: &Register,
        uimm: &usize,
    ) {
        let tmp = self.get(csr);
        self.set_csr(csr, tmp | uimm, mctx);
        self.set(rd, tmp);
        self.pc += 4;
    }

    pub fn emulate_csrrc(
        &mut self,
        mctx: &mut MiralisContext,
        csr: &Csr,
        rd: &Register,
        rs1: &Register,
    ) {
        let tmp = self.get(csr);
        self.set_csr(csr, tmp & !self.get(rs1), mctx);
        self.set(rd, tmp);
        self.pc += 4;
    }

    pub fn emulate_csrrci(
        &mut self,
        mctx: &mut MiralisContext,
        csr: &Csr,
        rd: &Register,
        uimm: &usize,
    ) {
        let tmp = self.get(csr);
        self.set_csr(csr, tmp & !uimm, mctx);
        self.set(rd, tmp);
        self.pc += 4;
    }

    pub fn emulate_mret(&mut self, mctx: &mut MiralisContext) {
        match parse_mpp_return_mode(self.csr.mstatus) {
            Mode::M => {
                log::trace!("mret to m-mode to {:x}", self.trap_info.mepc);
                // Mret is jumping back to machine mode, do nothing
            }
            Mode::S if mctx.hw.extensions.has_s_extension => {
                log::trace!("mret to s-mode with MPP to {:x}", self.trap_info.mepc);
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

        let ret_mpp_val: usize = if has_user_mode(self) { 0b00 } else { 0b11 };

        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::MPP_OFFSET,
            mstatus::MPP_FILTER,
            ret_mpp_val,
        );

        // Jump back to firmware
        self.pc = self.csr.mepc;
    }

    pub fn emulate_sfence_vma(
        &mut self,
        _mctx: &mut MiralisContext,
        rs1: &Register,
        rs2: &Register,
    ) {
        unsafe {
            let vaddr = match rs1 {
                Register::X0 => None,
                reg => Some(self.get(reg)),
            };
            let asid = match rs2 {
                Register::X0 => None,
                reg => Some(self.get(reg)),
            };
            Arch::sfencevma(vaddr, asid);
            self.pc += 4;
        }
    }

    pub fn emulate_hfence_gvma(
        &mut self,
        _mctx: &mut MiralisContext,
        rs1: &Register,
        rs2: &Register,
    ) {
        unsafe {
            let vaddr = match rs1 {
                Register::X0 => None,
                reg => Some(self.get(reg)),
            };
            let asid = match rs2 {
                Register::X0 => None,
                reg => Some(self.get(reg)),
            };
            Arch::hfencegvma(vaddr, asid);
            self.pc += 4;
        }
    }

    pub fn emulate_hfence_vvma(
        &mut self,
        _mctx: &mut MiralisContext,
        rs1: &Register,
        rs2: &Register,
    ) {
        unsafe {
            let vaddr = match rs1 {
                Register::X0 => None,
                reg => Some(self.get(reg)),
            };
            let asid = match rs2 {
                Register::X0 => None,
                reg => Some(self.get(reg)),
            };
            Arch::hfencevvma(vaddr, asid);
            self.pc += 4;
        }
    }
}

// ————————————————————————————————— Utils —————————————————————————————————— //

fn has_user_mode(ctx: &VirtContext) -> bool {
    (ctx.csr.misa & misa::U) != 0
}

/// Return the ID of the next interrupt to be delivered, if any.
fn get_next_interrupt(mie: usize, mip: usize, mideleg: usize) -> Option<usize> {
    let ints = mie & mip & !mideleg;
    if ints == 0 {
        None
    } else {
        // TODO: use the same priority as hardware.
        // Currently we serve the less significant bit first.
        Some(ints.trailing_zeros() as usize)
    }
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use super::get_next_interrupt;
    use crate::arch::{mie, Arch, Architecture, Csr};
    use crate::host::MiralisContext;
    use crate::virt::VirtContext;
    use crate::HwRegisterContextSetter;

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
        let mut mctx = MiralisContext::new(hw, 0x10000, 0x2000);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        // This should set mip.SEIP
        ctx.set_csr(Csr::Mip, mie::SEIE_FILTER, &mut mctx);

        assert_eq!(
            Arch::read_csr(Csr::Mip) & mie::SEIE_FILTER,
            mie::SEIE_FILTER,
            "mip.SEIP must be 1"
        );

        // This should clear mip.SEIP
        ctx.set_csr(Csr::Mip, 0, &mut mctx);

        assert_eq!(
            Arch::read_csr(Csr::Mip) & mie::SEIE_FILTER,
            0,
            "mip.SEIP must be 0"
        );
    }

    #[test]
    fn next_interrupt() {
        assert_eq!(get_next_interrupt(0b000, 0b000, 0b000), None);
        assert_eq!(get_next_interrupt(0b010, 0b000, 0b000), None);
        assert_eq!(get_next_interrupt(0b000, 0b010, 0b000), None);
        assert_eq!(get_next_interrupt(0b010, 0b010, 0b010), None);

        assert_eq!(get_next_interrupt(0b001, 0b001, 0b000), Some(0));
        assert_eq!(get_next_interrupt(0b011, 0b011, 0b000), Some(0));
        assert_eq!(get_next_interrupt(0b010, 0b010, 0b000), Some(1));
        assert_eq!(get_next_interrupt(0b010, 0b011, 0b000), Some(1));
        assert_eq!(get_next_interrupt(0b011, 0b011, 0b001), Some(1));
    }
}
