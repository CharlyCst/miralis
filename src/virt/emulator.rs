//! RISC-V privileged instruction emulation

use miralis_core::abi;

use super::csr::traits::*;
use super::{VirtContext, VirtCsr};
use crate::arch::hstatus::{GVA_FILTER, SPV_FILTER, SPVP_FILTER};
use crate::arch::mie::{
    MEIE_OFFSET, MSIE_OFFSET, MTIE_OFFSET, SEIE_OFFSET, SIE_FILTER, SSIE_FILTER, SSIE_OFFSET,
    STIE_OFFSET,
};
use crate::arch::mstatus::{
    MPP_FILTER, MPP_OFFSET, MPV_FILTER, SPIE_FILTER, SPIE_OFFSET, SPP_FILTER, SPP_OFFSET,
};
use crate::arch::pmp::pmpcfg;
use crate::arch::{
    Csr, MCause, Mode, Register, get_raw_faulting_instr, mie, misa, mstatus, mtvec,
    parse_mpp_return_mode, parse_spp_return_mode,
};
use crate::decoder::{IllegalInst, LoadInstr, StoreInstr};
use crate::device::VirtDevice;
use crate::host::MiralisContext;
use crate::modules::{MainModule, Module};
use crate::platform::{Plat, Platform};
use crate::utils::sign_extend;
use crate::{arch, config, device, logger, utils};

/// Whether to continue execution of the virtual firmware or payload, or terminate the run loop.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ExitResult {
    /// Continue execution of the virtual firmware or payload.
    Continue,
    /// Terminate execution successfully.
    Done,
}

/// A load or store instruction.
#[derive(Debug)]
enum LoadStoreInstr {
    Load(LoadInstr),
    Store(StoreInstr),
}

impl VirtContext {
    /// Emulates a privileged instruction that caused an illegal instruction trap.
    ///
    /// Dispatches to the appropriate emulation handler based on the instruction type,
    /// and increments the program counter by 4 bytes for all instructions (except MRET and SRET).
    fn emulate_privileged_instr(&mut self, instr: &IllegalInst, mctx: &mut MiralisContext) {
        match instr {
            // Exceptional control flow, return before incrementing PC
            //
            // Invalid CSR operations (unknown CSR, write to read-only CSR) raise an illegal
            // instruction exception, the decoder takes care of detecting them.
            //
            // Mret and Sret do not trap but also skip the PC+4.
            IllegalInst::InvalidCsrOp => return self.emulate_firmware_trap(),
            IllegalInst::Unknown => return self.emulate_firmware_trap(),
            IllegalInst::Mret => return self.emulate_mret(mctx),
            IllegalInst::Sret => return self.emulate_sret(mctx),

            // Instructions with without exceptional control flow
            IllegalInst::Csrrw { csr, rd, rs1 } => self.emulate_csrrw(mctx, *csr, *rd, *rs1),
            IllegalInst::Csrrs { csr, rd, rs1 } => self.emulate_csrrs(mctx, *csr, *rd, *rs1),
            IllegalInst::Csrrc { csr, rd, rs1 } => self.emulate_csrrc(mctx, *csr, *rd, *rs1),
            IllegalInst::Csrrwi { csr, rd, uimm } => self.emulate_csrrwi(mctx, *csr, *rd, *uimm),
            IllegalInst::Csrrsi { csr, rd, uimm } => self.emulate_csrrsi(mctx, *csr, *rd, *uimm),
            IllegalInst::Csrrci { csr, rd, uimm } => self.emulate_csrrci(mctx, *csr, *rd, *uimm),
            IllegalInst::Wfi => self.emulate_wfi(mctx),
            IllegalInst::Sfencevma { rs1, rs2 } => self.emulate_sfence_vma(mctx, rs1, rs2),
            IllegalInst::Hfencegvma { rs1, rs2 } => self.emulate_hfence_gvma(mctx, rs1, rs2),
            IllegalInst::Hfencevvma { rs1, rs2 } => self.emulate_hfence_vvma(mctx, rs1, rs2),
        }

        // All instructions with exceptional control flow returned before, we increment the PC for
        // all others.
        self.pc = self.pc.wrapping_add(4);
    }

