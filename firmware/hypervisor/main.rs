#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::info!("Hello from hypervisor firmware!");

    let misa: u64;
    unsafe {
        // Read the misa CSR into the variable misa
        asm!("csrr {}, misa", out(reg) misa);
    }

    if (misa & (1 << 7)) == 0 {
        log::info!("H extension is not available skipping the hypervisor payload");
        success();
    }

    unsafe {
        asm!(
            // Read the Hypervisor Status Register (hstatus)
            "csrr t0, hstatus",        // Store hstatus in t0
            "csrw hstatus, t0",        // Write t0 back into hstatus (just for demonstration)

            // Read Hypervisor Exception Delegation Register (hedeleg)
            "csrr t1, hedeleg",        // Store hedeleg in t1
            "csrw hedeleg, t1",        // Write t1 back into hedeleg

            // Read Hypervisor Interrupt Delegation Register (hideleg)
            "csrr t2, hideleg",        // Store hideleg in t2
            "csrw hideleg, t2",        // Write t2 back into hideleg

            // Example of hypervisor trap handling (setting the virtual supervisor address)
            "csrr t3, htval",          // Read htval (Hypervisor Trap Value Register)
            "csrw htval, t3",          // Write back to htval

            // Hypervisor virtual interrupt enable
            "csrr t4, hvip",           // Read Hypervisor Virtual Interrupt Pending Register
            "csrw hvip, t4",           // Write back to hvip

            // Insert fences for hypervisor virtual memory synchronization
            // HFENCE.VVMA vaddr, asid
            "hfence.vvma zero, zero",    // Synchronize virtual memory for all guest virtual machines

            // HFENCE.GVMA gaddr, gpid
            "hfence.gvma zero, zero",    // Synchronize virtual memory for the hypervisor

            out("t0") _, out("t1") _, out("t2") _, out("t3") _, out("t4") _,
        );
    }

    success();
}
