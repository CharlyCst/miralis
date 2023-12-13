//! Architecture specific functions
//!
//! All direct interaction with RISC-V specific architecture features should live here. In the
//! future, we could emulate RISC-V instructions to enable running the monitor in user space, which
//! would be very helpful for testing purpose.

use core::arch::asm;

/// Export the current architecture.
/// For now, only bare-metal is supported
pub type Arch = Metal;

/// Architecture abstraction layer.
pub trait Architecture {
    fn read_mstatus() -> usize;
}

/// Bare metal RISC-V runtime.
pub struct Metal {}

impl Architecture for Metal {
    fn read_mstatus() -> usize {
        let mstatus: usize;
        unsafe {
            asm!(
                "csrr {x}, mstatus",
                x = out(reg) mstatus)
        }
        return mstatus;
    }
}
