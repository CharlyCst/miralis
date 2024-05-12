#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    log::info!("Hello from default payload!");

    let secret: usize = 0x42;
    let res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
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
