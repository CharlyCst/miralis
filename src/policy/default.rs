//! The default policy module, which enforces no policy.

use miralis_core::abi;

use crate::arch::Register;
use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::{RegisterContextGetter, VirtContext};

/// The default policy module, which doesn't enforce any isolation between the firmware and the
/// rest of the system.
pub struct DefaultPolicy {}

impl PolicyModule for DefaultPolicy {
    fn init() -> Self {
        DefaultPolicy {}
    }

    fn name() -> &'static str {
        "Default Policy"
    }

    fn ecall_from_firmware(
        &mut self,
        _mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        if ctx.get(Register::X16) == abi::MIRALIS_LOCK_FID {
            ctx.pc += 4;
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }

    fn ecall_from_payload(
        &mut self,
        _mctx: &mut MiralisContext,
        _ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn jump_from_payload_to_firmware(&mut self, _: &mut VirtContext) {}

    fn jump_from_firmware_to_payload(&mut self, _: &mut VirtContext) {}
}
