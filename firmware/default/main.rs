#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::info!("Hello from default firmware!");

    let secret: usize = 0x42;
    let res: usize;
    unsafe {
        asm!(
            "csrw mscratch, {0}",
            "csrr {1}, mscratch",
            in(reg) secret,
            out(reg) res,
        );
    }

    if res == secret {
        success();
    }

    panic!();
}
