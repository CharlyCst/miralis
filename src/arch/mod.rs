//! Architecture specific functions
//!
//! All direct interaction with RISC-V specific architecture features should live here. In the
//! future, we could emulate RISC-V instructions to enable running the monitor in user space, which
//! would be very helpful for testing purpose.

mod metal;
mod registers;
mod trap;

pub use registers::{Csr, Register};
pub use trap::MCause;

use crate::virt::VirtContext;

/// Export the current architecture.
/// For now, only bare-metal is supported
pub type Arch = metal::Metal;

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
    unsafe fn enter_virt_firmware(ctx: &mut VirtContext);

    /// Return the faulting instruction.
    ///
    /// Assumptions:
    /// - The last trap is an illegal instruction trap (thus the instruction might be stored into
    ///   mtval)
    /// - The value of mepc has not been  modified since last trap, as this function might need to
    ///   read memory at mepc.
    unsafe fn get_raw_faulting_instr() -> usize;
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
