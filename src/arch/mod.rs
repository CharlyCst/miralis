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

// ———————————————————————————— Privilege Modes ————————————————————————————— //

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

// —————————————————————————————— Machine ISA ——————————————————————————————— //

/// The machine ISA (misa).
#[allow(unused)]
pub mod misa {
    use crate::platform::{Plat, Platform};

    /// Atomic extension
    pub const A: usize = 1 << 0;
    /// Compression extension
    pub const C: usize = 1 << 2;
    /// Double-precision floating-point extension
    pub const D: usize = 1 << 3;
    /// RV32E base ISA
    pub const E: usize = 1 << 4;
    /// Single-precision floating-point extension
    pub const F: usize = 1 << 5;
    /// Hypervisor extension
    pub const H: usize = 1 << 7;
    /// RV32I/64I/128I base ISA
    pub const I: usize = 1 << 8;
    /// Integer Multiply/Divide extension
    pub const M: usize = 1 << 12;
    /// Quad-precision floating-point extension
    pub const Q: usize = 1 << 16;
    /// Supervisor mode implemented
    pub const S: usize = 1 << 18;
    /// User mode implemented
    pub const U: usize = 1 << 20;
    /// Non-standard extensions present
    pub const X: usize = 1 << 23;

    /// Machine XLEN (i.e. one of 32, 64 or 128 bits).
    /// For now Mirage only supports 64 bits.
    pub const MXL: usize = 0b10 << (core::mem::size_of::<usize>() * 8 - 2);

    /// Architecture extensions disabled by the current configuration
    pub const DISABLED: usize = {
        // By default we disable compressed instructions for now, because emulation and the
        // decoded assume 4 bytes instructions.
        // We also disable H mode, because we don't provide support for it right now.
        // In addition, we disable floating points because we encountered some issues with those
        // and they will require special handling when context switching from the OS (checking the
        // mstatus.FS bits).
        let mut disabled = C | D | F | H | Q;
        // For the rest we look up the configuration
        if !Plat::HAS_S_MODE {
            disabled |= S;
        }
        disabled
    };
}

// —————————————————————————————————— PMP ——————————————————————————————————— //

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
