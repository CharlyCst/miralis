//! The Keystone security policy
//!
//! This policy module enforces the Keystone policies, i.e. it enables the creation of user-level
//! enclaves by leveraging PMP for memory isolation.

#![allow(unused)]
todo!("Remove the above warning");

use crate::arch::Register;
use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::RegisterContextSetter;
use crate::{RegisterContextGetter, VirtContext};
use core::ptr;


/// Keystone parameters
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sm/src/platform/generic/platform.h#L11
const ENCL_MAX: usize = 16; // Maximum number of enclaves
const ENCLAVE_REGION_MAX: usize = 8;


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


/// Keystone return codes
///
/// See https://github.com/keystone-enclave/keystone/blob/master/sdk/include/shared/sm_err.h
enum ReturnCode {
    Success = 0,
    NoFreeResources = 100013,
    NotImplemented = 100100,
}


/// Enclave definitions
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sm/src/enclave.h
#[derive(Default)]
enum EnclaveState {
    #[default]
    Invalid = -1,
    Destroying = 0,
    Allocated,
    Fresh,
    Stopped,
    Running,
}

#[derive(Default)]
enum EnclaveRegionType {
    #[default]
    Invalid,
    EPM,
    UTM,
    Other,
}

#[derive(Default)]
struct RuntimeParams {
    dram_base: usize,
    dram_size: usize,
    runtime_base: usize,
    user_base: usize,
    free_base: usize,
    untrusted_base: usize,
    untrusted_size: usize,
    free_requested: usize,
}

#[derive(Default)]
struct EnclaveRegion {
    pmp_rid: i32,
    enclave_type: EnclaveRegionType,
}

#[derive(Default)]
struct Enclave {
    eid: usize, // Enclave ID
    satp: usize, // Enclave's page table base
    state: EnclaveState, // Global state of the enclave
    regions: [EnclaveRegion; ENCLAVE_REGION_MAX], // Physical memory regions for this enclave
    // TODO: Add fields related to hash/signature
    params: RuntimeParams,
    // TODO: Add fields related to multi-threading
    // TODO: Add fields related to platform specific data
}


/// The keystone policy module
///
/// See https://keystone-enclave.org/
#[derive(Default)]
pub struct KeystonePolicy {
    enclaves: [Enclave; ENCL_MAX],
}

impl KeystonePolicy {
    /// Allocate an enclave slot and returns the index of the newly allocated enclave
    fn allocate_enclave(&mut self) -> Result<usize, ReturnCode> {
        // TODO: Add mutex to protect the enclave array
        for i in 0..ENCL_MAX {
            let enclave = &mut self.enclaves[i];
            if let EnclaveState::Invalid = enclave.state {
                enclave.state = EnclaveState::Allocated;
                return Ok(i);
            }
        }

        Err(ReturnCode::NoFreeResources)
    }

    fn create_enclave(&mut self, ctx: &mut VirtContext) -> ReturnCode {
        log::debug!("Keystone: Create enclave");

        // Read the arguments passed to create_enclave
        #[repr(C)]
        struct KeystoneRegion {
            paddr: usize,
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
        // TODO: Check if args is valid (see enclave.c line 351)

        // Create params
        let params = RuntimeParams {
            dram_base: args.epm_region.paddr,
            dram_size: args.epm_region.size,
            runtime_base: args.runtime_paddr,
            user_base: args.user_paddr,
            free_base: args.free_paddr,
            untrusted_base: args.upm_region.paddr,
            untrusted_size: args.upm_region.size,
            free_requested: args.free_requested,
        };

        // Find a free enclave slot
        let enclave_index = match Self::allocate_enclave(self) {
            Ok(index) => index,
            Err(code) => return code,
        };

        ReturnCode::Success
    }

    fn destroy_enclave(&mut self, ctx: &mut VirtContext) -> ReturnCode {
        log::debug!("Keystone: Destroy enclave");
        ReturnCode::NotImplemented
    }
}

/// To check how ecalls are handled, see https://github.com/riscv-software-src/opensbi/blob/2ffa0a153d804910c20b82974bfe2dedcf35a777/lib/sbi/sbi_ecall.c#L98
impl PolicyModule for KeystonePolicy {
    fn init() -> Self {
        Self::default()
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
        let eid = ctx.get(Register::X17);
        let fid = ctx.get(Register::X16);
        if eid != KEYSTONE_EID {
            return PolicyHookResult::Ignore;
        }

        let return_code: ReturnCode = match fid {
            CREATE_ENCLAVE_FID => Self::create_enclave(self, ctx),
            DESTROY_ENCLAVE_FID => Self::destroy_enclave(self, ctx),
            _ => {
                log::debug!("Keystone: Unknown FID {}", fid);
                ReturnCode::NotImplemented
            }
        };

        ctx.set(Register::X10, return_code as usize);
        ctx.pc += 4;

        PolicyHookResult::Overwrite
    }

    fn switch_from_payload_to_firmware(&mut self, _: &mut VirtContext) {}

    fn switch_from_firmware_to_payload(&mut self, _: &mut VirtContext) {}
}
