#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{setup_firmware, success};

setup_firmware!(main);

fn main() -> ! {
    for _ in 0..10_000 {
        unsafe  {
            asm!(
                "csrw mscratch, 0x1",       // Write ot mscratch to produce a trap to Miralis
            );
        }
    }
    success();
}
