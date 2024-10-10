//! Architecture specific functions
//!
//! All direct interaction with RISC-V specific architecture features should live here. In the
//! future, we could emulate RISC-V instructions to enable running the monitor in user space, which
//! would be very helpful for testing purpose.

#[cfg(not(feature = "userspace"))]
mod metal;
pub mod pmp;
mod registers;
mod trap;
mod userspace;

use pmp::{PmpFlush, PmpGroup};
pub use registers::{Csr, Register};
pub use trap::{MCause, TrapInfo};

use crate::arch::mstatus::{MPP_FILTER, MPP_OFFSET};
use crate::decoder::Instr;
use crate::utils::PhantomNotSendNotSync;
use crate::virt::{ExecutionMode, VirtContext};

// —————————————————————————— Select Architecture ——————————————————————————— //

/// Risc-V bare-metal M-mode architecture.
#[cfg(not(feature = "userspace"))]
pub type Arch = metal::MetalArch;

/// Host architecture, running in userspace.
#[cfg(feature = "userspace")]
pub type Arch = userspace::HostArch;

// ———————————————————————— Architecture Definition ————————————————————————— //

/// Architecture abstraction layer.
pub trait Architecture {
    fn init();

    /// Read a csr value
    fn read_csr(csr: Csr) -> usize;

    /// Write into csr and return previous value
    unsafe fn write_csr(csr: Csr, value: usize) -> usize;

    /// Clear csr_bits with mask
    unsafe fn clear_csr_bits(csr: Csr, bits_mask: usize);

    /// Set csr_bits with mask
    unsafe fn set_csr_bits(csr: Csr, bits_mask: usize);

    /// Change mstatus.MPP and return the previous mstatus.MPP
    unsafe fn set_mpp(mode: Mode) -> Mode;
    unsafe fn write_pmp(pmp: &PmpGroup) -> PmpFlush;
    unsafe fn sfencevma(vaddr: Option<usize>, asid: Option<usize>);
    unsafe fn hfencegvma(vaddr: Option<usize>, asid: Option<usize>);
    unsafe fn hfencevvma(vaddr: Option<usize>, asid: Option<usize>);
    unsafe fn run_vcpu(ctx: &mut VirtContext);

    /// Wait for interrupt
    fn wfi();

    /// Install a trap handler
    fn install_handler(handler: usize);

    /// Detect available hardware capabilities.
    ///
    /// Capabilities are local to a core: two cores (harts in RISC-V parlance) can have different
    /// sets of capabilities. This is modelled by the fact that [HardwareCapability] does not
    /// implement Send and Sync, meaning that it can't be shared across cores (which is enforced by
    /// the compiler and invariants in unsafe code).
    ///
    /// SAFETY:
    /// This function might temporarily change the state of the hart during the detection process.
    /// For this reason it is only safe to execute as part of the core initialization, not during
    /// standard operations.
    /// It should not be assume that any of the core configuration is preserved by this function.
    unsafe fn detect_hardware() -> HardwareCapability;

    /// Return the faulting instruction at the provided exception PC.
    ///
    /// SAFETY:
    /// The trap info must correspond to a valid trap info, no further checks are performed.
    unsafe fn get_raw_faulting_instr(trap_info: &TrapInfo) -> usize;

    /// SAFETY:
    /// None so far, TODO
    unsafe fn handle_virtual_load_store(instr: Instr, ctx: &mut VirtContext);

    /// Copies dest.len() bytes from src to dest, using the provided mode to read from src.
    /// This function can be useful to copy bytes from the virtual address space of a lower
    /// privileged mode, to a buffer in M-mode.
    ///
    /// Returns whether the copy succeeded or not (for example, the copy might not succeed if we try
    /// to read an address not accessible from the given mode).
    unsafe fn read_bytes_from_mode(src: *const u8, dest: &mut [u8], mode: Mode) -> Result<(), ()>;

    /// This function is similar to the function above except it is used to store bytes in virtual memory from a chphysical address.
    unsafe fn store_bytes_from_mode(src: &mut [u8], dest: *const u8, mode: Mode) -> Result<(), ()>;
}

// ——————————————————————————— Hardware Detection ——————————————————————————— //