    /// Handles a devie load instruction.
    ///
    /// Calculates the memory address, reads the value from the device,
    /// sign-extends (normal load) or zero-extends (unsigned load) it to 64 bits if necessary,
    /// applies a mask and writes the value to the device.
    ///
    /// - Normal load&store instructions are 4 bytes long.
    /// - The immediate (`imm`) value can be positive or negative.
    /// - Compressed load&store instructions are 2 bytes long.
    /// - The immediate (`imm`) value is always positive.
    fn handle_device_load(&mut self, device: &VirtDevice, instr: &LoadInstr) {
        let LoadInstr {
            rd,
            rs1,
            imm,
            len,
            is_compressed,
            is_unsigned,
        } = instr;
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

    /// Handles a device store instruction.
    ///
    /// Calculates the memory address and writes the value
    /// to the device (after applying a mask to prevent overflow).
    fn handle_device_store(&mut self, device: &VirtDevice, instr: &StoreInstr) {
        let StoreInstr {
            rs2,
            rs1,
            imm,
            len,
            is_compressed,
        } = instr;
        let address = utils::calculate_addr(self.get(*rs1), *imm);
        let offset = address - device.start_addr;

        let value = self.get(*rs2);

        let mask = if len.to_bits() < usize::BITS as usize {
            (1 << len.to_bits()) - 1
        } else {
            usize::MAX
        };

        if value > mask {
            logger::warn_once!(
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

    /// Check whether a virtual M-mode data access of `[addr, addr + width)` is permitted by the
    /// guest firmware's virtual PMP registers (matching RISC-V M-mode `pmpCheck` semantics:
    /// partial matches always fault, unlocked full matches succeed and shadow lower-priority
    /// entries, and locked full matches enforce R/W permissions).
    #[inline(never)]
    pub fn check_firmware_pmp(&self, addr: usize, width: usize, is_store: bool) -> bool {
        let Some(addr_end) = addr.checked_add(width) else {
            return false;
        };
        let req_perm = if is_store { pmpcfg::W } else { pmpcfg::R };
        let nb = self.nb_pmp.min(64);
        let mut prev_pmpaddr: usize = 0;

        for i in 0..nb {
            let cfg = self.get_pmpcfg(i);
            let pmpaddr = self.get(Csr::Pmpaddr(i)) & Csr::PMP_ADDR_LEGAL_MASK;
            let (begin, end) = match cfg & pmpcfg::A_MASK {
                pmpcfg::INACTIVE => {
                    prev_pmpaddr = pmpaddr;
                    continue;
                }
                pmpcfg::TOR => {
                    let prev = prev_pmpaddr;
                    prev_pmpaddr = pmpaddr;
                    if prev >= pmpaddr {
                        continue;
                    }
                    (prev << 2, pmpaddr << 2)
                }
                pmpcfg::NA4 => {
                    prev_pmpaddr = pmpaddr;
                    let b = pmpaddr << 2;
                    (b, b + 4)
                }
                pmpcfg::NAPOT => {
                    prev_pmpaddr = pmpaddr;
                    let mask = pmpaddr ^ (pmpaddr + 1);
                    let base_w = pmpaddr & !mask;
                    let end_w = base_w + mask + 1;
                    (base_w << 2, end_w << 2)
                }
                _ => unreachable!(),
            };

            if addr_end <= begin || end <= addr {
                continue;
            }
            if begin <= addr && addr_end <= end {
                return (cfg & req_perm) == req_perm || (cfg & pmpcfg::L) == 0;
            }
            // Partial match faults even in M-mode with L = 0
            return false;
        }

        true
    }

    /// Handle a PMP fault due to a load or store instruction.
    ///
    /// When Miralis gets an access fault there might be three causes:
    /// - A load/store with MPRV set to 1 and MPP != M (checked first so virtual addresses are
    ///   translated via SATP and checked against MPP-mode PMP rather than treated as physical MMIO)
    /// - An emulated MMIO access, that is a device is being accessed
    /// - A normal access fault, which should be forwarded
    fn handle_pmp_fault(&mut self, mctx: &mut MiralisContext, instr: LoadStoreInstr) {
        if (self.csr.mstatus & mstatus::MPRV_FILTER) >> mstatus::MPRV_OFFSET == 1
            && parse_mpp_return_mode(self.csr.mstatus) != Mode::M
        {
            // The fault is due to an access with MPRV = 1 and MPP != M.
            //
            // MPRV has no effect when MPP = M, so the pmp fault is genuine and should be
            // re-injected as such.
            //
            // When MPP != M, Miralis needs to emulate the memory access. When the virtual firmware
            // sets MPRV = 1 Miralis configures the PMP to trap on all loads and stores, catches the
            // traps and emulates the loads/stores one by one.
            //
            // Sadly this is necessary as MPRV = 1 changes the access rights for loads & stores, but
            // not for instruction fetches, thus it is not possible to emulate the MPRV = 1
            // behavior using page tables. Of course the current emulation strategy comes with a
            // performance overhead.
            logger::trace!(
                "Access fault {:x?} with a virtual address: 0x{:x}",
                &instr,
                self.trap_info.mtval
            );
            match instr {
                LoadStoreInstr::Load(instr) => unsafe {
                    arch::handle_virtual_load(instr, self, mctx);
                },
                LoadStoreInstr::Store(instr) => unsafe {
                    arch::handle_virtual_store(instr, self, mctx);
                },
            }
        } else if let Some(device) =
            device::find_matching_device(self.trap_info.mtval, mctx.devices)
        {
            let (addr, width, is_store) = match &instr {
                LoadStoreInstr::Load(i) => (
                    utils::calculate_addr(self.get(i.rs1), i.imm),
                    i.len.to_bytes(),
                    false,
                ),
                LoadStoreInstr::Store(i) => (
                    utils::calculate_addr(self.get(i.rs1), i.imm),
                    i.len.to_bytes(),
                    true,
                ),
            };
            if addr >= device.start_addr
                && addr.saturating_add(width) <= device.start_addr + device.size
                && self.check_firmware_pmp(addr, width, is_store)
            {
                // The fault is due to an allowed access to a virtual device
                logger::trace!(
                    "Accessed devices: {} | With instr: {:?}",
                    device.name,
                    instr
                );
                match instr {
                    LoadStoreInstr::Load(instr) => self.handle_device_load(device, &instr),
                    LoadStoreInstr::Store(instr) => self.handle_device_store(device, &instr),
                }
            } else {
                self.emulate_firmware_trap();
            }
        } else {
            logger::trace!(
                "No matching device found for address: {:x}",
                self.trap_info.mtval
            );
            self.emulate_firmware_trap();
        }
    }

    /// Check if an interrupt should be injected in virtual M-mode, and perform the injection if
    /// any.
    ///
    /// If an interrupt is injected, jumps to the firmware trap handler.
    pub fn check_and_inject_interrupts(&mut self) {
        // For now, we assume that the vCPU will be run each time this function is called (or
        // rather, that this function is called before each vCPU run). Therefore, by running the
        // vCPU we exit the WFI mode, even if no interrupt is received (spurious wake-ups).
        self.is_wfi = false;

        if let Some(int_id) = self.has_pending_interrupt() {
            self.inject_interrupt(int_id)
        }
    }

    /// Return the next pending interrupt, if any.
    fn has_pending_interrupt(&mut self) -> Option<usize> {
        if self.csr.mstatus & mstatus::MIE_FILTER == 0 && self.mode == Mode::M && !self.is_wfi {
            // Interrupts are disabled while in M-mode if mstatus.MIE is 0
            return None;
        }

        get_next_interrupt(self.csr.mie, self.csr.mip, self.csr.mideleg)
    }

    /// Inject a virtual interrupt.
    ///
    /// This function jumps to the trap handler for the corresponding interrupts and updates the
    /// virtual CSRs accordingly.
    ///
    /// Not inlined: `check_and_inject_interrupts` runs on every trap and stays cheap only if it
    /// can be inlined. However, inlining `inject_interrupt` into `check_and_inject_interrupts`
    /// prevents the inlining of `check_and_inject_interrupts`. As with all manual performance
    /// tunning, this might need to be revisited on compiler upgrade.
    #[inline(never)]
    fn inject_interrupt(&mut self, next_int: usize) {
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

    /// Emulate a firmware trap, jumping to the firmware's mtvec.
    ///
    /// This function modifies the virtual context to emulate a hardware trap to M-mode. It injects
    /// the data in the trap info to propagate the cause of the trap physical trap to the virtual M-mode.
    pub fn emulate_firmware_trap(&mut self) {
        // Precondition to emulate a jump, we must be in a trap
        assert_eq!(
            self.trap_info.mcause & (1 << 63),
            0,
            "Mcause should represent a trap, not an interrupt"
        );

        // We are now emulating a trap, registers need to be updated
        logger::trace!("Emulating jump to trap handler");
        self.csr.mcause = self.trap_info.mcause;
        self.csr.mtval = self.trap_info.mtval;
        self.csr.mepc = self.trap_info.mepc;

        if self.extensions.has_h_extension {
            self.csr.mtinst = self.trap_info.mtinst;
            self.csr.mtval2 = self.trap_info.mtval2;
        }

        // Real mip.SEIE bit should not be different from virtual mip.SEIE as it is read-only in S-Mode or U-Mode.
        // But csrr is modified for SEIE and return the logical-OR of SEIE and the interrupt signal from interrupt
        // controller. (refer to documentation for further detail).
        // MSIE and MEIE could not be set by payload so it should be 0. The real value is read from hardware when
        // the firmware want to read virtual mip.
        //
        // We also preserve the virtualized interrupt bits from the virtual mip, as those are pure
        // software and might not match the physical mip.
        //
        // LCOFIP is set by the hardware on counter overflow, but Sscofpmf is not exposed to the
        // firmware for now, so we keep the virtual value.
        let sw_mask = mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO | mie::LCOFIE_FILTER;
        let hw_mip_bits = self.trap_info.mip & !sw_mask;
        let sw_mip_bits = self.csr.mip & sw_mask;
        self.csr.mip = hw_mip_bits | sw_mip_bits;

        match self.mode {
            Mode::M => {
                // Emulate the mstatus changes, as a trap from M-mode would.
                let mie = (self.csr.mstatus & mstatus::MIE_FILTER) >> mstatus::MIE_OFFSET;
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MPIE_OFFSET,
                    mstatus::MPIE_FILTER,
                    mie,
                );
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MIE_OFFSET,
                    mstatus::MIE_FILTER,
                    0,
                );
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    Mode::M.to_bits(),
                );
            }
            _ => {
                // The physical mstatus holds the payload's mstatus after the trap, including MPP.
                self.csr.mstatus = self.trap_info.mstatus;
                self.mode = Mode::M;
            }
        }

        // Go to firmware trap handler
        self.set_pc_to_mtvec();
    }

    /// Emulate a payload trap, re-injecting the trap as if it was delegated to the payload.
    ///
    /// This function is a rust implementation of the function "sbi_trap_redirect" in the sbi_trap.c from the OpenSBI codebase
    /// This function corresponds to a payload to payload transition. Therefore, we don't modify the virtual context but the physical registers
    /// The only exception is ctx.pc because in the trap handler we write mepc with ctx.pc value.
    pub fn emulate_payload_trap(&mut self) {
        let mut mstatus = self.trap_info.mstatus;

        // The previous virtualisation mode
        let prev_is_virt: bool = mstatus & MPV_FILTER != 0;

        assert!(
            !prev_is_virt,
            "Currently, we never tested this code when virtualisation is active, the feature might be unstable"
        );

        // Sanity check on previous mode
        let prev_mode = parse_mpp_return_mode(mstatus);
        assert!(
            !(prev_mode != Mode::S && prev_mode != Mode::U),
            "Trying to redirect a trap from the firmware to the payload"
        );

        // If exceptions came from VS/VU-mode, redirect to VS-mode if delegated in hedeleg
        let next_is_virt = self.extensions.has_h_extension
            && prev_is_virt
            && MCause::is_trap(MCause::try_from(self.trap_info.mcause).unwrap());

        // Update MSTATUS MPV bits
        mstatus &= !MPV_FILTER;
        mstatus |= if next_is_virt { MPV_FILTER } else { 0 };

        // Update hypervisor CSRs if going to HS-mode
        if self.extensions.has_h_extension && !next_is_virt {
            let mut hstatus = arch::read_csr(Csr::Hstatus);

            if prev_is_virt {
                // hstatus.SPVP is only updated if coming from VS/VU-mode
                hstatus &= !SPVP_FILTER;
                hstatus |= if prev_mode == Mode::S { SPVP_FILTER } else { 0 };

                hstatus &= !SPV_FILTER;
                hstatus |= if prev_is_virt { SPV_FILTER } else { 0 };
                hstatus &= !GVA_FILTER;
                hstatus |= if self.trap_info.gva { GVA_FILTER } else { 0 };

                unsafe {
                    arch::write_csr(Csr::Hstatus, hstatus);
                    arch::write_csr(Csr::Htval, self.trap_info.mtval2);
                    arch::write_csr(Csr::Htinst, self.trap_info.mtinst);
                }
            }
        }

        // Update exception related CSRs
        if next_is_virt {
            // Update VS-mode exception info
            unsafe {
                arch::write_csr(Csr::Vstval, self.trap_info.mtval);
                arch::write_csr(Csr::Vsepc, self.trap_info.mepc);
                arch::write_csr(Csr::Vscause, self.trap_info.mcause);
            }

            // Set MEPC to VS-mode exception vector base
            self.pc = arch::read_csr(Csr::Vstvec);

            // Set MPP to VS-mode
            mstatus &= !MPP_FILTER;
            mstatus |= (Mode::S as usize) << MPP_OFFSET;

            // Get VS-mode SSTATUS CSR
            let mut vsstatus = arch::read_csr(Csr::Vsstatus);

            // Set SPP for VS-mode
            vsstatus &= !SPP_FILTER;
            if prev_mode == Mode::S {
                vsstatus |= 1 << SPP_OFFSET;
            }

            // Set SPIE for VS-mode
            vsstatus &= !SPIE_FILTER;
            if vsstatus & SSIE_FILTER != 0 {
                vsstatus |= 1 << SPIE_OFFSET;
            }

            // Clear SIE for VS-mode
            vsstatus &= !SIE_FILTER;

            // Update VS-mode SSTATUS CSR
            unsafe {
                arch::write_csr(Csr::Vsstatus, vsstatus);
            }
        } else {
            // Update S-mode exception info
            unsafe {
                arch::write_csr(Csr::Stval, self.trap_info.mtval);
                arch::write_csr(Csr::Sepc, self.trap_info.mepc);
                arch::write_csr(Csr::Scause, self.trap_info.mcause);
            }

            // Jump to the Payload trap handler
            self.pc = arch::read_csr(Csr::Stvec);

            // Set MPP to S-mode
            mstatus &= !MPP_FILTER;
            mstatus |= (Mode::S as usize) << MPP_OFFSET;

            // Set SPP for S-mode
            mstatus &= !SPP_FILTER;
            if prev_mode == Mode::S {
                mstatus |= 1 << SPP_OFFSET;
            }

            // Set SPIE for S-mode
            mstatus &= !SPIE_FILTER;
            if mstatus & SIE_FILTER != 0 {
                mstatus |= SPIE_FILTER
            }

            // Clear SIE for S-mode
            mstatus &= !mstatus::SIE_FILTER;
        }

        unsafe {
            arch::write_csr(Csr::Mstatus, mstatus);
        }
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
                    // We use a wrapping add here to avoid an overflow
                    (self.csr.mtvec & mtvec::BASE_FILTER)
                        .wrapping_add(4_usize.wrapping_mul(MCause::cause_number(self.csr.mcause)))
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
        let v_clint = Plat::get_vclint();
        v_clint.handle_machine_timer_interrupt(self, mctx);
    }

    /// Handles a machine software interrupt trap
    fn handle_machine_software_interrupt(
        &mut self,
        mctx: &mut MiralisContext,
        module: &mut MainModule,
    ) {
        // Clear the interrupt
        Plat::get_clint()
            .write_msip(mctx.hw.hart, 0)
            .expect("Failed to write msip");

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
            module.on_interrupt(self, mctx);
        }
    }

    /// Handle the trap coming from the firmware
    pub fn handle_firmware_trap(
        &mut self,
        mctx: &mut MiralisContext,
        module: &mut MainModule,
    ) -> ExitResult {
        // The hardware sets mstatus.FS to Dirty when the firmware writes FP registers
        self.csr.mstatus = mstatus::with_hw_fs(self.csr.mstatus, self.trap_info.mstatus);

        if module.trap_from_firmware(mctx, self).overwrites() {
            logger::trace!("Catching trap in the policy module");
            return ExitResult::Continue;
        }

        let cause = self.trap_info.get_cause();
        match cause {
            MCause::EcallFromUMode if module.ecall_from_firmware(mctx, self).overwrites() => {
                // Nothing to do, the policy module handles those ecalls
                logger::trace!("Catching E-call from firmware in the policy module");
            }
            MCause::EcallFromUMode
                if config::FIRMWARE_ABI && self.get(Register::X17) == abi::MIRALIS_EID =>
            {
                return self.handle_ecall();
            }
            MCause::EcallFromUMode => {
                self.trap_info.mcause = MCause::EcallFromMMode as usize;
                self.emulate_firmware_trap();
            }
            MCause::EcallFromSMode => {
                panic!("Firmware should not be able to come from S-mode");
            }
            MCause::IllegalInstr => {
                let instr = unsafe { get_raw_faulting_instr(self) };

                // Illegal instruction can have two causes:
                // - privileged (system) instructions excepts ebreak and ecall
                // - Vector/floating points while they are disabled
                // For now we only decode system instructions, but we should handle floating
                // points/vector in the future.
                self.emulate_illegal_instruction(mctx, instr)
            }
            MCause::Breakpoint => {
                self.emulate_firmware_trap();
            }
            MCause::StoreAccessFault => {
                let instr = unsafe { get_raw_faulting_instr(self) };
                if MiralisContext::is_supported_store(instr) {
                    let instr = mctx.decode_store(instr);
                    self.handle_pmp_fault(mctx, LoadStoreInstr::Store(instr));
                } else {
                    // Not an access Miralis emulates (e.g. FP store or AMO)
                    self.emulate_firmware_trap();
                }
            }
            MCause::LoadAccessFault => {
                let instr = unsafe { get_raw_faulting_instr(self) };
                if MiralisContext::is_supported_load(instr) {
                    let instr = mctx.decode_load(instr);
                    self.handle_pmp_fault(mctx, LoadStoreInstr::Load(instr));
                } else {
                    // Not an access Miralis emulates (e.g. FP load or LR)
                    self.emulate_firmware_trap();
                }
            }
            MCause::InstrAccessFault => {
                logger::trace!("Instruction access fault: {:x?}", self.trap_info);
                self.emulate_firmware_trap();
            }
            MCause::MachineTimerInt => {
                self.handle_machine_timer_interrupt(mctx);
            }
            MCause::MachineSoftInt => {
                self.handle_machine_software_interrupt(mctx, module);
            }
            MCause::MachineExternalInt => {
                todo!("Virtualize machine external interrupt")
            }
            MCause::LoadAddrMisaligned
            | MCause::StoreAddrMisaligned
            | MCause::InstrAddrMisaligned => self.emulate_firmware_trap(),
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
        module: &mut MainModule,
    ) -> ExitResult {
        // Update the current mode
        self.mode = parse_mpp_return_mode(self.trap_info.mstatus);

        if module.trap_from_payload(mctx, self).overwrites() {
            logger::trace!("Catching trap in the policy module");
            return ExitResult::Continue;
        }

        // Handle the exit.
        // We only care about ecalls and virtualized interrupts.
        match self.trap_info.get_cause() {
            MCause::EcallFromSMode if module.ecall_from_payload(mctx, self).overwrites() => {
                // Nothing to do, the Policy module handles those ecalls
                logger::trace!("Catching E-call from payload in the policy module");
            }
            MCause::EcallFromSMode if self.get(Register::X17) == abi::MIRALIS_EID => {
                return self.handle_ecall();
            }
            MCause::EcallFromSMode => {
                logger::debug!(
                    "Forwarding ecall from s-mode with values 0x{:x}, 0x{:x} to the firmware",
                    self.get(Register::X16),
                    self.get(Register::X17)
                );
                self.emulate_firmware_trap();
            }
            MCause::MachineTimerInt => {
                self.handle_machine_timer_interrupt(mctx);
            }
            MCause::MachineSoftInt => {
                self.handle_machine_software_interrupt(mctx, module);
            }
            _ => self.emulate_firmware_trap(),
        }

        ExitResult::Continue
    }

    /// Handles Miralis-specific ecalls from firmware or payload.
    ///
    /// Miralis-specific ecalls are ecalls from the firmware or payload with extension ID (`eid`)
    /// equal to `miralis_core::abi::MIRALIS_EID`. The individual ecall functon IDs (`fid`s) are
    /// defined in the `miralis_core::abi` crate.
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
                return ExitResult::Done;
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

                // For now we don't return error code or the length written
                self.set(Register::X10, 0);
                self.set(Register::X11, 0);
            }
            _ => panic!("Invalid Miralis FID: 0x{:x}", fid),
        }

