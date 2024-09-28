//! Policy Modules
//!
//! This module holds the definitions of policy modules for Miralis.

use config_select::select_env;

use crate::virt::VirtContext;

mod default;
mod keystone;

pub type Policy = select_env!["MIRALIS_POLICY_NAME":
    "keystone" => keystone::KeystonePolicy
    _          => default::DefaultPolicy
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
/// By default, Miralis does not enforce isolation between the firmware and the rest of the system,
/// therefore without any policy the firmware is not restricted in any way.
/// The role of a policy module is to enforce a set of policies on the firmware, for instance
/// restricting which memory is accessible to the firmware, how which `ecall`s are intercepted.
pub trait PolicyModule {
    fn name() -> &'static str;
    fn ecall_from_firmware(ctx: &mut VirtContext) -> PolicyHookResult;
    fn ecall_from_payload(ctx: &mut VirtContext) -> PolicyHookResult;
}
