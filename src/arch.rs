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
    unsafe fn set_mpp(mode: Mode);
    unsafe fn write_mepc(mepc: usize);
    unsafe fn write_mstatus(mstatus: usize);
    unsafe fn write_pmpcfg(idx: usize, pmpcfg: usize);
    unsafe fn write_pmpaddr(idx: usize, pmpaddr: usize);
    unsafe fn mret() -> !;
    unsafe fn ecall();
}

/// Privilege modes
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Mode {
    /// User
    U,
    /// Supervisor
    S,
    /// Machine
    M,
}

impl Mode {
    /// Returns the bit pattern corresponding to the given mode.
    pub fn to_bits(self) -> usize {
        match self {
            Mode::U => 0,
            Mode::S => 1,
            Mode::M => 3,
        }
    }
}

/// PMP Configuration
///
/// Hold constants for the pmpcfg CSRs.
#[allow(unused)]
pub mod pmpcfg {
    /// Read access
    pub const R: usize = 0b00000001;
    /// Write access
    pub const W: usize = 0b00000010;
    /// Execute access
    pub const X: usize = 0b00000100;

    /// Region is not active
    pub const OFF: usize = 0b00000000;
    /// Address is Top Of Range (TOP)
    pub const TOR: usize = 0b00001000;
    /// Naturally aligned four-byte region
    pub const NA4: usize = 0b00010000;
    /// Naturally aligned power of two
    pub const NAPOT: usize = 0b00011000;

    /// Locked
    pub const L: usize = 0b10000000;
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

    unsafe fn set_mpp(mode: Mode) {
        const MPP_MASK: usize = 0b11_usize << 11;
        let value = mode.to_bits() << 11;
        let mstatus = Self::read_mstatus();
        Self::write_mstatus((mstatus & !MPP_MASK) | value)
    }

    unsafe fn write_mepc(mepc: usize) {
        asm!(
            "csrw mepc, {x}",
            x = in(reg) mepc
        )
    }

    unsafe fn write_mstatus(mstatus: usize) {
        asm!(
            "csrw mstatus, {x}",
            x = in(reg) mstatus
        )
    }

    unsafe fn write_pmpcfg(idx: usize, pmpcfg: usize) {
        match idx {
            0 => {
                asm!(
                    "csrw pmpcfg0, {x}",
                    x = in(reg) pmpcfg
                )
            }
            _ => todo!("pmpcfg{} not yet implemented", idx),
        }
    }

    unsafe fn write_pmpaddr(idx: usize, pmpaddr: usize) {
        match idx {
            0 => {
                asm!(
                    "csrw pmpaddr0, {x}",
                    x = in(reg) pmpaddr
                )
            }
            1 => {
                asm!(
                    "csrw pmpaddr1, {x}",
                    x = in(reg) pmpaddr
                )
            }
            _ => todo!("pmpaddr{} not yet implemented", idx),
        }
    }

    unsafe fn mret() -> ! {
        asm!("mret", options(noreturn))
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
