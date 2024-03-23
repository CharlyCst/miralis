//! Architecture specific functions
//!
//! All direct interaction with RISC-V specific architecture features should live here. In the
//! future, we could emulate RISC-V instructions to enable running the monitor in user space, which
//! would be very helpful for testing purpose.

mod host;
#[cfg(not(feature = "host"))]
mod metal;
mod registers;
mod trap;

pub use registers::{Csr, Register};
pub use trap::{MCause, TrapInfo};

use crate::virt::VirtContext;

// —————————————————————————— Select Architecture ——————————————————————————— //

/// Risc-V bare-metal M-mode architecture.
#[cfg(not(feature = "host"))]
pub type Arch = metal::MetalArch;

/// Host architecture, running in userspace.
#[cfg(feature = "host")]
pub type Arch = host::HostArch;

// ———————————————————————— Architecture Definition ————————————————————————— //

/// Architecture abstraction layer.
pub trait Architecture {
    fn init();
    fn read_misa() -> usize;
    fn read_mstatus() -> usize;
    unsafe fn set_mpp(mode: Mode);
    unsafe fn write_misa(misa: usize);
    unsafe fn write_mstatus(mstatus: usize);
    unsafe fn write_pmpcfg(idx: usize, pmpcfg: usize);
    unsafe fn write_pmpaddr(idx: usize, pmpaddr: usize);
    unsafe fn mret() -> !;
    unsafe fn ecall();
    unsafe fn enter_virt_firmware(ctx: &mut VirtContext);

    /// Return the faulting instruction at the provided exception PC.
    ///
    /// SAFETY:
    /// The trap info must correspond to a valid payload trap info, no further checks are performed.
    unsafe fn get_raw_faulting_instr(trap_info: &TrapInfo) -> usize;
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
