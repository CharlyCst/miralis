//! Miralis
//!
//! The Miralis library, which needs to be embedded into an executable.
//! This library exposes two main functions: [init] and [main_loop].

// Mark the crate as no_std and no_main, but only when not running tests.
// We need both std and main to be able to run tests in user-space on the host architecture.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

pub mod arch;
pub mod benchmark;
pub mod config;
pub mod debug;
pub mod decoder;
pub mod device;
pub mod driver;
pub mod host;
pub mod logger;
pub mod platform;
pub mod policy;
pub mod utils;
pub mod virt;

use arch::{Arch, Architecture, Csr, Register};
use host::MiralisContext;
pub use platform::init;
use platform::{Plat, Platform};
use policy::{Policy, PolicyModule};
use virt::traits::*;
use virt::{ExecutionMode, ExitResult, VirtContext};

use crate::arch::write_pmp;
use crate::benchmark::{Benchmark, BenchmarkModule, Counter, Scope};

/// The virtuam firmware monitor main loop.
///
/// Runs the firmware and payload in a loop, handling the traps and interrupts and switching world
/// when required..
///
/// # Safety
///
/// This function will start by passing control to the firmware. The hardware must have
/// been initialized properly (including calling `miralis::init` and loading the firmware.
pub unsafe fn main_loop(ctx: &mut VirtContext, mctx: &mut MiralisContext, policy: &mut Policy) {
    loop {
        Benchmark::start_interval_counters(Scope::RunVCPU);

        unsafe { Arch::run_vcpu(ctx) };

        Benchmark::stop_interval_counters(Scope::RunVCPU);
        Benchmark::start_interval_counters(Scope::HandleTrap);

        if handle_trap(ctx, mctx, policy) == ExitResult::Donne {
            return;
        }

        Benchmark::stop_interval_counters(Scope::HandleTrap);
        Benchmark::increment_counter(Counter::TotalExits);
    }
}

fn handle_trap(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
    policy: &mut Policy,
) -> ExitResult {
    if logger::trace_enabled!() {
        log_ctx(ctx);
    }

    if let Some(max_exit) = config::MAX_FIRMWARE_EXIT {
        if ctx.nb_exits + 1 >= max_exit {
            log::error!("Reached maximum number of exits: {}", ctx.nb_exits);
            Plat::exit_failure();
        }
    }

    if ctx.trap_info.is_from_mmode() {
        // Trap comes from M mode: Miralis
        handle_miralis_trap(ctx);
        return ExitResult::Continue;
    }

    // Perform emulation
    let exec_mode = ctx.mode.to_exec_mode();

    // Keep track of the number of exit
    ctx.nb_exits += 1;
    let result = match exec_mode {
        ExecutionMode::Firmware => ctx.handle_firmware_trap(mctx, policy),
        ExecutionMode::Payload => ctx.handle_payload_trap(mctx, policy),
    };

    if exec_mode == ExecutionMode::Firmware {
        Benchmark::increment_counter(Counter::FirmwareExits);
    }

    if exec_mode != ctx.mode.to_exec_mode() {
        Benchmark::increment_counter(Counter::WorldSwitches);
    }

    // Inject interrupts if required
    ctx.check_and_inject_interrupts();

    // Check for execution mode change
    match (exec_mode, ctx.mode.to_exec_mode()) {
        (ExecutionMode::Firmware, ExecutionMode::Payload) => {
            logger::debug!("Execution mode: Firmware -> Payload");
            unsafe { ctx.switch_from_firmware_to_payload(mctx) };
            policy.switch_from_firmware_to_payload(ctx, mctx);

            unsafe {
                // Commit the PMP to hardware
                write_pmp(&mctx.pmp).flush();
            }
        }
        (ExecutionMode::Payload, ExecutionMode::Firmware) => {
            logger::debug!(
                "Execution mode: Payload -> Firmware ({:?})",
                ctx.trap_info.get_cause()
            );
            unsafe { ctx.switch_from_payload_to_firmware(mctx) };
            policy.switch_from_payload_to_firmware(ctx, mctx);

            unsafe {
                // Commit the PMP to hardware
                write_pmp(&mctx.pmp).flush();
            }
        }
        _ => {} // No execution mode transition
    }

    result
}

/// Handle the trap coming from miralis
fn handle_miralis_trap(ctx: &mut VirtContext) {
    let trap = &ctx.trap_info;
    log::error!("Unexpected trap while executing Miralis");
    log::error!("  cause:   {} ({:?})", trap.mcause, trap.get_cause());
    log::error!("  mepc:    0x{:x}", trap.mepc);
    log::error!("  mtval:   0x{:x}", trap.mtval);
    log::error!("  mstatus: 0x{:x}", trap.mstatus);
    log::error!("  mip:     0x{:x}", trap.mip);

    todo!("Miralis trap handler entered");
}

// —————————————————————————————— Debug Helper —————————————————————————————— //

