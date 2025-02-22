//! The offload policy is a special policy used to reduce the number of world switches
//! It emulates the misaligned loads and stores, the read of the "time" register and offlaods the timer extension handling in Miralis

use core::sync::atomic::{AtomicBool, Ordering};

use miralis_core::sbi_codes;

use crate::arch::{get_raw_faulting_instr, mie, Arch, Architecture, Csr, MCause, Mode, Register};
use crate::config::PLATFORM_NB_HARTS;
use crate::host::MiralisContext;
use crate::platform::{Plat, Platform};
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::memory::{emulate_misaligned_read, emulate_misaligned_write};
use crate::virt::traits::{RegisterContextGetter, RegisterContextSetter};
use crate::virt::VirtContext;

/// Policy Machine Software Interrupt (MSI) map
static POLICY_SSI_ARRAY: [AtomicBool; PLATFORM_NB_HARTS] =
    [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS];
/// Remote instruction fence map
static FENCE_I_ARRAY: [AtomicBool; PLATFORM_NB_HARTS] =
    [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS];
/// Remote vma fence map
static FENCE_VMA_ARRAY: [AtomicBool; PLATFORM_NB_HARTS] =
    [const { AtomicBool::new(false) }; PLATFORM_NB_HARTS];

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
            unsafe {
                Arch::sfencevma(None, None);
            }
        }
    }

    const NUMBER_PMPS: usize = 0;
}

impl OffloadPolicy {
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
                Self::broadcast_ssi(ctx.get(Register::X10) << ctx.get(Register::X11));
                ctx.pc += 4;
                ctx.set(Register::X10, sbi_codes::SBI_SUCCESS);
                PolicyHookResult::Overwrite
            }
            _ if sbi_codes::is_i_fence_request(fid, eid) => {
                Self::broadcast_i_fence(ctx.get(Register::X10));
                ctx.pc += 4;
                ctx.set(Register::X10, sbi_codes::SBI_SUCCESS);
                PolicyHookResult::Overwrite
            }
            _ if sbi_codes::is_vma_request(fid, eid) => {
                Self::broadcast_vma_fence(ctx.get(Register::X10));
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

        let mut clint = Plat::get_clint().lock();
        clint.trigger_msi_on_all_harts(mask);
    }

    fn broadcast_vma_fence(mask: usize) {
        #[allow(clippy::needless_range_loop)]
        for idx in 0..PLATFORM_NB_HARTS {
            if mask & (1 << idx) != 0 {
                FENCE_VMA_ARRAY[idx].store(true, Ordering::SeqCst);
            }
        }

        let mut clint = Plat::get_clint().lock();
        clint.trigger_msi_on_all_harts(mask);
    }

    unsafe fn set_physical_ssip(&self) {
        Arch::set_csr_bits(Csr::Mip, mie::SSIE_FILTER);
    }
}
