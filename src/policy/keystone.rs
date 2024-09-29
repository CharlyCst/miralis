//! The Keystone security policy
//!
//! This policy module enforces the Keystone policies, i.e. it enables the creation of user-level
//! enclaves by leveraging PMP for memory isolation.

use crate::arch::Register;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::RegisterContextSetter;
use crate::{RegisterContextGetter, VirtContext};
use core::ptr;

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
/// See https://github.com/keystone-enclave/keystone/blob/master/sdk/include/shared/sm_err.h
const ERR_SM_ENCLAVE_SUCCESS: usize = 0;
const ERR_SM_NOT_IMPLEMENTED: usize = 100100;

/// The keystone policy module
///
/// See https://keystone-enclave.org/
pub struct KeystonePolicy {}

impl KeystonePolicy {
    fn create_enclave(ctx: &mut VirtContext) -> usize {
        log::debug!("Keystone: Create enclave");

        // Read the arguments passed to create_enclave
        #[repr(C)]
        struct KeystoneRegion {
            addr: usize,
            size: usize,
        }

        #[repr(C)]
        struct CreateArgs {
            epm_region: KeystoneRegion, // Enclave region
            upm_region: KeystoneRegion, // Untrusted region

            runtime_paddr: usize,
            user_paddr: usize,
            free_paddr: usize,
            free_requested: usize,
        }

        // TODO: We should validate that the memory pointed by a0 is valid, and well aligned
        let args = unsafe { ptr::read(ctx.get(Register::X10) as *const CreateArgs) };

        ERR_SM_ENCLAVE_SUCCESS
    }

    fn destroy_enclave(ctx: &mut VirtContext) -> usize {
        log::debug!("Keystone: Destroy enclave");
        ERR_SM_ENCLAVE_SUCCESS
    }
}

/// To check how ecalls are handled, see https://github.com/riscv-software-src/opensbi/blob/2ffa0a153d804910c20b82974bfe2dedcf35a777/lib/sbi/sbi_ecall.c#L98
impl PolicyModule for KeystonePolicy {
    fn name() -> &'static str {
        "Keystone Policy"
    }

    fn ecall_from_firmware(_ctx: &mut VirtContext) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn ecall_from_payload(ctx: &mut VirtContext) -> PolicyHookResult {
        let eid = ctx.get(Register::X17);
        let fid = ctx.get(Register::X16);
        if eid != KEYSTONE_EID {
            return PolicyHookResult::Ignore;
        }

        let err_code: usize = match fid {
            CREATE_ENCLAVE_FID => Self::create_enclave(ctx),
            DESTROY_ENCLAVE_FID => Self::destroy_enclave(ctx),
            _ => {
                log::debug!("Keystone: Unknown FID {}", fid);
                ERR_SM_NOT_IMPLEMENTED
            }
        };

        ctx.set(Register::X10, err_code);
        ctx.pc += 4;

        PolicyHookResult::Overwrite
    }
}