/// Log the current context using the trace log level.
fn log_ctx(ctx: &VirtContext) {
    let trap_info = &ctx.trap_info;
    log::trace!(
        "Trapped on hart {}:  {:?}",
        ctx.hart_id,
        ctx.trap_info.get_cause()
    );
    log::trace!(
        "  mstatus: 0x{:<16x} mepc: 0x{:x}",
        trap_info.mstatus,
        trap_info.mepc
    );
    log::trace!(
        "  mtval:   0x{:<16x} exits: {}  {:?}-mode",
        ctx.trap_info.mtval,
        ctx.nb_exits,
        ctx.mode
    );
    log::trace!(
        "  x1  {:<16x}  x2  {:<16x}  x3  {:<16x}",
        ctx.get(Register::X1),
        ctx.get(Register::X2),
        ctx.get(Register::X3)
    );
    log::trace!(
        "  x4  {:<16x}  x5  {:<16x}  x6  {:<16x}",
        ctx.get(Register::X4),
        ctx.get(Register::X5),
        ctx.get(Register::X6)
    );
    log::trace!(
        "  x7  {:<16x}  x8  {:<16x}  x9  {:<16x}",
        ctx.get(Register::X7),
        ctx.get(Register::X8),
        ctx.get(Register::X9)
    );
    log::trace!(
        "  x10 {:<16x}  x11 {:<16x}  x12 {:<16x}",
        ctx.get(Register::X10),
        ctx.get(Register::X11),
        ctx.get(Register::X12)
    );
    log::trace!(
        "  x13 {:<16x}  x14 {:<16x}  x15 {:<16x}",
        ctx.get(Register::X13),
        ctx.get(Register::X14),
        ctx.get(Register::X15)
    );
    log::trace!(
        "  x16 {:<16x}  x17 {:<16x}  x18 {:<16x}",
        ctx.get(Register::X16),
        ctx.get(Register::X17),
        ctx.get(Register::X18)
    );
    log::trace!(
        "  x19 {:<16x}  x20 {:<16x}  x21 {:<16x}",
        ctx.get(Register::X19),
        ctx.get(Register::X20),
        ctx.get(Register::X21)
    );
    log::trace!(
        "  x22 {:<16x}  x23 {:<16x}  x24 {:<16x}",
        ctx.get(Register::X22),
        ctx.get(Register::X23),
        ctx.get(Register::X24)
    );
    log::trace!(
        "  x25 {:<16x}  x26 {:<16x}  x27 {:<16x}",
        ctx.get(Register::X25),
        ctx.get(Register::X26),
        ctx.get(Register::X27)
    );
    log::trace!(
        "  x28 {:<16x}  x29 {:<16x}  x30 {:<16x}",
        ctx.get(Register::X28),
        ctx.get(Register::X29),
        ctx.get(Register::X30)
    );
    log::trace!(
        "  x31 {:<16x}  mie {:<16x}  mip {:<16x}",
        ctx.get(Register::X31),
        ctx.get(Csr::Mie),
        ctx.get(Csr::Mip)
    );
}

// ————————————————————————————————— Tests —————————————————————————————————— //

/// We test some properties after handling a trap from firmware.
/// We simulate a trap by creating a dummy trap state for the context of the machine.
///
/// Mideleg must be 0: don't allow nested interrupts when running Miralis.
/// ctx.pc must be set to the handler start address.
/// Mie, vMie, vMideleg must not change.
/// vMepc and vMstatus.MIE must be set to corresponding values in ctx.trap_info.
/// vMip must be updated to the value of Mip.
/// In case of an interrupt, Mip must be cleared: avoid Miralis to trap again.
#[cfg(test)]
mod tests {

    use crate::arch::{mstatus, Arch, Architecture, Csr, MCause, Mode};
    use crate::handle_trap;
    use crate::host::MiralisContext;
    use crate::policy::{Policy, PolicyModule};
    use crate::virt::VirtContext;

    #[test]
    fn handle_trap_state() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw, 0x10000, 0x2000);
        let mut policy = Policy::init();
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        // Firmware is running
        ctx.mode = Mode::M;
        ctx.csr.mstatus = 0;
        ctx.csr.mie = 0b1;
        ctx.csr.mideleg = 0;
        ctx.csr.mtvec = 0x80200024; // Dummy mtvec

        // Simulating a trap
        ctx.trap_info.mepc = 0x80200042; // Dummy address
        ctx.trap_info.mstatus = 0b10000000;
        ctx.trap_info.mcause = MCause::Breakpoint as usize; // TODO : use a real int.
        ctx.trap_info.mip = 0b1;
        ctx.trap_info.mtval = 0;

        unsafe {
            Arch::write_csr(Csr::Mie, 0b1);
            Arch::write_csr(Csr::Mip, 0b1);
            Arch::write_csr(Csr::Mideleg, 0);
        };
        handle_trap(&mut ctx, &mut mctx, &mut policy);

        assert_eq!(Arch::read_csr(Csr::Mideleg), 0, "mideleg must be 0");
        assert_eq!(Arch::read_csr(Csr::Mie), 0b1, "mie must be 1");
        // assert_eq!(Arch::read_csr(Csr::Mip), 0, "mip must be 0"); // TODO : uncomment if using a real int.
        assert_eq!(ctx.pc, 0x80200024, "pc must be at handler start");
        assert_eq!(ctx.csr.mip, 0b1, "mip must to be updated");
        assert_eq!(ctx.csr.mie, 1, "mie must not change");
        assert_eq!(ctx.csr.mideleg, 0, "mideleg must not change");
        assert_eq!(ctx.csr.mepc, 0x80200042);
        assert_eq!(
            (ctx.csr.mstatus & mstatus::MPIE_FILTER) >> mstatus::MPIE_OFFSET,
            0b1,
            "mstatus.MPIE must be set to trap_info.mstatus.MPIE"
        );
    }
}