        self.pc += 4;
        ExitResult::Continue
    }

    /// Decodes and emulates an illegal instruction.
    fn emulate_illegal_instruction(&mut self, mctx: &mut MiralisContext, raw_instr: usize) {
        let instr = mctx.decode_illegal_instruction(raw_instr);
        logger::trace!("Faulting instruction: {:?}", instr);
        self.emulate_privileged_instr(&instr, mctx);
    }
}

// ——————————————————— Privileged Instructions Emulation ———————————————————— //

impl VirtContext {
    /// Emulate the WFI instruction, by putting the physical core in WFI state if needed.
    ///
    /// NOTE: for now there is no safeguard which guarantees that we will eventually get
    /// an interrupt, so the firmware might be able to put the core in perpetual sleep
    /// state.
    pub fn emulate_wfi(&mut self, _mctx: &mut MiralisContext) {
        // The WFI instruction put the processor in a special state that enables taking interrupts
        // even if mstatus.MIE = 0. We keep a bit in the virtual context to model that state.
        self.is_wfi = true;

        // If there is an interrupt that can be taken, then exit without doing a real WFI.
        // The emulator will inject the interrupt before resuming the vCPU.
        if get_next_interrupt(self.csr.mie, self.csr.mip, self.csr.mideleg).is_some() {
            // log::warn!("Early exit WFI on {}", self.hart_id);
            return;
        }

        // Otherwise if no interrupts are pending we execute a physical WFI.
        let prev_mie: usize;

        // Set mie to csr.mie, even if mstatus.MIE bit is cleared.
        unsafe { prev_mie = arch::write_csr(Csr::Mie, self.csr.mie) };

        arch::wfi();

        // Restore to previous mie value, including Miralis own bits
        unsafe { arch::write_csr(Csr::Mie, prev_mie) };
    }

