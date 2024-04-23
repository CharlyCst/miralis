//! Architecture specific functions
//!
//! All direct interaction with RISC-V specific architecture features should live here. In the
//! future, we could emulate RISC-V instructions to enable running the monitor in user space, which
//! would be very helpful for testing purpose.

mod host;
#[cfg(not(feature = "host"))]
mod metal;
pub mod pmp;
mod registers;
mod trap;

pub use registers::{Csr, Register};
pub use trap::{MCause, TrapInfo};

use crate::virt::{ExecutionMode, VirtContext};

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
    fn read_mtvec() -> usize;
    fn read_mstatus() -> usize;
    unsafe fn set_mpp(mode: Mode);
    unsafe fn write_mstatus(mstatus: usize);
    unsafe fn write_pmpcfg(idx: usize, pmpcfg: usize);
    unsafe fn write_pmpaddr(idx: usize, pmpaddr: usize);
    unsafe fn mret() -> !;
    unsafe fn ecall();
    unsafe fn run_vcpu(ctx: &mut VirtContext);
    unsafe fn switch_from_firmware_to_payload(ctx: &mut VirtContext);
    unsafe fn switch_from_payload_to_firmware(ctx: &mut VirtContext);

    /// Return the faulting instruction at the provided exception PC.
    ///
    /// SAFETY:
    /// The trap info must correspond to a valid payload trap info, no further checks are performed.
    unsafe fn get_raw_faulting_instr(trap_info: &TrapInfo) -> usize;
}

// ———————————————————————————— Privilege Modes ————————————————————————————— //

/// Privilege modes
#[derive(Clone, Copy, Debug)]
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

    /// Returns the Mirage execution mode corresponding the virtual mode.
    pub fn to_exec_mode(self) -> ExecutionMode {
        match self {
            Mode::M => ExecutionMode::Firmware,
            _ => ExecutionMode::Payload,
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

// ————————————————————————————— Machine Status ————————————————————————————— //

/// Constants for the Machine Status (mstatus) CSR.
#[allow(unused)]
pub mod mstatus {
    /// Constant to filter out WPRI fields of mstatus
    pub const MSTATUS_FILTER: usize = 0x8000003F007FFFEA; // Todo : depends on the extensions available : Hypervisor, etc...
    /// Constant to filter out WPRI fields of sstatus
    pub const SSTATUS_FILTER: usize = 0x80000003000DE763;
    /// Constant to filter out non-writable fields of the misa csr
    pub const MISA_CHANGE_FILTER: usize = 0x0000000003FFFFFF;
    /// Constant to filter out non-writable fields of the satp csr
    pub const SATP_CHANGE_FILTER: usize = 0x00000FFFFFFFFFFF;

    // Mstatus fields constants
    /// SIE
    pub const SIE_OFFSET: usize = 1;
    pub const SIE_FILTER: usize = 0b1;
    /// MIE
    pub const MIE_OFFSET: usize = 3;
    pub const MIE_FILTER: usize = 0b1;
    /// SPIE
    pub const SPIE_OFFSET: usize = 5;
    pub const SPIE_FILTER: usize = 0b1;
    /// UBE
    pub const UBE_OFFSET: usize = 6;
    pub const UBE_FILTER: usize = 0b1;
    /// MPIE
    pub const MPIE_OFFSET: usize = 7;
    pub const MPIE_FILTER: usize = 0b1;
    /// SPP
    pub const SPP_OFFSET: usize = 8;
    pub const SPP_FILTER: usize = 0b1;
    /// VS
    pub const VS_OFFSET: usize = 9;
    pub const VS_FILTER: usize = 0b11;
    /// MPP
    pub const MPP_OFFSET: usize = 11;
    pub const MPP_FILTER: usize = 0b11;
    /// FS
    pub const FS_OFFSET: usize = 13;
    pub const FS_FILTER: usize = 0b11;
    /// XS
    pub const XS_OFFSET: usize = 15;
    pub const XS_FILTER: usize = 0b11;
    /// MPRV
    pub const MPRV_OFFSET: usize = 17;
    pub const MPRV_FILTER: usize = 0b1;
    /// SUM
    pub const SUM_OFFSET: usize = 18;
    pub const SUM_FILTER: usize = 0b1;
    /// MXR
    pub const MXR_OFFSET: usize = 19;
    pub const MXR_FILTER: usize = 0b1;
    /// TVM
    pub const TVM_OFFSET: usize = 20;
    pub const TVM_FILTER: usize = 0b1;
    /// TW
    pub const TW_OFFSET: usize = 21;
    pub const TW_FILTER: usize = 0b1;
    /// TSR
    pub const TSR_OFFSET: usize = 22;
    pub const TSR_FILTER: usize = 0b1;
    /// UXL
    pub const UXL_OFFSET: usize = 32;
    pub const UXL_FILTER: usize = 0b11;
    /// SXL
    pub const SXL_OFFSET: usize = 34;
    pub const SXL_FILTER: usize = 0b11;
    /// SBE
    pub const SBE_OFFSET: usize = 36;
    pub const SBE_FILTER: usize = 0b1;
    /// MBE
    pub const MBE_OFFSET: usize = 37;
    pub const MBE_FILTER: usize = 0b1;
    /// MPV
    pub const MPV_OFFSET: usize = 39;
    pub const MPV_FILTER: usize = 0b1;
    /// SD
    pub const SD_OFFSET: usize = 63;
    pub const SD_FILTER: usize = 0b1;
}
