//! The offload policy is a special policy used to reduce the number of world switches
//! It emulates the misaligned loads and stores, the read of the "time" register and offlaods the timer extension handling in Miralis

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use miralis_core::sbi_codes;

use crate::arch::hstatus::{GVA_FILTER, SPVP_FILTER, SPV_FILTER};
use crate::arch::mie::{SIE_FILTER, SSIE_FILTER};
use crate::arch::mstatus::{
    MPP_FILTER, MPP_OFFSET, MPV_FILTER, SPIE_FILTER, SPIE_OFFSET, SPP_FILTER, SPP_OFFSET,
};
use crate::arch::{
    get_raw_faulting_instr, mie, mstatus, parse_mpp_return_mode, Arch, Architecture, Csr, MCause,
    Mode, Register, PAGE_SIZE,
};
use crate::config::PLATFORM_NB_HARTS;
use crate::host::MiralisContext;
use crate::platform::{Plat, Platform};
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::memory::{emulate_misaligned_read, emulate_misaligned_write};
use crate::virt::traits::{RegisterContextGetter, RegisterContextSetter};
use crate::virt::VirtContext;

/// Policy Supervisor Software Interrupt (MSI) map
static POLICY_SSI_ARRAY: [AtomicBool; PLATFORM_NB_HARTS] =
    [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS];
/// Remote instruction fence map
static FENCE_I_ARRAY: [AtomicBool; PLATFORM_NB_HARTS] =
    [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS];
/// Remote vma fence map
static FENCE_VMA_ARRAY: [AtomicBool; PLATFORM_NB_HARTS] =
    [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS];

static FENCE_VMA_START: [AtomicUsize; PLATFORM_NB_HARTS] =
    [const { AtomicUsize::new(0) }; PLATFORM_NB_HARTS];

static FENCE_VMA_SIZE: [AtomicUsize; PLATFORM_NB_HARTS] =
    [const { AtomicUsize::new(0) }; PLATFORM_NB_HARTS];

pub const OFFLOAD_POLICY_NAME: &str = "Offload Policy";

pub struct OffloadPolicy {}

impl PolicyModule for OffloadPolicy {
    fn init() -> Self {
        OffloadPolicy {}
    }

    fn name() -> &'static str {
        OFFLOAD_POLICY_NAME
    }

    fn trap_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        let trap_info = ctx.trap_info.clone();

        match trap_info.get_cause() {
            MCause::LoadAddrMisaligned => emulate_misaligned_read(ctx, mctx),
            MCause::StoreAddrMisaligned => emulate_misaligned_write(ctx, mctx),
            MCause::LoadPageFault | MCause::StorePageFault | MCause::InstrPageFault => {
                Self::redirect_trap_to_s_mode(ctx)
            }
            MCause::EcallFromSMode => {
                Self::check_ecall(ctx, mctx, ctx.get(Register::X16), ctx.get(Register::X17))
            }
            MCause::IllegalInstr => {
                let instr = unsafe { get_raw_faulting_instr(ctx) };

                let is_privileged_op: bool = instr & 0x7f == 0b111_0011;
                let is_time_register: bool = (instr >> 20) == 0b1100_0000_0001;

                if is_privileged_op && is_time_register {
                    let rd = (instr >> 7) & 0b11111;
                    let _rs1 = (instr >> 15) & 0b11111;

                    let func3_mask = instr & 0b111000000000000;
                    match func3_mask {
                        0x2000 => {
                            ctx.set(Register::try_from(rd).unwrap(), Arch::read_csr(Csr::Time));
                            ctx.pc += 4;
                            return PolicyHookResult::Overwrite;
                        }
                        0x1000 | 0x3000 | 0x5000 | 0x6000 | 0x7000 => {
                            todo!("Handle the offload of other CSR instructions")
                        }
                        _ => {}
                    }
                }

                PolicyHookResult::Ignore
            }
            _ => PolicyHookResult::Ignore,
        }
    }

    fn on_interrupt(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext) {
        if POLICY_SSI_ARRAY[mctx.hw.hart]
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            if ctx.mode != Mode::M {
                unsafe { self.set_physical_ssip() };
            } else {
                ctx.csr.mip |= mie::SSIE_FILTER;
            }
        }

        if FENCE_I_ARRAY[mctx.hw.hart]
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            unsafe {
                Arch::ifence();
            }
        }

        if FENCE_VMA_ARRAY[mctx.hw.hart]
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let start = FENCE_VMA_START[mctx.hw.hart].load(Ordering::SeqCst);
            let size = FENCE_VMA_SIZE[mctx.hw.hart].load(Ordering::SeqCst);

            if start == 0 && size == usize::MAX {
                unsafe { Arch::sfencevma(None, None) };
            } else {
                for address in (start..start + size).step_by(PAGE_SIZE) {
                    unsafe { Arch::sfencevma(Some(address), None) };
                }
            }
        }
    }

    const NUMBER_PMPS: usize = 0;
}

