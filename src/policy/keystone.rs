//! The Keystone security policy
//!
//! This policy module enforces the Keystone policies, i.e. it enables the creation of user-level
//! enclaves by leveraging PMP for memory isolation.

use crate::arch::Register;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::{RegisterContextGetter, VirtContext};
use crate::virt::RegisterContextSetter;

/// Keystone EID & FIDs
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sdk/include/shared/sm_call.h#L5C1-L6C1
const KEYSTONE_EID: usize = 0x08424b45;

// 1999-2999 are called by host
const CREATE_ENCLAVE_FID: usize = 2001;
const DESTROY_ENCLAVE_FID: usize = 2002;
const RUN_ENCLAVE_FID: usize = 2003;
const RESUME_ENCLAVE_FID: usize = 2005;

// 2999-3999 are called by enclave
const RANDOM_FID: usize = 3001;
const ATTEST_ENCLAVE_FID: usize = 3002;
const GET_SEALING_KEY_FID: usize = 3003;
const STOP_ENCLAVE_FID: usize = 3004;
const EXIT_ENCLAVE_FID: usize = 3006;

/// SBI return codes
///
/// See chapter 2 of https://www.scs.stanford.edu/~zyedidia/docs/riscv/riscv-sbi.pdf
enum ReturnCode{
    Success = 0,
    ErrFailed = -1,
    ErrNotSupported = -2,
}

/// The keystone policy module
///
/// See https://keystone-enclave.org/
pub struct KeystonePolicy {}

impl KeystonePolicy {
    fn handle_ecall(ctx: &mut VirtContext) {
        let fid = ctx.get(Register::X16);
        match fid {
            CREATE_ENCLAVE_FID => log::info!("Keystone: Create enclave"),
            DESTROY_ENCLAVE_FID => log::info!("Keystone: Destroy enclave"),
            RUN_ENCLAVE_FID => log::info!("Keystone: Run enclave"),
            RESUME_ENCLAVE_FID => log::info!("Keystone: Resume enclave"),
            RANDOM_FID => log::info!("Keystone: Random"),
            ATTEST_ENCLAVE_FID => log::info!("Keystone: Attest enclave"),
            GET_SEALING_KEY_FID => log::info!("Keystone: Get sealing key"),
            STOP_ENCLAVE_FID => log::info!("Keystone: Stop enclave"),
            EXIT_ENCLAVE_FID => log::info!("Keystone: Exit enclave"),
            _ => log::info!("Keystone: Unknown FID {}", fid)
        }
        ctx.set(Register::X10, ReturnCode::Success as usize);
        ctx.set(Register::X11, 0);
        ctx.pc += 4;
    }
}

impl PolicyModule for KeystonePolicy {
    fn name() -> &'static str {
        "Keystone Policy"
    }

    fn ecall_from_firmware(_ctx: &mut VirtContext) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn ecall_from_payload(ctx: &mut VirtContext) -> PolicyHookResult {
        let eid = ctx.get(Register::X17);
        if eid == KEYSTONE_EID {
            Self::handle_ecall(ctx);
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }
}
