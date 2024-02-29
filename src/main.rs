#![no_std]
#![no_main]

mod arch;
mod decoder;
mod logger;
mod platform;
mod virt;

use core::panic::PanicInfo;

use arch::{pmpcfg, Arch, Architecture};
use platform::{init, Plat, Platform};

use crate::arch::{Csr, MCause, Register};
use crate::decoder::{decode, Instr};
use crate::virt::{RegisterContext, VirtContext};

// Defined in the linker script
extern "C" {
    pub(crate) static _stack_start: usize;
    pub(crate) static _stack_end: usize;
}

pub(crate) extern "C" fn main(hart_id: usize, device_tree_blob_addr: usize) -> ! {
    init();
    log::info!("Hello, world!");
    log::info!("Hart ID: {}", hart_id);
    log::info!("mstatus: 0x{:x}", Arch::read_mstatus());
    log::info!("DTS address: 0x{:x}", device_tree_blob_addr);

    log::info!("Preparing jump into payload");
    let payload_addr = Plat::load_payload();
    let mut ctx = VirtContext::new(hart_id);
    unsafe {
        // Set return address, mode and PMP permissions
        Arch::write_mepc(payload_addr);
        Arch::set_mpp(arch::Mode::U);
        Arch::write_pmpcfg(0, pmpcfg::R | pmpcfg::W | pmpcfg::X | pmpcfg::TOR);
        Arch::write_pmpaddr(0, usize::MAX);

        // Configure the payload context
        ctx.set(Register::X2, Plat::payload_stack_address());
        ctx.set(Register::X10, hart_id);
        ctx.set(Register::X11, device_tree_blob_addr);
        ctx.set(Csr::Misa, Arch::read_misa());
    }

    main_loop(ctx);
}

fn main_loop(mut ctx: VirtContext) -> ! {
    loop {
        unsafe {
            Arch::enter_virt_firmware(&mut ctx);
            handle_trap(&mut ctx);
            log::trace!("{:x?}", &ctx);
        }
    }
}

fn handle_trap(ctx: &mut VirtContext) {
    log::trace!("Trapped!");
    log::trace!("  mcause:  {:?}", Arch::read_mcause());
    log::trace!("  mstatus: 0x{:x}", Arch::read_mstatus());
    log::trace!("  mepc:    0x{:x}", Arch::read_mepc());
    log::trace!("  mtval:   0x{:x}", Arch::read_mtval());

    // Keep track of the number of exit
    ctx.nb_exits += 1;
    log::trace!("  exits:   {}", ctx.nb_exits);
    if ctx.nb_exits >= 200 {
        log::error!("Trap counter reached limit: {}", ctx.nb_exits);
        Plat::exit_failure();
    }

    match Arch::read_mcause() {
        MCause::EcallFromMMode | MCause::EcallFromUMode => {
            // For now we just exit successfuly
            log::info!("Success!");
            log::info!("Number of payload exits: {}", ctx.nb_exits);
            Plat::exit_success();
        }
        MCause::IllegalInstr => {
            let instr = unsafe { Arch::get_raw_faulting_instr() };
            let instr = decode(instr);
            log::trace!("Faulting instruction: {:?}", instr);
            emulate_instr(ctx, &instr);
        }
        _ => (), // Continue
    }

    // Skip instruction and return
    unsafe {
        log::trace!("Skipping trapping instruction");
        Arch::write_mepc(Arch::read_mepc() + 4);
    }
}

fn emulate_instr(ctx: &mut VirtContext, instr: &Instr) {
    match instr {
        Instr::Wfi => {
            // For now payloads only call WFI when panicking
            log::error!("Payload panicked!");
            Plat::exit_failure();
        }
        Instr::Csrrw { csr, rd, rs1 } => {
            if csr.is_unknown() {
                todo!("Unknown CSR");
            }
            let tmp = ctx.get(csr);
            ctx.set(csr, ctx.get(rs1));
            ctx.set(rd, tmp);
        }
        Instr::Csrrs { csr, rd, rs1 } => {
            if csr.is_unknown() {
                todo!("Unknown CSR");
            }
            let tmp = ctx.get(csr);
            ctx.set(csr, tmp | ctx.get(rs1));
            ctx.set(rd, tmp);
        }
        Instr::Csrrwi { csr, rd, uimm } => {
            if csr.is_unknown() {
                todo!("Unknown CSR");
            }
            ctx.set(rd, ctx.get(csr));
            ctx.set(csr, *uimm);
        }
        Instr::Csrrsi { csr, rd, uimm } => {
            if csr.is_unknown() {
                todo!("Unknown CSR");
            }
            let tmp = ctx.get(csr);
            ctx.set(csr, tmp | uimm);
            ctx.set(rd, tmp);
        }
        Instr::Csrrc { csr, rd, rs1 } => {
            if csr.is_unknown() {
                todo!("Unknown CSR");
            }
            let tmp = ctx.get(csr);
            ctx.set(csr, tmp & !ctx.get(rs1));
            ctx.set(rd, tmp);
        }
        Instr::Csrrci { csr, rd, uimm } => {
            if csr.is_unknown() {
                todo!("Unknown CSR");
            }
            let tmp = ctx.get(csr);
            ctx.set(csr, tmp & !uimm);
            ctx.set(rd, tmp);
        }
        _ => todo!("Instruction not yet implemented: {:?}", instr),
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    Plat::exit_failure();
}