/// A struct that contains information about the hardware capability.
///
/// This struct has to be local to a core (it is !Send and !Sync) and can be obtained though
/// hardware capability detection using the [Architecture] trait.
#[derive(Debug, Clone)]
pub struct HardwareCapability {
    /// Bitmap of valid interrupts, marks valid bits in `mie` and `mip`.
    pub interrupts: usize,
    /// Structure indicating the presence of optional registers.
    pub available_reg: RegistersCapability,
    /// Structure indicating the presence of optional extensions.
    pub extensions: ExtensionsCapability,
    /// The hart ID, as read from mhartid.
    pub hart: usize,
    /// Prevent the struct from being used on another core.
    _marker: PhantomNotSendNotSync,
}

/// A struct that contains information about the available registers
#[derive(Debug, Clone)]
pub struct RegistersCapability {
    /// Boolean value indicating if Machine environment configuration register is present
    pub menvcfg: bool,
    /// Boolean value indicating if Supervisor environment configuration register is present
    pub senvcfg: bool,
    /// The number of implemented and non-zero PMP registers
    pub nb_pmp: usize,
}

/// A struct that contains information about the available extensions
#[derive(Debug, Clone)]
pub struct ExtensionsCapability {
    /// Hypervisor extension
    pub has_h_extension: bool,
    /// Supervisor extension
    pub has_s_extension: bool,
    /// Single precision floating point extension
    pub _has_f_extension: bool,
    /// Double precision floating point extension
    pub _has_d_extension: bool,
    /// Quadruple precision floating point extension
    pub _has_q_extension: bool,
}

// ———————————————————————————— Privilege Modes ————————————————————————————— //

/// Privilege modes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// User
    U,
    /// Supervisor
    S,
    /// Machine
    M,
}

/// Returns the mode corresponding to the bit pattern
pub fn parse_mpp_return_mode(mstatus_reg: usize) -> Mode {
    match (mstatus_reg & MPP_FILTER) >> MPP_OFFSET {
        0 => Mode::U,
        1 => Mode::S,
        3 => Mode::M,
        _ => panic!("Unknown mode!"),
    }
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

    /// Returns the Miralis execution mode corresponding the virtual mode.
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
    /// For now Miralis only supports 64 bits.
    pub const MXL: usize = 0b10 << (core::mem::size_of::<usize>() * 8 - 2);

    /// Architecture extensions disabled by the current configuration
    pub const DISABLED: usize = {
        // By default we disable compressed instructions for now, because emulation and the
        // decoded assume 4 bytes instructions.
        // We also disable H mode, because we don't provide support for it right now.
        // In addition, we disable floating points because we encountered some issues with those
        // and they will require special handling when context switching from the OS (checking the
        // mstatus.FS bits).
        C | D | F | Q
    };

    /// Constant to filter out non-writable fields of the misa csr
    pub const MISA_CHANGE_FILTER: usize = 0x0000000003FFFFFF;
}

// ————————————— Supervisor Address Translation and Protection —————————————— //

#[allow(unused)]
pub mod satp {
    /// Constant to filter out non-writable fields of the satp csr
    pub const SATP_CHANGE_FILTER: usize = 0x00000FFFFFFFFFFF;
}

// ————————————————————————————— Machine Status ————————————————————————————— //

/// Constants for the Machine Status (mstatus) CSR.
#[allow(unused)]
pub mod mstatus {
    /// Constant to filter out WPRI fields of mstatus
    // Todo : depends on the extensions available : Hypervisor, etc...
    pub const MSTATUS_FILTER: usize = SSTATUS_FILTER
        | MIE_FILTER
        | MPIE_FILTER
        | MPP_FILTER
        | MPRV_FILTER
        | TVM_FILTER
        | TW_FILTER
        | TSR_FILTER
        | SXL_FILTER
        | SBE_FILTER
        | MBE_FILTER;

    /// Constant to filter out WPRI fields of sstatus
    pub const SSTATUS_FILTER: usize = SIE_FILTER
        | SPIE_FILTER
        | UBE_FILTER
        | SPP_FILTER
        | VS_FILTER
        | FS_FILTER
        | XS_FILTER
        | SUM_FILTER
        | MXR_FILTER
        | UXL_FILTER
        | SD_FILTER;

