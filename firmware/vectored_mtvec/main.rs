#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::hint::spin_loop;

use miralis_abi::{failure, setup_binary, success};
use test_helpers::clint;

setup_binary!(main);

fn main() -> ! {
    // Set mtvec to vectored mode at base _raw_interrupt_trap_handler
    // Then produce an interrupt (MTI) to fall inside the vector
    // The redirection should end in success_trap_handler
    unsafe {
        let _raw_interrupt_trap_handler = _raw_interrupt_trap_handler as usize | 0b1;

        asm!(
            "csrw mtvec, {handler}",       // Setup trap handler
            "csrw mideleg, 0",
            "csrs mstatus, {mstatus_mie}", // Enable interrupts (MIE)
            "csrs mie, {mtie}",            // Enable machine timer interrupt (MTIE)
            handler = in(reg) _raw_interrupt_trap_handler,
            mstatus_mie = in(reg) 0x8,
            mtie = in(reg) 0x80,
        );
    }

    // Setup a timer deadline in the past to trap directly
    clint::set_mtimecmp_deadline(0, 0);

    for _ in 0..10_000 {
        spin_loop()
    }

    // The trap handler should exit, if we reach that point the handler did not do its job
    log::error!("Firmware didn't trapped!");
    failure()
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

/// This function should be called from the raw trap handler
extern "C" fn success_trap_handler() {
    success();
}

/// This function should be called from the raw trap handler
extern "C" fn failure_trap_handler() {
    let mcause: usize;
    unsafe {
        asm!(
            "csrr {mcause}, mcause",
            mcause = out(reg) mcause,
        )
    };
    log::info!("mcause: ox{:x}", mcause);
    failure();
}

// Define your vector table
global_asm!(
    r#"
.text
.align 4
.global _raw_interrupt_trap_handler
_raw_interrupt_trap_handler:
    j {failure_trap_handler} // 0: User Software Interrupt
    j {failure_trap_handler} // 1: Supervisor Software Interrupt
    j {failure_trap_handler} // 2: Reserved
    j {failure_trap_handler} // 3: Machine Software Interrupt
    j {failure_trap_handler} // 4: User Timer Interrupt
    j {failure_trap_handler} // 5: Supervisor Timer Interrupt
    j {failure_trap_handler} // 6: Reserved
    j {success_trap_handler} // 7: Machine Timer Interrupt
    j {failure_trap_handler} // 8: User External Interrupt
    j {failure_trap_handler} // 9: Supervisor External Interrupt
    j {failure_trap_handler} // 10: Reserved
    j {failure_trap_handler} // 11: Machine External Interrupt

"#,
    failure_trap_handler = sym failure_trap_handler,
    success_trap_handler = sym success_trap_handler,
);

unsafe extern "C" {
    fn _raw_interrupt_trap_handler();
}
