//! Miralis entry point
//!
//! The main function is called directly after platform specific minimal setup (such as
//! configuration of the stack).

// Mark the crate as no_std and no_main, but only when not running tests.
// We need both std and main to be able to run tests in user-space on the host architecture.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

mod arch;
mod config;
mod debug;
mod decoder;
mod device;
mod driver;
mod host;
mod logger;
mod platform;
mod utils;
mod virt;

use arch::pmp::pmpcfg;
use arch::{pmp, Arch, Architecture};
use platform::{init, Plat, Platform};

use crate::arch::{misa, Csr, Register};
use crate::host::MiralisContext;
use crate::virt::traits::*;
use crate::virt::{ExecutionMode, VirtContext};

// Defined in the linker script
extern "C" {
    pub(crate) static _stack_start: u8;
}

pub(crate) extern "C" fn main(hart_id: usize, device_tree_blob_addr: usize) -> ! {
    // For now we simply park all the harts other than the boot one
    if hart_id != 0 {
        loop {
            Arch::wfi();
            core::hint::spin_loop();
        }
    }

    init();
    log::info!("Hello, world!");
    log::info!("Platform name: {}", Plat::name());
    log::info!("Hart ID: {}", hart_id);
    log::info!("misa:    0x{:x}", Arch::read_csr(Csr::Misa));
    log::info!(
        "vmisa:   0x{:x}",
        Arch::read_csr(Csr::Misa) & !misa::DISABLED
    );
    log::info!("mstatus: 0x{:x}", Arch::read_csr(Csr::Mstatus));
    log::info!("DTS address: 0x{:x}", device_tree_blob_addr);

    log::info!("Preparing jump into firmware");
    let firmware_addr = Plat::load_firmware();
    let nb_pmp = Plat::get_nb_pmp();
    let nb_virt_pmp;
    let clint = Plat::create_clint_device();

    // Detect hardware capabilities
    // SAFETY: this lust happen before hardware initialization
    let hw = unsafe { Arch::detect_hardware() };

    // Initialize Miralis's own context
    let mut mctx = MiralisContext::new(nb_pmp, hw);

    // Configure PMP registers, if available
    if nb_pmp >= 16 {
        // Protect Miralis with the first pmp
        let (start, size) = Plat::get_miralis_memory_start_and_size();
        mctx.pmp
            .set(0, pmp::build_napot(start, size).unwrap(), pmpcfg::NAPOT);
        // Protect CLINT memory to trap firmware read/writes there
        mctx.pmp.set(
            1,
            pmp::build_napot(clint.start_addr, clint.size).unwrap(),
            pmpcfg::NAPOT,
        );
        // Add an inactive 0 entry so that the next PMP sees 0 with TOR configuration
        mctx.pmp.set(2, 0, pmpcfg::INACTIVE);
        // Finally, set the last PMP to grant access to the whole memory
        mctx.pmp
            .set(nb_pmp - 1, usize::MAX, pmpcfg::RWX | pmpcfg::NAPOT);
        // Give 8 PMPs to the firmware
        mctx.virt_pmp_offset = 2;
        if let Some(max_virt_pmp) = config::VCPU_MAX_PMP {
            nb_virt_pmp = core::cmp::min(8, max_virt_pmp);
        } else {
            nb_virt_pmp = 8;
        }
    } else {
        nb_virt_pmp = 0;
    }

    // Initialize the virtual context and configure architecture
    let mut ctx = VirtContext::new(hart_id, nb_virt_pmp);
    unsafe {
        // Set return address, mode and PMP permissions
        Arch::set_mpp(arch::Mode::U);
        // Update the PMPs prior to first entry
        Arch::write_pmp(&mctx.pmp);
        Arch::sfence_vma();

        // Configure the firmware context
        ctx.set(Register::X10, hart_id);
        ctx.set(Register::X11, device_tree_blob_addr);
        ctx.set_csr(
            Csr::Misa,
            Arch::read_csr(Csr::Misa) & !misa::DISABLED,
            &mctx.hw,
        );
        ctx.pc = firmware_addr;
    }

    main_loop(ctx, mctx);
}

fn main_loop(mut ctx: VirtContext, mut mctx: MiralisContext) -> ! {
    loop {
        unsafe {
            Arch::run_vcpu(&mut ctx);
            handle_trap(&mut ctx, &mut mctx);
        }
    }
}

fn handle_trap(ctx: &mut VirtContext, mctx: &mut MiralisContext) {
    if log::log_enabled!(log::Level::Trace) {
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
        return;
    }

    // Perform emulation
    let exec_mode = ctx.mode.to_exec_mode();

    // Keep track of the number of exit
    ctx.nb_exits += 1;
    match exec_mode {
        ExecutionMode::Firmware => ctx.handle_firmware_trap(&mctx),
        ExecutionMode::Payload => ctx.emulate_jump_trap_handler(),
    }

    // Check for execution mode change
    match (exec_mode, ctx.mode.to_exec_mode()) {
        (ExecutionMode::Firmware, ExecutionMode::Payload) => {
            log::debug!("Execution mode: Firmware -> Payload");
            unsafe { ctx.switch_from_firmware_to_payload(mctx) };
        }
        (ExecutionMode::Payload, ExecutionMode::Firmware) => {
            log::debug!("Execution mode: Payload -> Firmware");
            unsafe { ctx.switch_from_payload_to_firmware(mctx) };
        }
        _ => {} // No execution mode transition
    }
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

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    unsafe { debug::log_stack_usage() };
    Plat::exit_failure();
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
        "  x30 {:<16x}  mie {:<16x}  mip {:<16x}",
        ctx.get(Register::X30),
        ctx.get(Csr::Mie),
        ctx.get(Csr::Mip)
    );
}