    // Mstatus fields constants
    /// SIE
    pub const SIE_OFFSET: usize = 1;
    pub const SIE_FILTER: usize = 0b1 << SIE_OFFSET;
    /// MIE
    pub const MIE_OFFSET: usize = 3;
    pub const MIE_FILTER: usize = 0b1 << MIE_OFFSET;
    /// SPIE
    pub const SPIE_OFFSET: usize = 5;
    pub const SPIE_FILTER: usize = 0b1 << SPIE_OFFSET;
    /// UBE
    pub const UBE_OFFSET: usize = 6;
    pub const UBE_FILTER: usize = 0b1 << UBE_OFFSET;
    /// MPIE
    pub const MPIE_OFFSET: usize = 7;
    pub const MPIE_FILTER: usize = 0b1 << MPIE_OFFSET;
    /// SPP
    pub const SPP_OFFSET: usize = 8;
    pub const SPP_FILTER: usize = 0b1 << SPP_OFFSET;
    /// VS
    pub const VS_OFFSET: usize = 9;
    pub const VS_FILTER: usize = 0b11 << VS_OFFSET;
    /// MPP
    pub const MPP_OFFSET: usize = 11;
    pub const MPP_FILTER: usize = 0b11 << MPP_OFFSET;
    /// FS
    pub const FS_OFFSET: usize = 13;
    pub const FS_FILTER: usize = 0b11 << FS_OFFSET;
    /// XS
    pub const XS_OFFSET: usize = 15;
    pub const XS_FILTER: usize = 0b11 << XS_OFFSET;
    /// MPRV
    pub const MPRV_OFFSET: usize = 17;
    pub const MPRV_FILTER: usize = 0b1 << MPRV_OFFSET;
    /// SUM
    pub const SUM_OFFSET: usize = 18;
    pub const SUM_FILTER: usize = 0b1 << SUM_OFFSET;
    /// MXR
    pub const MXR_OFFSET: usize = 19;
    pub const MXR_FILTER: usize = 0b1 << MXR_OFFSET;
    /// TVM
    pub const TVM_OFFSET: usize = 20;
    pub const TVM_FILTER: usize = 0b1 << TVM_OFFSET;
    /// TW
    pub const TW_OFFSET: usize = 21;
    pub const TW_FILTER: usize = 0b1 << TW_OFFSET;
    /// TSR
    pub const TSR_OFFSET: usize = 22;
    pub const TSR_FILTER: usize = 0b1 << TSR_OFFSET;
    /// UXL
    pub const UXL_OFFSET: usize = 32;
    pub const UXL_FILTER: usize = 0b11 << UXL_OFFSET;
    /// SXL
    pub const SXL_OFFSET: usize = 34;
    pub const SXL_FILTER: usize = 0b11 << SXL_OFFSET;
    /// SBE
    pub const SBE_OFFSET: usize = 36;
    pub const SBE_FILTER: usize = 0b1 << SBE_OFFSET;
    /// MBE
    pub const MBE_OFFSET: usize = 37;
    pub const MBE_FILTER: usize = 0b1 << MBE_OFFSET;
    /// MPV
    pub const MPV_OFFSET: usize = 39;
    pub const MPV_FILTER: usize = 0b1 << MPV_OFFSET;
    /// SD
    pub const SD_OFFSET: usize = 63;
    pub const SD_FILTER: usize = 0b1 << SD_OFFSET;
}

// ———————————————————————— Machine Interrupt-Enabled ——————————————————————— //

#[allow(unused)]
pub mod mie {
    /// Constant to filter out SIE bits of mstatus
    pub const SIE_FILTER: usize = SSIE_FILTER | STIE_FILTER | SEIE_FILTER | LCOFIE_FILTER;

    /// Constant to filter out writable bits of mie.
    pub const MIE_WRITE_FILTER: usize = SIE_FILTER | MSIE_FILTER | MTIE_FILTER | MEIE_FILTER;

    /// Constant to filter out writable bits of mip.
    pub const MIP_WRITE_FILTER: usize = SSIE_FILTER | STIE_FILTER | SEIE_FILTER;

    /// The bits in mideleg that must be read-only one.
    ///
    /// Some interrupts are forced to be delegated to S-mode because Miralis doesn't implement
    /// virtualization for them (as that would incur a cost in terms of complexity and
    /// performance).
    pub const MIDELEG_READ_ONLY_ONE: usize =
        SSIE_FILTER | STIE_FILTER | SEIE_FILTER | LCOFIE_FILTER;

