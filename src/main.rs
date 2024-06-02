//! Mirage entry point
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
mod host;
mod logger;
mod platform;
mod virt;

use arch::pmp::pmpcfg;
use arch::{pmp, Arch, Architecture};
use platform::{init, Plat, Platform};

use crate::arch::{misa, Csr, Register};
use crate::host::MirageContext;
use crate::virt::{ExecutionMode, RegisterContext, VirtContext};

// Defined in the linker script
extern "C" {
    pub(crate) static _stack_bottom: u8;
    pub(crate) static _stack_top: u8;
}

pub(crate) extern "C" fn main(hart_id: usize, device_tree_blob_addr: usize) -> ! {
    init();
    log::info!("Hello, world!");
    log::info!("Hart ID: {}", hart_id);
    log::info!("misa:    0x{:x}", Arch::read_misa());
    log::info!("vmisa:   0x{:x}", Arch::read_misa() & !misa::DISABLED);
    log::info!("mstatus: 0x{:x}", Arch::read_mstatus());
    log::info!("DTS address: 0x{:x}", device_tree_blob_addr);

    log::info!("Preparing jump into firmware");
    let firmware_addr = Plat::load_firmware();
    let nb_pmp = Plat::get_nb_pmp();
    let nb_virt_pmp;

    // Initialize Mirage's own context
    let mut mctx = MirageContext::new(nb_pmp);

    // Configure PMP registers, if available
    if nb_pmp >= 16 {
        // Protect Mirage with the first pmp
        let (start, size) = Plat::get_mirage_memory_start_and_size();
        mctx.pmp
            .set(0, pmp::build_napot(start, size).unwrap(), pmpcfg::NAPOT);
        // Add an inactive 0 entry so that the next PMP sees 0 with TOR configuration
        mctx.pmp.set(1, 0, pmpcfg::INACTIVE);
        // Finally, set the last PMP to grant access to the whole memory
        mctx.pmp
            .set(nb_pmp - 1, usize::MAX, pmpcfg::RWX | pmpcfg::NAPOT);
        // Give 8 PMPs to the firmware
        mctx.virt_pmp_offset = 2;
        nb_virt_pmp = 8;
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
        ctx.set(Csr::Misa, Arch::read_misa() & !misa::DISABLED);
        ctx.pc = firmware_addr;
    }

    main_loop(ctx, mctx);
}

fn main_loop(mut ctx: VirtContext, mut mctx: MirageContext) -> ! {
    let max_exit = debug::get_max_firmware_exits();

    loop {
        unsafe {
            Arch::run_vcpu(&mut ctx);
            handle_trap(&mut ctx, &mut mctx, max_exit);
            log::trace!("{:x?}", &ctx);
        }
    }
}

fn handle_trap(ctx: &mut VirtContext, mctx: &mut MirageContext, max_exit: Option<usize>) {
    log::trace!("Trapped!");
    log::trace!("  mcause:  {:?}", ctx.trap_info.mcause);
    log::trace!("  mstatus: 0x{:x}", ctx.trap_info.mstatus);
    log::trace!("  mepc:    0x{:x}", ctx.trap_info.mepc);
    log::trace!("  mtval:   0x{:x}", ctx.trap_info.mtval);
    log::trace!("  exits:   {}", ctx.nb_exits + 1);
    log::trace!("  mode:    {:?}", ctx.mode);

    if let Some(max_exit) = max_exit {
        if ctx.nb_exits + 1 >= max_exit {
            log::error!("Reached maximum number of exits: {}", ctx.nb_exits);
            Plat::exit_failure();
        }
    }

    if ctx.trap_info.is_from_mmode() {
        // Trap comes from M mode: Mirage
        handle_mirage_trap(ctx);
        return;
    }

    // Perform emulation
    let exec_mode = ctx.mode.to_exec_mode();
    match exec_mode {
        ExecutionMode::Firmware => handle_firmware_trap(ctx),
        ExecutionMode::Payload => handle_os_trap(ctx),
    }

    // Check for execution mode change
    match (exec_mode, ctx.mode.to_exec_mode()) {
        (ExecutionMode::Firmware, ExecutionMode::Payload) => {
            log::debug!("Execution mode: Firmware -> Payload");
            unsafe { Arch::switch_from_firmware_to_payload(ctx, mctx) };
        }
        (ExecutionMode::Payload, ExecutionMode::Firmware) => {
            log::debug!("Execution mode: Payload -> Firmware");
            unsafe { Arch::switch_from_payload_to_firmware(ctx, mctx) };
        }
        _ => {} // No execution mode transition
    }
}

fn handle_firmware_trap(ctx: &mut VirtContext) {
    ctx.handle_payload_trap();
}

fn handle_os_trap(ctx: &mut VirtContext) {
    ctx.nb_exits += 1;
    ctx.emulate_jump_trap_handler();
}

/// Handle the trap coming from mirage
fn handle_mirage_trap(ctx: &mut VirtContext) {
    let trap = &ctx.trap_info;
    log::error!("Unexpected trap while executing Mirage");
    log::error!("  cause:   {} ({:?})", trap.mcause, trap.get_cause());
    log::error!("  mepc:    0x{:x}", trap.mepc);
    log::error!("  mtval:   0x{:x}", trap.mtval);
    log::error!("  mstatus: 0x{:x}", trap.mstatus);
    log::error!("  mip:     0x{:x}", trap.mip);

    todo!("Mirage trap handler entered");
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    unsafe { debug::log_stack_usage() };
    Plat::exit_failure();
}
