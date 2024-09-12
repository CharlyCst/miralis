//! The Keystone security policy
//!
//! This policy module enforces the Keystone policies, i.e. it enables the creation of user-level
//! enclaves by leveraging PMP for memory isolation.

use crate::arch::Register;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::{RegisterContextGetter, VirtContext};

/// The Keystone FID
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sdk/include/shared/sm_call.h#L5C47-L5C57
const KEYSTONE_FID: usize = 0x08424b45;

/// The keystone policy module
///
/// See https://keystone-enclave.org/
pub struct KeystonePolicy {}

impl PolicyModule for KeystonePolicy {
    fn name() -> &'static str {
        "Keystone Policy"
    }

    fn ecall_from_firmware(_ctx: &mut VirtContext) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn ecall_from_payload(ctx: &mut VirtContext) -> PolicyHookResult {
        let fid = ctx.get(Register::X17);
        if fid == KEYSTONE_FID {
            // TODO: do something with the ecall
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }
}
