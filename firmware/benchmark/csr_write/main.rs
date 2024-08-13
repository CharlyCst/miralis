#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{setup_firmware, success, BENCHMARK_NB_ITER};

setup_firmware!(main);

fn main() -> ! {
    for _ in 0..BENCHMARK_NB_ITER {
        unsafe  {
            asm!(
                "csrw mscratch, 0x1", // Write ot mscratch to produce a trap to Miralis
            );
        }
    }
    success();
}