impl OffloadPolicy {
    fn prepare_hart_mask(ctx: &mut VirtContext) -> usize {
        let hart_mask: usize = ctx.get(Register::X10);
        // Hart mask base corresponds to the hart where the mask starts
        let hart_mask_base: usize = ctx.get(Register::X11);
        hart_mask << hart_mask_base
    }

    fn check_ecall(
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
        fid: usize,
        eid: usize,
    ) -> PolicyHookResult {
        match (fid, eid) {
            _ if sbi_codes::is_timer_request(fid, eid) => {
                let v_clint = Plat::get_vclint();
                v_clint.set_payload_deadline(ctx, mctx, ctx.regs[Register::X10 as usize]);
                ctx.pc += 4;
                PolicyHookResult::Overwrite
            }
            _ if sbi_codes::is_ipi_request(fid, eid) => {
                Self::broadcast_ssi(Self::prepare_hart_mask(ctx));
                ctx.pc += 4;
                ctx.set(Register::X10, sbi_codes::SBI_SUCCESS);
                PolicyHookResult::Overwrite
            }
            _ if sbi_codes::is_i_fence_request(fid, eid) => {
                Self::broadcast_i_fence(Self::prepare_hart_mask(ctx));
                ctx.pc += 4;
                ctx.set(Register::X10, sbi_codes::SBI_SUCCESS);
                PolicyHookResult::Overwrite
            }
            _ if sbi_codes::is_vma_request(fid, eid) => {
                let start_address = ctx.get(Register::X12);
                let size = ctx.get(Register::X13);
                Self::broadcast_vma_fence(Self::prepare_hart_mask(ctx), start_address, size);
                ctx.pc += 4;
                ctx.set(Register::X10, sbi_codes::SBI_SUCCESS);
                PolicyHookResult::Overwrite
            }
            _ => PolicyHookResult::Ignore,
        }
    }

    fn broadcast_ssi(mask: usize) {
        #[allow(clippy::needless_range_loop)]
        for idx in 0..PLATFORM_NB_HARTS {
            if mask & (1 << idx) != 0 {
                POLICY_SSI_ARRAY[idx].store(true, Ordering::SeqCst);
            }
        }

        Plat::broadcast_policy_interrupt(mask);
    }

    fn broadcast_i_fence(mask: usize) {
        #[allow(clippy::needless_range_loop)]
        for idx in 0..PLATFORM_NB_HARTS {
            if mask & (1 << idx) != 0 {
                FENCE_I_ARRAY[idx].store(true, Ordering::SeqCst);
            }
        }

        Plat::broadcast_policy_interrupt(mask);
    }

    fn broadcast_vma_fence(mask: usize, start_address: usize, size: usize) {
        #[allow(clippy::needless_range_loop)]
        for idx in 0..PLATFORM_NB_HARTS {
            if mask & (1 << idx) != 0 {
                FENCE_VMA_ARRAY[idx].store(true, Ordering::SeqCst);
                FENCE_VMA_START[idx].store(start_address, Ordering::SeqCst);
                FENCE_VMA_SIZE[idx].store(size, Ordering::SeqCst);
            }
        }

        Plat::broadcast_policy_interrupt(mask);
    }