    /// Emulates the CSRRW (CSR Read-Write) instruction.
    ///
    /// Atomically swaps values between a CSR and a register: reads the CSR into `rd`,
    /// then writes the value from `rs1` to the CSR.
    pub fn emulate_csrrw(
        &mut self,
        mctx: &mut MiralisContext,
        csr: Csr,
        rd: Register,
        rs1: Register,
    ) {
        let tmp = self.get(csr);
        self.set_csr(csr, self.get(rs1), mctx);
        self.set(rd, tmp);
    }

    /// Emulates the CSRRS (CSR Read and Set Bits) instruction.
    ///
    /// Reads the CSR into `rd`, then sets bits in the CSR according to the mask in `rs1`.
    /// If `rs1` is X0, the write is skipped.
    pub fn emulate_csrrs(
        &mut self,
        mctx: &mut MiralisContext,
        csr: Csr,
        rd: Register,
        rs1: Register,
    ) {
        let tmp = self.get(csr);

        // Skip the write if the mask is x0.
        //
        // This makes the emulator simpler as some pseudo-instructions (such as RDTIME) translate
        // to CSRRS with x0 as the mask.
        if rs1 != Register::X0 {
            self.set_csr(csr, tmp | self.get(rs1), mctx);
        }

        self.set(rd, tmp);
    }

