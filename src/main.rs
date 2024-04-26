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
    let mut temp_ctx = VirtContext::new(hart_id);

    let mut guest_ctx = VirtContext::new(hart_id);
    let mut mirage_ctx = VirtContext::new(hart_id);
    let mut runner = Runner::Firmware;

    unsafe {
        // Set return address, mode and PMP permissions
        Arch::set_mpp(arch::Mode::U);
        Arch::write_pmpcfg(0, pmpcfg::R | pmpcfg::W | pmpcfg::X | pmpcfg::TOR);
        Arch::write_pmpaddr(0, usize::MAX);

        // Configure misa to execute with expected features
        Arch::write_misa(Arch::read_misa() & !misa::DISABLED);

        // TODO : I do not think these writes are working

        //Set the mirage context to the correct configuration
        mirage_ctx.set(Csr::Mstatus, Arch::read_mstatus());
        //mirage_ctx.set(Csr::Pmpcfg(0), Arch::read_pmpcfg(0));
        mirage_ctx.csr.pmp_cfg[0] = pmpcfg::R | pmpcfg::W | pmpcfg::X | pmpcfg::TOR;
        //mirage_ctx.set(Csr::Pmpaddr(0), Arch::read_pmpaddr(0));
        mirage_ctx.csr.pmp_addr[0] = usize::MAX;

        mirage_ctx.set(Csr::Misa, Arch::read_misa());
        mirage_ctx.set(Csr::Mtvec, Arch::read_mtvec());

        // Configure the payload context
        guest_ctx.set(Register::X10, hart_id);
        guest_ctx.set(Register::X11, device_tree_blob_addr);
        guest_ctx.set(Csr::Misa, Arch::read_misa() & !misa::DISABLED);
        guest_ctx.pc = payload_addr;
    }

    log::trace!("  guest: {:x?}", guest_ctx);
    log::trace!("  mirage: {:x?}", mirage_ctx);
    log::trace!("  temp: {:x?}", temp_ctx);

    VirtContext::copy_csr_regs(&mut temp_ctx, &mut mirage_ctx);
    VirtContext::copy_simple_regs(&mut temp_ctx, &mut guest_ctx);

    log::trace!("  guest: {:x?}", guest_ctx);
    log::trace!("  mirage: {:x?}", mirage_ctx);
    log::trace!("  temp: {:x?}", temp_ctx);

    main_loop(temp_ctx, guest_ctx, mirage_ctx, &mut runner);
}

fn main_loop(
    mut temp_ctx: VirtContext,
    mut guest_ctx: VirtContext,
    mut mirage_ctx: VirtContext,
    runner: &mut Runner,
) -> ! {
    let max_exit = debug::get_max_payload_exits();

    loop {
        unsafe {
            log::trace!("ENTERING FIRMWARE");
            Arch::enter_virt_firmware(&mut temp_ctx);
            handle_trap(
                &mut temp_ctx,
                &mut guest_ctx,
                &mut mirage_ctx,
                max_exit,
                runner,
            );
            log::trace!("temp post trap : {:x?}", &temp_ctx);
        }
    }
}

fn handle_trap(
    temp_ctx: &mut VirtContext,
    guest_ctx: &mut VirtContext,
    mirage_ctx: &mut VirtContext,
    max_exit: Option<usize>,
    runner: &mut Runner,
) {
    log::trace!("Trapped!");
    log::trace!("  mcause:  {:?}", temp_ctx.trap_info.mcause);
    log::trace!("  mstatus: 0x{:x}", temp_ctx.trap_info.mstatus);
    log::trace!("  mepc:    0x{:x}", temp_ctx.trap_info.mepc);
    log::trace!("  mtval:   0x{:x}", temp_ctx.trap_info.mtval);
    log::trace!("  exits:   {}", temp_ctx.nb_exits + 1);
    log::trace!("  runner:  {:?}", *runner );

    log::trace!("  temp: {:x?}", temp_ctx);

    if let Some(max_exit) = max_exit {
        if temp_ctx.nb_exits + 1 >= max_exit {
            log::error!("Reached maximum number of exits: {}", temp_ctx.nb_exits);
            Plat::exit_failure();
        }
    }

    match *runner {
        Runner::Firmware => {
            if temp_ctx.trap_info.from_mmode() {
                //Trap comes from M mode : mirage
                handle_mirage_trap(temp_ctx);
            } else {
                VirtContext::copy_simple_regs(guest_ctx, temp_ctx);

                emulate_and_setup_trap_return(runner, temp_ctx, mirage_ctx, guest_ctx);
            }
        }
        Runner::OS => {
            log::debug!("trap is from OS");
            // Trap comes from the guest OS : need to context switch and jump into the trap handler of the guest firmware
            VirtContext::complete_copy(guest_ctx, temp_ctx);
            VirtContext::copy_csr_regs(temp_ctx, mirage_ctx);

            log::trace!("  guest: {:x?}", guest_ctx);

            emulate_and_setup_trap_return(runner, temp_ctx, mirage_ctx, guest_ctx);
        }
    }
}

fn emulate_and_setup_trap_return(
    runner: &mut Runner,
    temp_ctx: &mut VirtContext,
    mirage_ctx: &mut VirtContext,
    guest_ctx: &mut VirtContext,
) {
    log::trace!("Function entered");

    guest_ctx.trap_info = temp_ctx.trap_info;
    log::trace!("  guest: {:x?}", guest_ctx);
    match *runner {
        Runner::Firmware => guest_ctx.handle_payload_trap(runner),
        Runner::OS => {
            log::debug!("Jump into firmware trap_handler");
            guest_ctx.emulate_jump_trap_handler(runner);
            guest_ctx.nb_exits += 1;
        }
    }

    temp_ctx.nb_exits = guest_ctx.nb_exits;

    log::trace!("trap handled");

    match *runner {
        Runner::Firmware => {
            log::debug!("post trap : firmware");
            VirtContext::copy_simple_regs(temp_ctx, guest_ctx);
        }
        Runner::OS => {
            log::debug!("post trap : OS");
            VirtContext::copy_csr_regs(mirage_ctx, temp_ctx);
            VirtContext::complete_copy(temp_ctx, guest_ctx);

            log::trace!("  guest: {:x?}", guest_ctx);
            log::trace!("  mirage: {:x?}", mirage_ctx);
            log::trace!("  temp: {:x?}", temp_ctx);

           // todo!("jump to S-mode fails");
        }
    }

    log::trace!("guest post trap handle: {:x?}", guest_ctx);
    log::debug!("mstatus post copy : 0x{:x}", temp_ctx.csr.mstatus);

    log::trace!("function finished");
} 

/// Handle the trap coming from mirage
fn handle_mirage_trap(_ctx: &mut VirtContext) {
    log::trace!("Mirage trap handler entered");
    todo!();
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    unsafe { debug::log_stack_usage() };
    Plat::exit_failure();
}
