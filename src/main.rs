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
mod logger;
mod platform;
mod virt;

use arch::{pmpcfg, Arch, Architecture};
use platform::{init, Plat, Platform};

use crate::arch::{misa, Csr, Register};
use crate::virt::{RegisterContext, Runner, VirtContext};

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

    log::info!("Preparing jump into payload");
    let payload_addr = Plat::load_payload();
    let mut ctx = VirtContext::new(hart_id); // Virtual context used to copy from and to hardware
    let mut runner = Runner::Firmware; // The runner indicates who is running on the hardware

    unsafe {
        // Set return address, mode and PMP permissions
        Arch::set_mpp(arch::Mode::U);
        Arch::write_pmpcfg(0, pmpcfg::R | pmpcfg::W | pmpcfg::X | pmpcfg::TOR); // TODO: I do not think these writes are working
        Arch::write_pmpaddr(0, usize::MAX); // TODO: I do not think these writes are working

        // Configure misa to execute with expected features
        Arch::write_misa(Arch::read_misa() & !misa::DISABLED); // TODO: I do not think these writes are working

        // Configure the payload context
        ctx.set(Register::X10, hart_id);
        ctx.set(Register::X11, device_tree_blob_addr);
        ctx.set(Csr::Misa, Arch::read_misa() & !misa::DISABLED);
        ctx.pc = payload_addr;
    }

    main_loop(ctx, &mut runner);
}

fn main_loop(mut ctx: VirtContext, runner: &mut Runner) -> ! {
    let max_exit = debug::get_max_payload_exits();

    loop {
        unsafe {
            match *runner {
                Runner::Firmware => Arch::enter_virt_firmware(&mut ctx),
                Runner::OS => Arch::enter_virt_os(&mut ctx),
            }
            handle_trap(&mut ctx, max_exit, runner);
        }
    }
}

fn handle_trap(ctx: &mut VirtContext, max_exit: Option<usize>, runner: &mut Runner) {
    log::trace!("Trapped!");
    log::trace!("  mcause:  {:?}", ctx.trap_info.mcause);
    log::trace!("  mstatus: 0x{:x}", ctx.trap_info.mstatus);
    log::trace!("  mepc:    0x{:x}", ctx.trap_info.mepc);
    log::trace!("  mtval:   0x{:x}", ctx.trap_info.mtval);
    log::trace!("  exits:   {}", ctx.nb_exits + 1);
    log::trace!("  runner:  {:?}", *runner);

    if let Some(max_exit) = max_exit {
        if ctx.nb_exits + 1 >= max_exit {
            log::error!("Reached maximum number of exits: {}", ctx.nb_exits);
            Plat::exit_failure();
        }
    }

    match *runner {
        Runner::Firmware => {
            // Firmware trap : can come from emulated firmware or mirage (emulation code could trap)
            if ctx.trap_info.from_mmode() {
                //Trap comes from M mode: mirage
                handle_mirage_trap(ctx, runner);
            } else {
                handle_firmware_trap(ctx, runner);
            }
        }
        Runner::OS => {
            handle_os_trap(ctx, runner);
        }
    }
}

fn handle_firmware_trap(ctx: &mut VirtContext, runner: &mut Runner) {
    ctx.handle_payload_trap(runner);
}

fn handle_os_trap(ctx: &mut VirtContext, runner: &mut Runner) {
    ctx.nb_exits += 1;
    ctx.emulate_jump_trap_handler(runner);
    *runner = Runner::Firmware;
}

/// Handle the trap coming from mirage
fn handle_mirage_trap(_ctx: &mut VirtContext, runner: &mut Runner) {
    todo!("Mirage trap handler entered");
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    unsafe { debug::log_stack_usage() };
    Plat::exit_failure();
}