    /// The bits in mideleg that are read-only zero
    ///
    /// The corresponding interrupts are virtualized by Miralis. For now Miralis only virtualizes
    /// M-mode interrupts.
    pub const MIDELEG_READ_ONLY_ZERO: usize = MSIE_FILTER | MTIE_FILTER | MEIE_FILTER;

    // Mie fields constants
    /// SSIE
    pub const SSIE_OFFSET: usize = 1;
    pub const SSIE_FILTER: usize = 0b1 << SSIE_OFFSET;
    /// MSIE
    pub const MSIE_OFFSET: usize = 3;
    pub const MSIE_FILTER: usize = 0b1 << MSIE_OFFSET;
    /// STIE
    pub const STIE_OFFSET: usize = 5;
    pub const STIE_FILTER: usize = 0b1 << STIE_OFFSET;
    /// MTIE
    pub const MTIE_OFFSET: usize = 7;
    pub const MTIE_FILTER: usize = 0b1 << MTIE_OFFSET;
    /// SEIE
    pub const SEIE_OFFSET: usize = 9;
    pub const SEIE_FILTER: usize = 0b1 << SEIE_OFFSET;
    /// MEIE
    pub const MEIE_OFFSET: usize = 11;
    pub const MEIE_FILTER: usize = 0b1 << MEIE_OFFSET;
    /// LCOFIE
    pub const LCOFIE_OFFSET: usize = 13;
    pub const LCOFIE_FILTER: usize = 0b1 << LCOFIE_OFFSET;
}

// ———————————————————— Machine Trap-Vector Base-Address ———————————————————— //

#[allow(unused)]
pub mod mtvec {
    /// Constant to filter out MODE bits of mtvec
    pub const MODE_FILTER: usize = 0b11;

    /// Constant to filter out BASE bits of mtvec
    pub const BASE_FILTER: usize = usize::MAX & !MODE_FILTER;

    /// Privilege modes
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Mode {
        /// User
        Direct = 0,
        /// Machine
        Vectored = 1,
    }

    pub fn get_mode(mtvec: usize) -> Mode {
        match mtvec & MODE_FILTER {
            0 => Mode::Direct,
            1 => Mode::Vectored,
            _ => panic!("Invalid trap-vector mode."),
        }
    }
}

// ————————————————————————————— Hypervisor Status ————————————————————————————— //

/// Constants for the Machine Status (mstatus) CSR.
#[allow(unused)]
pub mod hstatus {

    // VSBE
    pub const VSBE_OFFSET: usize = 5;
    pub const VSBE_FILTER: usize = 0b1 << VSBE_OFFSET;

    // TVM
    pub const VTVM_OFFSET: usize = 20;
    pub const VTVM_FILTER: usize = 0b1 << VTVM_OFFSET;

    // TW
    pub const VTW_OFFSET: usize = 21;
    pub const VTW_FILTER: usize = 0b1 << VTW_OFFSET;

    // TSR
    pub const VTSR_OFFSET: usize = 22;
    pub const VTSR_FILTER: usize = 0b1 << VTSR_OFFSET;

    // VSXL
    pub const VSXL_OFFSET: usize = 32;
    pub const VSXL_FILTER: usize = 0b11 << VSXL_OFFSET;
}

// ——————————————————————— Width of Access Instructions —————————————————————— //

/// Represents different data widths:
///  - `Byte`: 8 bits (1 byte)
///  - `Byte2`: 16 bits (2 bytes)
///  - `Byte4`: 32 bits (4 bytes)
///  - `Byte8`: 64 bits (8 bytes)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Width {
    Byte = 8,
    Byte2 = 16,
    Byte4 = 32,
    Byte8 = 64,
}

impl Width {
    pub fn to_bits(self) -> usize {
        self as usize
    }

    pub fn to_bytes(self) -> usize {
        self.to_bits() / 8
    }
}

impl From<usize> for Width {
    fn from(value: usize) -> Self {
        match value {
            8 => Width::Byte,
            16 => Width::Byte2,
            32 => Width::Byte4,
            64 => Width::Byte8,
            _ => panic!("Invalid width value"),
        }
    }
}
