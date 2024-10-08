#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{miralis_end_benchmark, setup_binary, BENCHMARK_NB_ITER};

setup_binary!(main);

fn main() -> ! {
    for _ in 0..BENCHMARK_NB_ITER {
        unsafe  {
            asm!(
                "csrw mscratch, 0x1", // Write ot mscratch to produce a trap to Miralis
            );
        }
    }

    miralis_end_benchmark()
}
