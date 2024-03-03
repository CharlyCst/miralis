#![no_std]

use core::arch::asm;
use core::hint;

/// Ask Mirage to exit with a success error code.
pub fn success() -> ! {
    unsafe {
        asm!("ecall");
    }

    // Loop forever, this should never happen as Mirage will terminate the execution before.
    loop {
        hint::spin_loop();
    }
}

/// Ask Mirage to exit with a failure error code.
pub fn failure() -> ! {
    unsafe {
        asm!("wfi");
    }

    // Loop forever, this should never happen as Mirage will terminate the execution before.
    loop {
        hint::spin_loop();
    }
}
