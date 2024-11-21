//! The default policy module, which enforces no policy.

use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::VirtContext;

/// The default policy module, which doesn't enforce any isolation between the firmware and the
/// rest of the system.
pub struct DefaultPolicy {}

impl PolicyModule for DefaultPolicy {
    fn init(_mctx: &mut MiralisContext, _device_tree_blob_addr: usize) -> Self {
        DefaultPolicy {}
    }

    fn name() -> &'static str {
        "Default Policy"
    }

    fn ecall_from_firmware(
        &mut self,
        _mctx: &mut MiralisContext,
        _ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn ecall_from_payload(
        &mut self,
        _mctx: &mut MiralisContext,
        _ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn switch_from_payload_to_firmware(&mut self, _: &mut VirtContext, _: &mut MiralisContext) {}

    fn switch_from_firmware_to_payload(&mut self, _: &mut VirtContext, _: &mut MiralisContext) {}

    fn on_interrupt(&mut self, _ctx: &mut VirtContext, _mctx: &mut MiralisContext) {}

    const NUMBER_PMPS: usize = 0;
}
