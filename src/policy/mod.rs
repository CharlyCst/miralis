//! Policy Modules
//!
//! This modules holds the definitions of policy modules for Miralis.

use config_select::select_env;

use crate::host::MiralisContext;
use crate::virt::VirtContext;

pub mod ace;
mod default;
mod keystone;
mod protect_payload;

pub type Policy = select_env!["MIRALIS_POLICY_NAME":
    "keystone" => keystone::KeystonePolicy
    "protect_payload" => protect_payload::ProtectPayloadPolicy
    _ => ace::AcePolicy
    // _          => default::DefaultPolicy
];

/// The result of a call into a policy hook function
///
/// A policy module can either overwrite standard Miralis emulation, or ignore an event and let
/// Miralis perform standard handling of the event. For instance the [DefaultPolicy] module which
/// does not enforce any restriction will return [PolicyHookResult::Ignore] for most of the policy
/// hooks.
#[derive(Debug, Clone, Copy)]
pub enum PolicyHookResult {
    /// Signal to Miralis that the policy module already handled the event, no further actions are
    /// required.
    Overwrite,
    /// Signal to Miralis that the policy module did not handle the event, Miralis will proceed
    /// normally.
    Ignore,
}

impl PolicyHookResult {
    pub fn overwrites(self) -> bool {
        match self {
            PolicyHookResult::Overwrite => true,
            PolicyHookResult::Ignore => false,
        }
    }
}

/// A Miralis firmware isolation policy
///
/// By default Miralis does not enforce isolation between the firmware and the rest of the system,
/// therefore without any policy the firmware is not restricted in any way.
/// The role of a policy module is to enforce a set of policies on the firmware, for instance
/// restricting which memory is accessible to the firmware, how which `ecall`s are intercepted.
pub trait PolicyModule {
    fn init(mctx: &mut MiralisContext, device_tree_blob_addr: usize) -> Self;
    fn name() -> &'static str;

    /// Handle an ecall from the virtualized firmware.
    ///
    /// Note that ecalls are a subset of traps.
    fn ecall_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        let _ = mctx;
        let _ = ctx;
        PolicyHookResult::Ignore
    }

    /// Handle an ecall from the payload.
    ///
    /// Note that ecalls are a subset of traps.
    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        let _ = mctx;
        let _ = ctx;
        PolicyHookResult::Ignore
    }

    /// Handle a trap from the virtualized firmware.
    fn trap_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        let _ = mctx;
        let _ = ctx;
        PolicyHookResult::Ignore
    }

    /// Handle a trap from the payload.
    fn trap_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        let _ = mctx;
        let _ = ctx;
        PolicyHookResult::Ignore
    }

    fn switch_from_payload_to_firmware(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext);

    fn switch_from_firmware_to_payload(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext);

    /// Callback for policy MSI.
    ///
    /// This function can be triggered across harts by sending a policy MSI. As such it can be used
    /// for synchronisation between multiple harts. Note that there is no guarantee that the MSI
    /// will be received without a delay, and as such a proper barrier must be used if
    /// synchronisation is critical for security.
    fn on_interrupt(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext);

    const NUMBER_PMPS: usize;
}