    /// Emulates the CSRRWI (CSR Read-Write Immediate) instruction.
    ///
    /// Atomically swaps values between a CSR and an immediate: reads the CSR into `rd`,
    /// then writes the immediate value `uimm` to the CSR.
    pub fn emulate_csrrwi(
        &mut self,
        mctx: &mut MiralisContext,
        csr: Csr,
        rd: Register,
        uimm: usize,
    ) {
        self.set(rd, self.get(csr));
        self.set_csr(csr, uimm, mctx);
    }

    /// Emulates the CSRRSI (CSR Read and Set Bits Immediate) instruction.
    ///
    /// Reads the CSR into `rd`, then sets bits in the CSR according to the immediate mask.
    /// If `uimm` is 0, the write is ignored per the RISC-V Sail specification.
    pub fn emulate_csrrsi(
        &mut self,
        mctx: &mut MiralisContext,
        csr: Csr,
        rd: Register,
        uimm: usize,
    ) {
        let tmp = self.get(csr);

        // In the Sail specification writes with 0 as immediate are ignored
        if uimm != 0 {
            self.set_csr(csr, tmp | uimm, mctx);
        }

        self.set(rd, tmp);
    }

    /// Emulates the CSRRC (CSR Read and Clear Bits) instruction.
    ///
    /// Reads the CSR into `rd`, then clears bits in the CSR according to the mask in `rs1`.
    /// If `rs1` is X0, the write is skipped.
    pub fn emulate_csrrc(
        &mut self,
        mctx: &mut MiralisContext,
        csr: Csr,
        rd: Register,
        rs1: Register,
    ) {
        let tmp = self.get(csr);

        // Skip the write if the mask is x0.
        //
        // This makes the emulator simpler as some pseudo-instructions translate to CSRRC with x0
        // as the mask.
        if rs1 != Register::X0 {
            self.set_csr(csr, tmp & !self.get(rs1), mctx);
        }

        self.set(rd, tmp);
    }

