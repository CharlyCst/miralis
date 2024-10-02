//! The protect payload policy, it protects the payload.

use miralis_core::abi_protect_payload;

use crate::arch::pmp::{build_napot, pmpcfg};
use crate::arch::{MCause, Register};
use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::{RegisterContextGetter, VirtContext};

/// The protect payload policy module, which allow the payload to protect himself from the firmware at some point in time and enfore a boundary between the two components.
pub struct ProtectPayloadPolicy {
    pub protected: bool,
    general_register: [usize; 32],
}

impl PolicyModule for ProtectPayloadPolicy {
    fn init() -> Self {
        ProtectPayloadPolicy {
            protected: false,
            general_register: [0; 32],
        }
    }
    fn name() -> &'static str {
        "Protect Payload Policy"
    }

    fn ecall_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        if !self.is_policy_call(ctx) {
            return PolicyHookResult::Ignore;
        }

        if ctx.get(Register::X16) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_LOCK_FID {
            self.lock(mctx);
            ctx.pc += 4;
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }

    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        if !self.is_policy_call(ctx) {
            return PolicyHookResult::Ignore;
        }

        if ctx.get(Register::X16) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_LOCK_FID {
            self.lock(mctx);
            ctx.pc += 4;
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }

    fn switch_from_payload_to_firmware(&mut self, ctx: &mut VirtContext) {
        if !self.protected {
            return;
        }

        // Step 1: Clear general purpose registers
        for i in 0..32 {
            self.general_register[i] = ctx.regs[i];
            // We don't clear ecall registers
            if self.clear_register(i, ctx) {
                ctx.regs[i] = 0;
            }
        }
    }

    fn switch_from_firmware_to_payload(&mut self, ctx: &mut VirtContext) {
        if !self.protected {
            return;
        }

        // Step 1: Restore general purpose registers
        for i in 0..32 {
            // 10 & 11 are return registers
            if self.restore_register(i, ctx) {
                ctx.regs[i] = self.general_register[i];
            }
        }
    }

    const NUMBER_PMPS: usize = 1;
}

impl ProtectPayloadPolicy {
    fn lock(&mut self, mctx: &mut MiralisContext) {
        // TODO: Make it dynamic in the future
        // First set pmp entry protection
        mctx.pmp
            .set_from_policy(0, build_napot(0x80400000, 0x80000).unwrap(), pmpcfg::NAPOT);

        // Then we can mark the payload as protected
        self.protected = true;
    }

    fn is_policy_call(&mut self, ctx: &VirtContext) -> bool {
        ctx.get(Register::X17) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_EID
    }

    fn clear_register(&mut self, idx: usize, ctx: &mut VirtContext) -> bool {
        !(ctx.trap_info.get_cause() == MCause::EcallFromSMode && (10..18).contains(&idx))
    }

    fn restore_register(&mut self, idx: usize, ctx: &mut VirtContext) -> bool {
        !(ctx.trap_info.get_cause() == MCause::EcallFromSMode && (10..12).contains(&idx))
    }
}
