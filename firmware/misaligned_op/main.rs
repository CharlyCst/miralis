#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::info!("Hello from Misaligned operations firmware");

    // Error code 4 - LoadAddrMisaligned
    unsafe {
        asm!("li t1, 0x80400001", "ld t2, 0(t1)", out("t1") _, out("t2") _);
    }

    // Error code 6 - StoreAddrMisaligned
    unsafe {
        asm!("li t1, 0x80400001", "sd t2,0(t1)", out("t1") _, out("t2") _);
    }

    // Ensure emulation works correctly
    let mut var: usize = 0xdeadbeef;
    let var2: usize;

    unsafe {
        asm!("li t1, 0x80400001", "sd t2,0(t1)", "ld t3, 0(t1)", out("t1") _, inout("t2") var, out("t3") var2);
    }

    assert_eq!(var, var2, "Emulation doesn't work properly");

    // Correctness tests
    success()
}
