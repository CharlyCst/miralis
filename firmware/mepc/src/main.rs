#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{setup_firmware, success};

setup_firmware!(main);

fn main() -> ! {
    let secret: usize = 0x42;
    let res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mepc, {0}",
            "csrr {1}, mepc",
            in(reg) secret,
            out(reg) res,
        );
    }

    if res == secret {
        success();
    }

    panic!();
}
