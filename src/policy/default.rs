//! The default policy module, which enforces no policy.

use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::VirtContext;

/// The default policy module, which doesn't enforce any isolation between the firmware and the
/// rest of the system.
pub struct DefaultPolicy {}

impl PolicyModule for DefaultPolicy {
    fn name() -> &'static str {
        "Default Policy"
    }

    fn ecall_from_firmware(_ctx: &mut VirtContext) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn ecall_from_payload(_ctx: &mut VirtContext) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }
}
