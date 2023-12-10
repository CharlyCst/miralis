#![no_std]
#![no_main]

mod platform;

use core::panic::PanicInfo;
use core::arch::asm;

use platform::{debug_print, init};

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
    debug_print(core::format_args!("Hello, world!\n"));

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
