//! Miralis Modules
//!
//! This file defines the Miralis module interface, and hosts the [MainModule] struct that is generated
//! from combining all modules selected at compile time.

use module_macro::{build_modules, for_each_module};

use crate::arch::{Arch, Architecture, Csr};
use crate::config::PLATFORM_BOOT_HART_ID;
use crate::host::MiralisContext;
use crate::virt::{ExecutionMode, VirtContext};

// ———————————————————————————— Module Interface ———————————————————————————— //

/// The Miralis module interface
///
/// By default Miralis does nothing more than virtualizing a RISC-V firmware (M-mode program), any
/// extra amount of functionalities, such as intercepting firmware traps or enforcing isolation
/// policies, has to be done through modules.
///
/// The [Module] trait exposes functions (called hooks) that are called by Miralis during firmware
/// virtualization and allow extending the functionalities of Miralis. For instance, modules can be
/// use to isolate the firmware from the OS by protecting OS memory and hiding registers on world
/// switches. Modules can also be used to monitor firmware behavior.
///
/// All module hooks are optional, if not implemented by a module they will simply be ignored.
pub trait Module {
    /// The name of the module.
    const NAME: &'static str;

    /// The number of PMP entries used by the module.
    const NUMBER_PMPS: usize = 0;

    /// The initialization function of the module, called by Miralis at boot time.
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
    ) -> ModuleAction {
        let _ = mctx;
        let _ = ctx;
        ModuleAction::Ignore
    }

    /// Handle an ecall from the payload.
    ///
    /// Note that ecalls are a subset of traps.
    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        let _ = mctx;
        let _ = ctx;
        ModuleAction::Ignore
    }

    /// Handle a trap from the virtualized firmware.
    fn trap_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        let _ = mctx;
        let _ = ctx;
        ModuleAction::Ignore
    }

    /// Handle a trap from the payload.
    fn trap_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        let _ = mctx;
        let _ = ctx;
        ModuleAction::Ignore
    }

    /// Interpose on the switch from payload to firmware mode.
    ///
    /// Note: Miralis will proceed with the switch anyway, this does not provide an option for
    /// aborting the switch.
    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        let _ = ctx;
        let _ = mctx;
    }

    /// Interpose on the switch from firmware to payload mode.
    ///
    /// Note: Miralis will proceed with the switch anyway, this does not provide an option for
    /// aborting the switch.
    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        let _ = ctx;
        let _ = mctx;
    }

    /// Interpose after the next mode has been decided, but before world switch if any.
    ///
    /// This module hook can be useful for collecting statistics about traps to firmware, such as
    /// causes and frequency.
    fn decided_next_exec_mode(
        &mut self,
        ctx: &mut VirtContext,
        previous_mode: ExecutionMode,
        next_mode: ExecutionMode,
    ) {
        let _ = ctx;
        let _ = previous_mode;
        let _ = next_mode;
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

    /// Hook called before shutting down.
    fn on_shutdown(&mut self) {}
}

/// Outcome of a module hook.
///
/// Some module hook can be used to overwrite the standard behavior, for instance exposing new
/// ecalls. Such hooks returns a [ModuleAction] which indicates whether Miralis should handle the
/// event ([ModuleAction::Ignore]) or not ([ModuleAction::Overwrite]).
#[derive(Debug, Clone, Copy)]
pub enum ModuleAction {
    /// Signal to Miralis that the module already handled the event, no further actions are
    /// required.
    Overwrite,
    /// Signal to Miralis that the module did not handle the event, Miralis will proceed
    /// normally.
    Ignore,
}

impl ModuleAction {
    /// True if the result of the action is [ModuleAction::Overwrite].
    pub fn overwrites(self) -> bool {
        match self {
            ModuleAction::Overwrite => true,
            ModuleAction::Ignore => false,
        }
    }
}

// —————————————————————————————— Main Module ——————————————————————————————— //
// The MainModule is defined using a proc macro, this is required to choose   //
// enabled modules at compile time.                                           //
// Further, we use a second proc macro to iterate over all modules included   //
// at compile time to implement the MainModule.                               //
//                                                                            //
// When adding new modules, the `build_modules` macro should be updated to    //
// indicate the path of the added modules.                                    //
// —————————————————————————————————————————————————————————————————————————— //

build_modules! {
    "keystone" => crate::policy::keystone::KeystonePolicy
    "protect_payload" => crate::policy::protect_payload::ProtectPayloadPolicy
    "offload" => crate::policy::offload::OffloadPolicy
    "exit_counter" => crate::benchmark::counter::CounterBenchmark
    "exit_counter_per_cause" => crate::benchmark::counter_per_cause::CounterPerMcauseBenchmark
    "boot_counter" => crate::benchmark::boot::BootBenchmark
}

impl Module for MainModule {
    const NAME: &'static str = "Main Module";

    /// The total number of PMPs is computed as the sum of the number of PMPs for each selected
    /// module.
    const NUMBER_PMPS: usize = MainModule::TOTAL_PMPS;

    fn init() -> Self {
        let module = for_each_module!(
            Self {
                $($module: Module::init()),*
            }
        );

        // We log the lists of modules on the boot hart only to avoid cluttering the screen too
        // much. Models are the same on all cores.
        if Arch::read_csr(Csr::Mhartid) == PLATFORM_BOOT_HART_ID {
            module.log_all_modules();
        }

        module
    }

    fn ecall_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                if self.$module.ecall_from_firmware(mctx, ctx).overwrites() {
                    return ModuleAction::Overwrite
                }
            )*
        );

        ModuleAction::Ignore
    }

    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                if self.$module.ecall_from_payload(mctx, ctx).overwrites() {
                    return ModuleAction::Overwrite
                }
            )*
        );

        ModuleAction::Ignore
    }

    fn trap_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                if self.$module.trap_from_firmware(mctx, ctx).overwrites() {
                    return ModuleAction::Overwrite
                }
            )*
        );

        ModuleAction::Ignore
    }

    /// Handle a trap from the payload.
    fn trap_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        // Remove "unused" warning when building with no modules
        let _ = &mctx;
        let _ = &ctx;

        for_each_module!(
            $(
                if self.$module.trap_from_payload(mctx, ctx).overwrites() {
                    return ModuleAction::Overwrite
                }
            )*
        );

        ModuleAction::Ignore
    }

    fn decided_next_exec_mode(
        &mut self,
        ctx: &mut VirtContext,
        previous_mode: ExecutionMode,
        next_mode: ExecutionMode,
    ) {
        // Remove "unused" warning when building with no modules
        let _ = &ctx;
        let _ = &previous_mode;
        let _ = &next_mode;

        for_each_module!(
            $(
                self.$module.decided_next_exec_mode(ctx, previous_mode, next_mode);
            )*
        );
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

    fn on_shutdown(&mut self) {
        for_each_module!(
            $(
                self.$module.on_shutdown();
            )*
        );
    }
}

impl MainModule {
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
