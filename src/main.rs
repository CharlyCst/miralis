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

use crate::arch::pmp_lib::write_pmp_cfg_and_addr;
use crate::arch::{misa, Csr, Register};
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

    log::info!("Preparing jump into payload");
    let payload_addr = Plat::load_payload();
    let mut ctx = VirtContext::new(hart_id); // Virtual context used to copy from and to hardware

    unsafe {
        // Set return address, mode and PMP permissions
        Arch::set_mpp(arch::Mode::U);
        if Plat::get_nb_pmp() > 0 {
            let mirage_pmps = setup_mirage_pmp();
            // Give all the remaining PMPs to the firmware
            // Must keep the last PMP for mirage: to control global permissions
            ctx.set_pmp_values(Plat::get_nb_pmp() - mirage_pmps - 1, mirage_pmps);
        }

        // Configure the payload context
        ctx.set(Register::X10, hart_id);
        ctx.set(Register::X11, device_tree_blob_addr);
        ctx.set(Csr::Misa, Arch::read_misa() & !misa::DISABLED);
        ctx.pc = payload_addr;
    }

    main_loop(ctx);
}

fn main_loop(mut ctx: VirtContext) -> ! {
    let max_exit = debug::get_max_payload_exits();

    log::debug!("starting loop");
    loop {
        unsafe {
            Arch::run_vcpu(&mut ctx);
            handle_trap(&mut ctx, max_exit);
            log::trace!("{:x?}", &ctx);
        }
    }
}

fn handle_trap(ctx: &mut VirtContext, max_exit: Option<usize>) {
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

    if ctx.trap_info.from_mmode() {
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
            unsafe { Arch::switch_from_firmware_to_payload(ctx) };
        }
        (ExecutionMode::Payload, ExecutionMode::Firmware) => {
            log::debug!("Execution mode: Payload -> Firmware");
            unsafe { Arch::switch_from_payload_to_firmware(ctx) };
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

fn setup_mirage_pmp() -> usize {
    // These can be used to compute the values to put into the PMPs
    /*
    // 0 -> 1
    let cfg0 = pmp_write_compute(0, 0, 0x80000000, pmpcfg::R | pmpcfg::W | pmpcfg::X);
    // 1 -> 2
    let cfg1 = pmp_write_compute(
        1,
        0x80000000,
        0x80100000 - 0x80000000,
        pmpcfg::R | pmpcfg::W | pmpcfg::X,
    );
    // 3 -> 15
    let cfg2 = pmp_write_compute(2, 0, usize::MAX, pmpcfg::R | pmpcfg::W | pmpcfg::X);

    log::debug!("(0) : 1st cfg 0x{:x}", cfg0.cfg1);
    log::debug!("(0) : 2nd cfg 0x{:x}", cfg0.cfg2);
    log::debug!("(0) : 1st addr 0x{:x}", cfg0.addr1);
    log::debug!("(0) : 2nd addr 0x{:x}", cfg0.addr2);
    log::debug!("(0) : addr mode 0x{:?}", cfg0.addressing_mode);

    log::debug!("(1) : 1st cfg 0x{:x}", cfg1.cfg1);
    log::debug!("(1) : 2nd cfg 0x{:x}", cfg1.cfg2);
    log::debug!("(1) : 1st addr 0x{:x}", cfg1.addr1);
    log::debug!("(1) : 2nd addr 0x{:x}", cfg1.addr2);
    log::debug!("(1) : addr mode 0x{:?}", cfg1.addressing_mode);

    log::debug!("(2) : 1st cfg 0x{:x}", cfg2.cfg1);
    log::debug!("(2) : 2nd cfg 0x{:x}", cfg2.cfg2);
    log::debug!("(2) : 1st addr 0x{:x}", cfg2.addr1);
    log::debug!("(2) : 2nd addr 0x{:x}", cfg2.addr2);
    log::debug!("(2) : addr mode 0x{:?}", cfg2.addressing_mode);
    */

    // This can be used to figure out the granularity of the PMPs of the hardware
    /*
    pmpcfg_csr_write(0, 0);
    pmpaddr_csr_write(0, usize::MAX);

    log::debug!("pmpcfg0 is 0x{:x}", pmpcfg_csr_read(0));
    log::debug!("pmpaddr0 is 0x{:x}", pmpaddr_csr_read(0));
     */

    // Setup 4 PMPs for mirage
    write_pmp_cfg_and_addr(
        0,
        pmpcfg::R | pmpcfg::W | pmpcfg::X | pmpcfg::TOR,
        0x20000000,
    );
    write_pmp_cfg_and_addr(1, pmpcfg::TOR, 0x20040000);
    write_pmp_cfg_and_addr(2, 0, 0);
    write_pmp_cfg_and_addr(3, 0, 0);
    write_pmp_cfg_and_addr(
        Plat::get_nb_pmp() - 1,
        pmpcfg::R | pmpcfg::W | pmpcfg::X | pmpcfg::TOR,
        0x3fffffffffffffff,
    );
    /*
    log::debug!("pmpcfg0 is 0x{:x}", pmpcfg_csr_read(0));
    log::debug!("pmpcfg2 is 0x{:x}", pmpcfg_csr_read(Plat::get_nb_pmp() - 1));

    log::debug!("pmpaddr0 is 0x{:x}", pmpaddr_csr_read(0));
    log::debug!("pmpaddr1 is 0x{:x}", pmpaddr_csr_read(1));
    log::debug!("pmpaddr2 is 0x{:x}", pmpaddr_csr_read(2));
    log::debug!("pmpaddr3 is 0x{:x}", pmpaddr_csr_read(3));

    log::debug!("pmpaddr4 is 0x{:x}", pmpaddr_csr_read(4));
    log::debug!("pmpaddr5 is 0x{:x}", pmpaddr_csr_read(5));
    log::debug!("pmpaddr6 is 0x{:x}", pmpaddr_csr_read(6));
    log::debug!("pmpaddr7 is 0x{:x}", pmpaddr_csr_read(7));
    log::debug!("pmpaddr8 is 0x{:x}", pmpaddr_csr_read(8));
    log::debug!("pmpaddr9 is 0x{:x}", pmpaddr_csr_read(9));
    log::debug!("pmpaddr10 is 0x{:x}", pmpaddr_csr_read(10));
    log::debug!("pmpaddr11 is 0x{:x}", pmpaddr_csr_read(11));

    log::debug!(
        "pmpaddr15 is 0x{:x}",
        pmpaddr_csr_read(Plat::get_nb_pmp() - 1)
    );
    */
    unsafe { Arch::flush_with_sfence() };

    let nb_mirage_pmps = 4;
    return nb_mirage_pmps;
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    unsafe { debug::log_stack_usage() };
    Plat::exit_failure();
}
