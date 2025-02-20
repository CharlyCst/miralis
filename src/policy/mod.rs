//! Policy Modules
//!
//! This modules holds the definitions of policy modules for Miralis.

use module_macro::{build_modules, for_each_module};

use crate::host::MiralisContext;
use crate::virt::VirtContext;

mod keystone;
pub mod offload;
mod protect_payload;

pub type Policy = Modules;

build_modules! {
    "keystone" => keystone::KeystonePolicy
    "protect_payload" => protect_payload::ProtectPayloadPolicy
    "offload" => offload::OffloadPolicy
}

impl PolicyModule for Modules {
    const NAME: &'static str = "Main Module";

    fn init() -> Self {
        let module = for_each_module!(
            Self {
                $($module: PolicyModule::init()),*
            }
        );
        module.log_all_modules();
        module
    }

    fn ecall_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                if self.$module.ecall_from_firmware(mctx, ctx).overwrites() {
                    return PolicyHookResult::Overwrite
                }
            )*
        );

        PolicyHookResult::Ignore
    }

    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                if self.$module.ecall_from_payload(mctx, ctx).overwrites() {
                    return PolicyHookResult::Overwrite
                }
            )*
        );

        PolicyHookResult::Ignore
    }

    fn trap_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                if self.$module.trap_from_firmware(mctx, ctx).overwrites() {
                    return PolicyHookResult::Overwrite
                }
            )*
        );

        PolicyHookResult::Ignore
    }

    /// Handle a trap from the payload.
    fn trap_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                if self.$module.trap_from_payload(mctx, ctx).overwrites() {
                    return PolicyHookResult::Overwrite
                }
            )*
        );

        PolicyHookResult::Ignore
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                self.$module.switch_from_payload_to_firmware(ctx, mctx);
            )*
        );
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                self.$module.switch_from_firmware_to_payload(ctx, mctx);
            )*
        );
    }

    fn on_interrupt(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext) {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                self.$module.on_interrupt(ctx, mctx);
            )*
        );
    }

    /// The total number of PMPs is computed as the sum of the number of PMPs for each selected
    /// module.
    const NUMBER_PMPS: usize = Modules::TOTAL_PMPS;
}

impl Modules {
    /// Display the list of all installed module.
    fn log_all_modules(&self) {
        // Count the number of modules
        #[allow(unused_mut)]
        let mut nb_modules: usize = 0;
        for_each_module!(
            $(
                nb_modules += 1;
            )*
        );

        if nb_modules > 0 {
            log::info!("Installed {} modules:", nb_modules);
            for_each_module!(
                $(
                    log::info!("  - {}", self.$module.name());
                )*
            );
        } else {
            log::info!("No module installed")
        }
    }
}

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
    const NUMBER_PMPS: usize = 0;
    const NAME: &'static str;

    fn init() -> Self;

    fn name(&self) -> &'static str {
        Self::NAME
    }

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

    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        let _ = ctx;
        let _ = mctx;
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        let _ = ctx;
        let _ = mctx;
    }

    /// Callback for policy MSI.
    ///
    /// This function can be triggered across harts by sending a policy MSI. As such it can be used
    /// for synchronisation between multiple harts. Note that there is no guarantee that the MSI
    /// will be received without a delay, and as such a proper barrier must be used if
    /// synchronisation is critical for security.
    fn on_interrupt(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext) {
        let _ = ctx;
        let _ = mctx;
    }
}
