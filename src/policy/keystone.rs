//! The Keystone security policy
//!
//! This policy module enforces the Keystone policies, i.e. it enables the creation of user-level
//! enclaves by leveraging PMP for memory isolation.
//! TODO: Remove allow(unused)

use core::ptr;

use crate::arch::{parse_mpp_return_mode, Arch, Architecture, Csr, Register};
use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::RegisterContextSetter;
use crate::{RegisterContextGetter, VirtContext};

/// Keystone parameters
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sm/src/platform/generic/platform.h#L11
const ENCL_MAX: usize = 16; // Maximum number of enclaves
#[allow(unused)]
const ENCLAVE_REGION_MAX: usize = 8;

/// Keystone EID & FIDs
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sdk/include/shared/sm_call.h#L5C1-L6C1
mod sbi {
    #![allow(unused)]
    pub const KEYSTONE_EID: usize = 0x08424b45;

    // 1999-2999 are called by host
    pub const CREATE_ENCLAVE_FID: usize = 2001;
    pub const DESTROY_ENCLAVE_FID: usize = 2002;
    pub const RUN_ENCLAVE_FID: usize = 2003;
    pub const RESUME_ENCLAVE_FID: usize = 2005;

    // 2999-3999 are called by enclave
    pub const RANDOM_FID: usize = 3001;
    pub const ATTEST_ENCLAVE_FID: usize = 3002;
    pub const GET_SEALING_KEY_FID: usize = 3003;
    pub const STOP_ENCLAVE_FID: usize = 3004;
    pub const EXIT_ENCLAVE_FID: usize = 3006;
}

/// Keystone return codes
///
/// See https://github.com/keystone-enclave/keystone/blob/master/sdk/include/shared/sm_err.h
enum ReturnCode {
    Success = 0,
    IllegalArgument = 100008,
    NoFreeResources = 100013,
    NotImplemented = 100100,
}

/// Enclave definitions
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sm/src/enclave.h
#[derive(Default)]
#[allow(unused)]
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
#[allow(unused)]
enum EnclaveRegionType {
    #[default]
    Invalid,
    Epm,
    Utm,
    Other,
}

#[derive(Default)]
#[allow(unused)]
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
struct Enclave {
    eid: usize,          // Enclave ID
    state: EnclaveState, // Global state of the enclave
    params: RuntimeParams,
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
        for i in 0..ENCL_MAX {
            let enclave = &mut self.enclaves[i];
            if let EnclaveState::Invalid = enclave.state {
                return Ok(i);
            }
        }

        Err(ReturnCode::NoFreeResources)
    }

    fn create_enclave(&mut self, ctx: &mut VirtContext) -> ReturnCode {
        #[repr(C)]
        struct CreateArgs {
            epm_paddr: usize, // Enclave region
            epm_size: usize,
            utm_paddr: usize, // Untrusted region
            utm_size: usize,
            runtime_paddr: usize,
            user_paddr: usize,
            free_paddr: usize,
            free_requested: usize,
        }

        // Copy the arguments from the S-mode virtual memory to the M-mode physical memory
        const ARGS_SIZE: usize = size_of::<CreateArgs>();
        let src = ctx.get(Register::X10) as *const u8;
        let mut dest: [u8; ARGS_SIZE] = [0; ARGS_SIZE];
        let mode = parse_mpp_return_mode(Arch::read_csr(Csr::Mstatus));
        let res = unsafe { Arch::read_bytes_from_mode(src, &mut dest, mode) };
        if res.is_err() {
            return ReturnCode::IllegalArgument;
        }

        // TODO: Check if args is valid (see enclave.c line 351)
        let args = unsafe { ptr::read(dest.as_ptr() as *const CreateArgs) };

        // Find a free enclave slot and initialize it
        let eid = match Self::allocate_enclave(self) {
            Ok(index) => index,
            Err(code) => return code,
        };

        self.enclaves[eid].eid = eid;
        self.enclaves[eid].state = EnclaveState::Allocated;
        self.enclaves[eid].params = RuntimeParams {
            dram_base: args.epm_paddr,
            dram_size: args.epm_size,
            runtime_base: args.runtime_paddr,
            user_base: args.user_paddr,
            free_base: args.free_paddr,
            untrusted_base: args.utm_paddr,
            untrusted_size: args.utm_size,
            free_requested: args.free_requested,
        };

        ReturnCode::Success
    }

    fn destroy_enclave(&mut self, _ctx: &mut VirtContext) -> ReturnCode {
        log::debug!("Keystone: Destroy enclave");
        ReturnCode::NotImplemented
    }
}

/// To check how ecalls are handled, see https://github.com/riscv-software-src/opensbi/blob/2ffa0a153d804910c20b82974bfe2dedcf35a777/lib/sbi/sbi_ecall.c#L98
impl PolicyModule for KeystonePolicy {
    fn init(_mctx: &mut MiralisContext, _device_tree_blob_addr: usize) -> Self {
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
        if eid != sbi::KEYSTONE_EID {
            return PolicyHookResult::Ignore;
        }

        let return_code: ReturnCode = match fid {
            sbi::CREATE_ENCLAVE_FID => Self::create_enclave(self, ctx),
            sbi::DESTROY_ENCLAVE_FID => Self::destroy_enclave(self, ctx),
            _ => {
                log::debug!("Keystone: Unknown FID {}", fid);
                ReturnCode::NotImplemented
            }
        };

        ctx.set(Register::X10, return_code as usize);
        ctx.pc += 4;

        PolicyHookResult::Overwrite
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        _ctx: &mut VirtContext,
        _mctx: &mut MiralisContext,
    ) {
        // TODO: Implement
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        _ctx: &mut VirtContext,
        _mctx: &mut MiralisContext,
    ) {
        // TODO: Implement
    }

    fn on_interrupt(&mut self, _ctx: &mut VirtContext, _mctx: &mut MiralisContext) {}

    const NUMBER_PMPS: usize = 0;
}
