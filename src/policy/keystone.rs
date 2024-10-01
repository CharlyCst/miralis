//! The Keystone security policy
//!
//! This policy module enforces the Keystone policies, i.e. it enables the creation of user-level
//! enclaves by leveraging PMP for memory isolation.

use crate::arch::Register;
use crate::host::MiralisContext;
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
    fn init() -> Self {
        KeystonePolicy {}
    }

    fn name() -> &'static str {
        "Keystone Policy"
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
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        let fid = ctx.get(Register::X17);
        if fid == KEYSTONE_FID {
            ctx.pc += 4;
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }

    fn jump_from_payload_to_firmware(&mut self, _: &mut VirtContext) {}

    fn jump_from_firmware_to_payload(&mut self, _: &mut VirtContext) {}
}
