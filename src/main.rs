#![no_std]
#![no_main]

mod arch;
mod decoder;
mod logger;
mod platform;
mod trap;
mod virt;

use arch::{pmpcfg, Arch, Architecture};
use core::arch::asm;
use core::panic::PanicInfo;

use platform::{init, Plat, Platform};

use crate::arch::{Csr, Register};
use crate::decoder::{decode, Instr};
use crate::trap::MCause;
use crate::virt::VirtContext;

// Defined in the linker script
extern "C" {
    static _stack_start: usize;
    static _stack_end: usize;
}

#[no_mangle]
#[link_section = ".entry_point"]
pub unsafe extern "C" fn _start() -> ! {
    /// Address of the top of the stack (stack grow towerd lower addresses)
    static STACK: &'static usize = unsafe { &_stack_end };

    // Initialize stack pointer and jump into main
    // TODO: zero-out the BSS (QEMU might do it for us, but real hardware will not)
    asm!(
        "mv sp, {stack}",
        "j {main}",
        main = sym main,
        stack = in(reg) STACK,
        options(noreturn)
    );
}

extern "C" fn main() -> ! {
    init();
    log::info!("Hello, world!");
    log::info!("mstatus: 0x{:x}", Arch::read_mstatus());

    log::info!("Preparing jump into payload");
    let payload_addr = Plat::load_payload();
    unsafe {
        // Set return address, mode and PMP permissions
        Arch::write_mepc(payload_addr);
        Arch::set_mpp(arch::Mode::U);
        Arch::write_pmpcfg(0, pmpcfg::R | pmpcfg::W | pmpcfg::X | pmpcfg::TOR);
        Arch::write_pmpaddr(0, usize::MAX);
    }

    main_loop();
}

fn main_loop() -> ! {
    let mut ctx = VirtContext::default();
    ctx[Register::X2] = Plat::stack_address();

    loop {
        unsafe {
            Arch::enter_virt_firmware(&mut ctx);
            handle_trap(&mut ctx);
            log::info!("{:x?}", &ctx);
        }
    }
}

fn handle_trap(ctx: &mut VirtContext) {
    log::info!("Trapped!");
    log::info!("  mcause:  {:?}", Arch::read_mcause());
    log::info!("  mstatus: 0x{:x}", Arch::read_mstatus());
    log::info!("  mepc:    0x{:x}", Arch::read_mepc());
    log::info!("  mtval:   0x{:x}", Arch::read_mtval());

    // Temporary safeguard
    unsafe {
        static mut TRAP_COUNTER: usize = 0;

        if TRAP_COUNTER >= 10 {
            log::error!("Trap counter reached limit");
            Plat::exit_failure();
        }
        TRAP_COUNTER += 1;
    }

    match Arch::read_mcause() {
        MCause::EcallFromMMode | MCause::EcallFromUMode => {
            // For now we just exit successfuly
            log::info!("Success!");
            Plat::exit_success();
        }
        MCause::IllegalInstr => {
            let instr = unsafe { Arch::get_raw_faulting_instr() };
            let instr = decode(instr);
            log::info!("Faulting instruction: {:?}", instr);
            emulate_instr(ctx, &instr);
        }
        _ => (), // Continue
    }

    // Skip instruction and return
    unsafe {
        log::info!("Skipping trapping instruction");
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
        Instr::Csrrw(csr, reg) => {
            if csr.is_unknown() {
                todo!("Unknown CSR");
            }
            ctx[*csr] = ctx[*reg]
        }
        Instr::Csrrs(csr, reg) => match *csr {
            Csr::Mstatus => todo!("CSR not yet supported"),
            Csr::Mscratch => ctx[*reg] = ctx[*csr],
            Csr::Unknown => todo!("Unknown CSR"),
        },
        _ => (),
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    Plat::exit_failure();
}
