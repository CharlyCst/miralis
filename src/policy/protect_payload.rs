//! The protect payload policy, it protects the payload.

use miralis_core::abi_protect_payload;

use crate::arch::pmp::pmpcfg;
use crate::arch::{MCause, Register};
use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::{RegisterContextGetter, VirtContext};

/// The protect payload policy module, which allow the payload to protect himself from the firmware at some point in time and enfore a boundary between the two components.
pub struct ProtectPayloadPolicy {
    protected: bool,
    general_register: [usize; 32],
    last_cause: MCause,
}

impl PolicyModule for ProtectPayloadPolicy {
    fn init() -> Self {
        ProtectPayloadPolicy {
            protected: false,
            general_register: [0; 32],
            // It is important to let the first mode be EcallFromSMode as the firmware passes some information to the OS.
            // Setting this last_cause allows to pass the arguments during the first call.
            last_cause: MCause::EcallFromSMode,
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
            log::info!("Locking payload from firmware");
            self.lock(mctx, ctx);
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
            log::info!("Locking payload from payload");
            self.lock(mctx, ctx);
            ctx.pc += 4;
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        if !self.protected {
            return;
        }

        // Restore general purpose registers
        for i in 0..32 {
            self.general_register[i] = ctx.regs[i];
            // We don't clear ecall registers
            if self.clear_register(i, ctx) {
                ctx.regs[i] = 0;
            }
        }

        // Lock memory
        mctx.pmp
            .set_from_policy(0, 0x80400000 / 4, pmpcfg::INACTIVE);
        mctx.pmp.set_from_policy(1, usize::MAX / 4, pmpcfg::TOR);

        self.last_cause = ctx.trap_info.get_cause();
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        if !self.protected {
            return;
        }

        // Restore general purpose registers
        for i in 0..32 {
            // 10 & 11 are return registers
            if self.restore_register(i, ctx) {
                ctx.regs[i] = self.general_register[i];
            }
        }

        // Unlock memory
        mctx.pmp
            .set_from_policy(0, 0x80400000 / 4, pmpcfg::INACTIVE);
        mctx.pmp
            .set_from_policy(1, usize::MAX / 4, pmpcfg::TOR | pmpcfg::RWX);
    }

    const NUMBER_PMPS: usize = 2;
}

impl ProtectPayloadPolicy {
    fn lock(&mut self, _mctx: &mut MiralisContext, _ctx: &mut VirtContext) {
        self.protected = true;
    }

    fn is_policy_call(&mut self, ctx: &VirtContext) -> bool {
        ctx.get(Register::X17) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_EID
    }

    fn clear_register(&mut self, idx: usize, ctx: &mut VirtContext) -> bool {
        !(10..18).contains(&idx) || ctx.trap_info.get_cause() != MCause::EcallFromSMode
    }

    fn restore_register(&mut self, idx: usize, _ctx: &mut VirtContext) -> bool {
        !(10..12).contains(&idx) || self.last_cause != MCause::EcallFromSMode
    }
}