    /// Emulates the CSRRCI (CSR Read and Clear Bits Immediate) instruction.
    ///
    /// Reads the CSR into `rd`, then clears bits in the CSR according to the immediate mask.
    /// If `uimm` is 0, the write is ignored per the RISC-V Sail specification.
    pub fn emulate_csrrci(
        &mut self,
        mctx: &mut MiralisContext,
        csr: Csr,
        rd: Register,
        uimm: usize,
    ) {
        let tmp = self.get(csr);

        // In the Sail specification writes with 0 as immediate are ignored
        if uimm != 0 {
            self.set_csr(csr, tmp & !uimm, mctx);
        }

        self.set(rd, tmp);
    }

    /// Emulates the MRET (Machine Return) instruction.
    pub fn emulate_mret(&mut self, mctx: &mut MiralisContext) {
        match parse_mpp_return_mode(self.csr.mstatus) {
            Mode::M => {
                logger::trace!("mret to m-mode to {:x}", self.trap_info.mepc);
                // Mret is jumping back to machine mode, do nothing
            }
            Mode::S if mctx.hw.extensions.has_s_extension => {
                logger::trace!("mret to s-mode to {:x}", self.trap_info.mepc);
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
                logger::trace!("mret to u-mode to {:x}", self.trap_info.mepc);
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

    /// Emulates the SRET (Supervisor Return) instruction.
    pub fn emulate_sret(&mut self, mctx: &mut MiralisContext) {
        match parse_spp_return_mode(self.csr.mstatus) {
            Mode::S if mctx.hw.extensions.has_s_extension => {
                log::trace!("sret to s-mode with SPP to {:x}", self.csr.sepc);
                // Sret is jumping to supervisor mode, the runner is the guest OS
                self.mode = Mode::S;
            }
            Mode::U => {
                log::trace!("mret to u-mode with SPP");
                // Sret is jumping to user mode, the runner is the guest OS
                self.mode = Mode::U;
            }
            _ => {
                panic!(
                    "SRET is not going to S/U mode: {} with MPP {:x}",
                    self.csr.mstatus,
                    (self.csr.mstatus & mstatus::SPP_FILTER) >> mstatus::SPP_OFFSET
                );
            }
        }

        let spie = (self.csr.mstatus & mstatus::SPIE_FILTER) >> mstatus::SPIE_OFFSET;

        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::SPIE_OFFSET,
            mstatus::SPIE_FILTER,
            1,
        );
        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::SIE_OFFSET,
            mstatus::SIE_FILTER,
            spie,
        );

        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::SPP_OFFSET,
            mstatus::SPP_FILTER,
            0b0,
        );

        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::MPRV_OFFSET,
            mstatus::MPRV_FILTER,
            0,
        );

