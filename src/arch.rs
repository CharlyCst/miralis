//! Architecture specific functions
//!
//! All direct interaction with RISC-V specific architecture features should live here. In the
//! future, we could emulate RISC-V instructions to enable running the monitor in user space, which
//! would be very helpful for testing purpose.

use core::arch::{asm, global_asm};

use crate::trap::{trap_handler, MCause};

/// Export the current architecture.
/// For now, only bare-metal is supported
pub type Arch = Metal;

/// Architecture abstraction layer.
pub trait Architecture {
    fn init();
    fn read_mstatus() -> usize;
    fn read_mcause() -> MCause;
    fn read_mepc() -> usize;
    fn read_mtval() -> usize;
    unsafe fn write_mepc(mepc: usize);
    unsafe fn mret();
    unsafe fn ecall();
}

// ——————————————————————————————— Bare Metal ——————————————————————————————— //

/// Bare metal RISC-V runtime.
pub struct Metal {}

impl Architecture for Metal {
    fn init() {
        // Set trap handler
        let handler = _raw_trap_handler as usize;
        unsafe { write_mtvec(handler) };
        let mtvec = read_mtvec();
        assert_eq!(handler, mtvec, "Failed to set trap handler");

        // Try trapping, just to test :)
        unsafe {
            Self::ecall();
        }
    }

    fn read_mstatus() -> usize {
        let mstatus: usize;
        unsafe {
            asm!(
                "csrr {x}, mstatus",
                x = out(reg) mstatus);
        }
        return mstatus;
    }

    fn read_mcause() -> MCause {
        let mcause: usize;
        unsafe {
            asm!(
                "csrr {x}, mcause",
                x = out(reg) mcause);
        }
        return MCause::new(mcause);
    }

    fn read_mepc() -> usize {
        let mepc: usize;
        unsafe {
            asm!(
                "csrr {x}, mepc",
                x = out(reg) mepc);
        }
        return mepc;
    }

    fn read_mtval() -> usize {
        let mtval: usize;
        unsafe {
            asm!(
                "csrr {x}, mtval",
                x = out(reg) mtval);
        }
        return mtval;
    }

    unsafe fn write_mepc(mepc: usize) {
        asm!(
            "csrw mepc, {x}",
            x = in(reg) mepc
        )
    }

    unsafe fn mret() {
        asm!("mret")
    }

    unsafe fn ecall() {
        asm!("ecall")
    }
}

unsafe fn write_mtvec(value: usize) {
    asm!(
        "csrw mtvec, {x}",
        x = in(reg) value
    )
}

fn read_mtvec() -> usize {
    let mtvec: usize;
    unsafe {
        asm!(
            "csrr {x}, mtvec",
            x = out(reg) mtvec
        )
    }
    return mtvec;
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(r#"
.text
.align 4
.global _raw_trap_handler
_raw_trap_handler:
    j {handler}
"#,
    handler = sym trap_handler,
);

extern "C" {
    fn _raw_trap_handler();
}
