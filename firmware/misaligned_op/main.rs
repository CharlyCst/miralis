//! This firmware acts as a test for misaligned stores and loads. It is only useful in the presence of a platform that doesn't emulate misaligned loads and stores
#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::info!("Hello from Misaligned operations firmware");

    // Error code 4 - LoadAddrMisaligned
    unsafe {
        asm!(
        "ld {r}, 0({addr})",
        addr = in(reg) 0x80400001,
        r = out(reg) _,
        )
    }

    // Error code 6 - StoreAddrMisaligned
    unsafe {
        asm!(
        "ld {r}, 0({addr})",
        addr = in(reg) 0x80400001,
        r = out(reg) 0xdeaddead,
        )
    }

    // Correctness test
    let misaligned_address: usize = 0x80400001;
    let mut read_value: usize;

    unsafe {
        asm!("li t2, 0xdeadbeef", "sd t2,0(t1)","ld t3, 0(t1)", in("t1") misaligned_address, out("t2") _, out("t3") read_value);
    }

    assert_eq!(
        read_value, 0xdeadbeef,
        "Misaligned loads and stores emulation doesn't work properly"
    );

    success()
}
