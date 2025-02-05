//! The default policy module, which enforces no policy.
use crate::policy::PolicyModule;

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
}
