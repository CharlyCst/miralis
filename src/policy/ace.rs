//! The ace security policy. This policy colocates the ACE security monitor (https://github.com/IBM/ACE-RISCV) with Miralis such that the Firmware can be untrusted..

use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::VirtContext;

/// The ACE policy, which colocates the ACE security monitor with Miralis
pub struct AcePolicy {}

impl PolicyModule for AcePolicy {
    fn init() -> Self {
        AcePolicy {}
    }

    fn name() -> &'static str {
        "ACE policy"
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

    const NUMBER_PMPS: usize = 2;
}