    unsafe fn set_physical_ssip(&self) {
        Arch::set_csr_bits(Csr::Mip, mie::SSIE_FILTER);
    }

    /// This function is a rust implementation of the function "sbi_trap_redirect" in the sbi_trap.c from the OpenSBI codebase
    /// This function corresponds to a payload to payload transition. Therefore, we don't modify the virtual context but the physical registers
    /// The only exception is ctx.pc because in the trap handler we write mepc with ctx.pc value.
    #[allow(unused)]
    fn redirect_trap_to_s_mode(ctx: &mut VirtContext) -> PolicyHookResult {
        let mut mstatus = ctx.trap_info.mstatus;

        // The previous virtualisation mode
        let prev_is_virt: bool = mstatus & MPV_FILTER != 0;

        assert!(
            !prev_is_virt,
            "Currently, we never tested this code when virtualisation is active, the feature might be unstable"
        );

        // Sanity check on previous mode
        let prev_mode = parse_mpp_return_mode(mstatus);
        assert!(
            prev_mode != Mode::S && prev_mode != Mode::U,
            "Trying to redirect a trap from the firmware to the payload"
        );

        // If exceptions came from VS/VU-mode, redirect to VS-mode if delegated in hedeleg
        let next_is_virt = ctx.extensions.has_h_extension
            && prev_is_virt
            && MCause::is_trap(MCause::try_from(ctx.trap_info.mcause).unwrap());

        // Update MSTATUS MPV bits
        mstatus &= !MPV_FILTER;
        mstatus |= if next_is_virt { MPV_FILTER } else { 0 };

        // Update hypervisor CSRs if going to HS-mode
        if ctx.extensions.has_h_extension && !next_is_virt {
            let mut hstatus = Arch::read_csr(Csr::Hstatus);

            if prev_is_virt {
                // hstatus.SPVP is only updated if coming from VS/VU-mode
                hstatus &= !SPVP_FILTER;
                hstatus |= if prev_mode == Mode::S { SPVP_FILTER } else { 0 };

                hstatus &= !SPV_FILTER;
                hstatus |= if prev_is_virt { SPV_FILTER } else { 0 };
                hstatus &= !GVA_FILTER;
                hstatus |= if ctx.trap_info.gva { GVA_FILTER } else { 0 };

                unsafe {
                    Arch::write_csr(Csr::Hstatus, hstatus);
                    Arch::write_csr(Csr::Htval, ctx.trap_info.mtval2);
                    Arch::write_csr(Csr::Htinst, ctx.trap_info.mtinst);
                }
            }
        }

        // Update exception related CSRs
        if (next_is_virt) {
            // Update VS-mode exception info
            unsafe {
                Arch::write_csr(Csr::Vstval, ctx.trap_info.mtval);
                Arch::write_csr(Csr::Vsepc, ctx.trap_info.mepc);
                Arch::write_csr(Csr::Vscause, ctx.trap_info.mcause);
            }

            // Set MEPC to VS-mode exception vector base
            ctx.pc = Arch::read_csr(Csr::Vstvec);

            // Set MPP to VS-mode
            mstatus &= !MPP_FILTER;
            mstatus |= (Mode::S as usize) << MPP_OFFSET;

            // Get VS-mode SSTATUS CSR
            let mut vsstatus = Arch::read_csr(Csr::Vsstatus);

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
                Arch::write_csr(Csr::Vsstatus, vsstatus);
            }
        } else {
            // Update S-mode exception info
            unsafe {
                Arch::write_csr(Csr::Stval, ctx.trap_info.mtval);
                Arch::write_csr(Csr::Sepc, ctx.trap_info.mepc);
                Arch::write_csr(Csr::Scause, ctx.trap_info.mcause);
            }

            // Jump to the Payload trap handler
            ctx.pc = Arch::read_csr(Csr::Stvec);

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

            unsafe {
                Arch::write_csr(Csr::Mstatus, mstatus);
            }
        }

        PolicyHookResult::Overwrite
    }
}