        // Jump back to firmware
        self.pc = self.csr.sepc;
    }

    /// Emulate sfencevma by emitting a physical sfencevma.
    pub fn emulate_sfence_vma(
        &mut self,
        _mctx: &mut MiralisContext,
        rs1: &Register,
        rs2: &Register,
    ) {
        let vaddr = match rs1 {
            Register::X0 => None,
            reg => Some(self.get(reg)),
        };
        let asid = match rs2 {
            Register::X0 => None,
            reg => Some(self.get(reg)),
        };
        arch::sfencevma(vaddr, asid);
    }

    /// Emulate hfencegvma by emitting a physical hfencegvma.
    pub fn emulate_hfence_gvma(
        &mut self,
        _mctx: &mut MiralisContext,
        rs1: &Register,
        rs2: &Register,
    ) {
        let vaddr = match rs1 {
            Register::X0 => None,
            reg => Some(self.get(reg)),
        };
        let asid = match rs2 {
            Register::X0 => None,
            reg => Some(self.get(reg)),
        };
        arch::hfencegvma(vaddr, asid);
    }

    /// Emulate hfencevvma by emitting a physical hfencevvma.
    pub fn emulate_hfence_vvma(
        &mut self,
        _mctx: &mut MiralisContext,
        rs1: &Register,
        rs2: &Register,
    ) {
        let vaddr = match rs1 {
            Register::X0 => None,
            reg => Some(self.get(reg)),
        };
        let asid = match rs2 {
            Register::X0 => None,
            reg => Some(self.get(reg)),
        };
        arch::hfencevvma(vaddr, asid);
    }
}

// ————————————————————————————————— Utils —————————————————————————————————— //

/// Returns true if the virtual machine has support for U-mode.
fn has_user_mode(ctx: &VirtContext) -> bool {
    (ctx.csr.misa & misa::U) != 0
}

/// Retrieves the next interrupt by priority similar to the official risc-v specification
fn find_pending_interrupt_by_priority(ip: usize) -> Option<usize> {
    if ip & mie::MEIE_FILTER != 0 {
        Some(MEIE_OFFSET)
    } else if ip & mie::MSIE_FILTER != 0 {
        Some(MSIE_OFFSET)
    } else if ip & mie::MTIE_FILTER != 0 {
        Some(MTIE_OFFSET)
    } else if ip & mie::SEIE_FILTER != 0 {
        Some(SEIE_OFFSET)
    } else if ip & mie::SSIE_FILTER != 0 {
        Some(SSIE_OFFSET)
    } else if ip & mie::STIE_FILTER != 0 {
        Some(STIE_OFFSET)
    } else {
        None
    }
}

