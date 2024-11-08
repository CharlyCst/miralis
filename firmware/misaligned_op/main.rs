//! This firmware acts as a test for misaligned stores and loads. It is only useful in the presence of a platform that doesn't emulate misaligned loads and stores
#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::info!("Hello from Misaligned operations firmware");
    let misaligned_address: usize = 0x80400001;
    let value: usize = 0xdeaddead;

    // Error code 4 - LoadAddrMisaligned
    unsafe {
        asm!(
        "ld {r}, 0({addr})",
        addr = in(reg) misaligned_address,
        r = out(reg) _,
        )
    }

    // Error code 6 - StoreAddrMisaligned
    unsafe {
        asm!(
        "sd {r}, 0({addr})",
        addr = in(reg) misaligned_address,
        r = in(reg) value,
        )
    }

    // Correctness test
    let mut read_value: usize;
    unsafe {
        asm!(
        "ld {r}, 0({addr})",
        addr = in(reg) misaligned_address,
        r = out(reg) read_value,
        )
    }

    assert_eq!(
        read_value, value,
        "Misaligned loads and stores emulation doesn't work properly"
    );

    success()
}
