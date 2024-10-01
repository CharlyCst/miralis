//! The default policy module, which enforces no policy.

use miralis_core::abi;

use crate::arch::pmp::{build_napot, pmpcfg};
use crate::arch::{Arch, Architecture, Register};
use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::{RegisterContextGetter, VirtContext};

/// The default policy module, which doesn't enforce any isolation between the firmware and the
/// rest of the system.
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
        if ctx.get(Register::X16) == abi::MIRALIS_LOCK_FID {
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
        if ctx.get(Register::X16) == abi::MIRALIS_LOCK_FID {
            self.lock(mctx);
            ctx.pc += 4;
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }

    fn jump_from_payload_to_firmware(&mut self, ctx: &mut VirtContext) {
        if !self.protected {
            return;
        }

        // Step 1: Clear general purpose registers
        for i in 0..32 {
            self.general_register[i] = ctx.regs[i];
            // We don't clear ecall registers
            if !(10..18).contains(&i) {
                ctx.regs[i] = 0;
            }
        }
    }

    fn jump_from_firmware_to_payload(&mut self, ctx: &mut VirtContext) {
        if !self.protected {
            return;
        }

        // Step 1: Restore general purpose registers
        for i in 0..32 {
            // 10 & 11 are return registers
            if (10..12).contains(&i) {
                continue;
            }
            ctx.regs[i] = self.general_register[i];
        }
    }
}

impl ProtectPayloadPolicy {
    fn lock(&mut self, mctx: &mut MiralisContext) {
        // First set pmp entry protection
        mctx.pmp.set(
            mctx.devices.len() + 2,
            build_napot(0x80400000, 0x8000).unwrap(),
            pmpcfg::NAPOT,
        );

        unsafe {
            Arch::write_pmp(&mctx.pmp).flush();
        }

        // Then we can mark the payload as protected
        self.protected = true;
    }
}