/// Return the ID of the next interrupt to be delivered, if any.
fn get_next_interrupt(mie: usize, mip: usize, mideleg: usize) -> Option<usize> {
    let ip = mie & mip & !mideleg;

    find_pending_interrupt_by_priority(ip)
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use super::get_next_interrupt;
    use crate::arch::{Csr, mie};
    use crate::host::MiralisContext;
    use crate::virt::VirtContext;
    use crate::{HwRegisterContextSetter, arch};

    /// If the firmware wants to read the `mip` register after cleaning `vmip.SEIP`,
    /// and we don't sync `vmip.SEIP` with `mip.SEIP`, it can't know if there is an interrupt
    /// signal from the interrupt controller as the CSR read will be a logical-OR of the
    /// signal and `mip.SEIP` (which is one), and so always 1.
    /// If vmip.SEIP is 0, CSR read of mip.SEIP should return the interrupt signal.
    ///
    /// Then, we need to synchronize vmip.SEIP with mip.SEIP.
    #[test]
    fn csrr_external_interrupt() {
        let hw = unsafe { arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, 0x10000, 0x2000);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        // This should set mip.SEIP
        ctx.set_csr(Csr::Mip, mie::SEIE_FILTER, &mut mctx);

        assert_eq!(
            arch::read_csr(Csr::Mip) & mie::SEIE_FILTER,
            mie::SEIE_FILTER,
            "mip.SEIP must be 1"
        );

        // This should clear mip.SEIP
        ctx.set_csr(Csr::Mip, 0, &mut mctx);

        assert_eq!(
            arch::read_csr(Csr::Mip) & mie::SEIE_FILTER,
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
        assert_eq!(get_next_interrupt(0b001, 0b001, 0b000), None);

        assert_eq!(get_next_interrupt(0b011, 0b011, 0b000), Some(1));
        assert_eq!(get_next_interrupt(0b010, 0b010, 0b000), Some(1));
        assert_eq!(get_next_interrupt(0b010, 0b011, 0b000), Some(1));
        assert_eq!(get_next_interrupt(0b011, 0b011, 0b001), Some(1));
    }

    #[test]
    fn firmware_ecall_forwarding() {
        use crate::modules::Module;
        let hw = unsafe { arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, 0x10000, 0x2000);
        let mut module = crate::modules::MainModule::init();
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());
        ctx.csr.mtvec = 0x8020_1000;
        ctx.trap_info.mcause = crate::arch::MCause::EcallFromUMode as usize;
        ctx.trap_info.mepc = 0x8020_0100;
        ctx.trap_info.mtval = 0;

        let res = ctx.handle_firmware_trap(&mut mctx, &mut module);
        assert!(res == super::ExitResult::Continue);
        assert_eq!(ctx.csr.mcause, crate::arch::MCause::EcallFromMMode as usize);
        assert_eq!(ctx.csr.mepc, 0x8020_0100);
        assert_eq!(ctx.csr.mtval, 0);
        assert_eq!(ctx.pc, 0x8020_1000);
    }

    #[test]
    fn pmp_lock_tor_and_firmware_check() {
        use crate::RegisterContextGetter;
        use crate::arch::pmp::{build_napot, build_tor, pmpcfg};

        let hw = unsafe { arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, 0x10000, 0x2000);

        // Non-multiple-of-8 nb_pmp (e.g., 3 virtual PMPs on an 8-PMP core)
        let mut ctx3 = VirtContext::new(0, 3, mctx.hw.extensions.clone());
        ctx3.pmp_grain = 0;
        let cfg_all = 0x0f0f_0f0f_0f0f_0f0f_usize;
        ctx3.set_csr(Csr::Pmpcfg(0), cfg_all, &mut mctx);
        assert_eq!(ctx3.get(Csr::Pmpcfg(0)), 0x0000_0000_000f_0f0f);
        ctx3.set_csr(Csr::Pmpaddr(2), 0x1234, &mut mctx);
        ctx3.set_csr(Csr::Pmpaddr(3), 0x5678, &mut mctx);
        assert_eq!(ctx3.get(Csr::Pmpaddr(2)), 0x1234);
        assert_eq!(ctx3.get(Csr::Pmpaddr(3)), 0);

        // Lock bit & TOR-lock semantics + check_firmware_pmp
        let mut ctx = VirtContext::new(0, 8, mctx.hw.extensions.clone());
        ctx.pmp_grain = 0;

        // Entry 0: base address 0x2000_000 (unlocked OFF), Entry 1: TOR up to 0x2010_000, L=1, R-only
        ctx.set_csr(Csr::Pmpaddr(0), build_tor(0x2000_000), &mut mctx);
        ctx.set_csr(Csr::Pmpaddr(1), build_tor(0x2010_000), &mut mctx);
        let cfg = ((pmpcfg::L | pmpcfg::TOR | pmpcfg::R) as usize) << 8;
        ctx.set_csr(Csr::Pmpcfg(0), cfg, &mut mctx);

        // Entry 1 is locked: writes to pmpcfg[1], pmpaddr[1], AND pmpaddr[0] (TOR-locked!) are ignored
        ctx.set_csr(Csr::Pmpcfg(0), 0, &mut mctx);
        assert_eq!(ctx.get(Csr::Pmpcfg(0)), cfg);
        ctx.set_csr(Csr::Pmpaddr(0), 0, &mut mctx);
        ctx.set_csr(Csr::Pmpaddr(1), 0, &mut mctx);
        assert_eq!(ctx.get(Csr::Pmpaddr(0)), build_tor(0x2000_000));
        assert_eq!(ctx.get(Csr::Pmpaddr(1)), build_tor(0x2010_000));

        // Firmware M-mode check: R allowed, W denied by locked Entry 1, partial match faults
        assert!(ctx.check_firmware_pmp(0x2000_000, 4, false));
        assert!(!ctx.check_firmware_pmp(0x2000_000, 4, true));
        assert!(!ctx.check_firmware_pmp(0x1fff_ffe, 4, false));

        // Higher-priority unlocked Entry 0 (L=0, NAPOT, R=W=X=0) shadows locked Entry 1 in M-mode!
        ctx.set_csr(
            Csr::Pmpcfg(0),
            cfg | (pmpcfg::NAPOT as usize), // Entry 0 is still unlocked (only pmpaddr0 was TOR-locked)
            &mut mctx,
        );
        // Set Entry 2 as locked NAPOT R=0,W=0 over [0x8020_0000, 0x8020_1000), shadowed by unlocked Entry 0?
        // Test priority shadowing cleanly on entries 2 (L=0) and 3 (L=1):
        let napot = build_napot(0x8020_0000, 0x1000).unwrap();
        ctx.set_csr(Csr::Pmpaddr(2), napot, &mut mctx);
        ctx.set_csr(Csr::Pmpaddr(3), napot, &mut mctx);
        let cfg23 =
            ((pmpcfg::NAPOT as usize) << 16) | (((pmpcfg::L | pmpcfg::NAPOT) as usize) << 24);
        ctx.set_csr(Csr::Pmpcfg(0), cfg | cfg23, &mut mctx);
        assert!(ctx.check_firmware_pmp(0x8020_0000, 8, true));
    }
}
