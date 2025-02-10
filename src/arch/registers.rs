//! RISC-V Registers

/// General purpose registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    /// zero - Hard-wired zero register
    X0 = 0,
    /// ra - Return address
    X1 = 1,
    /// sp - Stack pointer
    X2 = 2,
    /// gp - Global pointer
    X3 = 3,
    /// tp - Thread pointer
    X4 = 4,
    /// t0 - Temporary register
    X5 = 5,
    /// t1 - Temporary register
    X6 = 6,
    /// t2 - Temporary register
    X7 = 7,
    /// s0 / fp Saved register / frame pointer
    X8 = 8,
    /// s1 - Saved register
    X9 = 9,
    /// a0 - Function argument / return value
    X10 = 10,
    /// a1 - Function argument / return value
    X11 = 11,
    /// a2 - Function argument
    X12 = 12,
    /// a3 - Function argument
    X13 = 13,
    /// a4 - Function argument
    X14 = 14,
    /// a5 - Function argument
    X15 = 15,
    /// a6 - Function argument
    X16 = 16,
    /// a7 - Function argument
    X17 = 17,
    /// s2 - Saved register
    X18 = 18,
    /// s3 - Saved register
    X19 = 19,
    /// s4 - Saved register
    X20 = 20,
    /// s5 - Saved register
    X21 = 21,
    /// s6 - Saved register
    X22 = 22,
    /// s7 - Saved register
    X23 = 23,
    /// s8 - Saved register
    X24 = 24,
    /// s9 - Saved register
    X25 = 25,
    /// s10 - Saved register
    X26 = 26,
    /// s11 - Saved register
    X27 = 27,
    /// t3 - Temporary register
    X28 = 28,
    /// t4 - Temporary register
    X29 = 29,
    /// t5 - Temporary register
    X30 = 30,
    /// t6 - Temporary register
    X31 = 31,
}

impl Register {
    /// Convert a `usize` to a register by masking high order bits.
    pub fn from(value: usize) -> Self {
        Register::try_from(value & 0b11111).unwrap()
    }
}

/// A RISC-V Control and Status Register (CSR).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Csr {
    // Machine mode CSRs
    //
    /// Machine Hart ID
    Mhartid,
    /// Machine Status
    Mstatus,
    /// Machine ISA extensions
    Misa,
    /// Machine Interrupt Enable
    Mie,
    /// Machine Trap Vector
    Mtvec,
    /// Machine Scratch
    Mscratch,
    /// Machine Interrupt Pending
    Mip,
    /// Machine Vendor ID
    Mvendorid,
    /// Machine Architecture ID
    Marchid,
    /// Machine Implementation ID
    Mimpid,
    /// PMP config
    Pmpcfg(usize),
    /// PMP addr
    Pmpaddr(usize),
    /// Machine cycle counter
    Mcycle,
    /// Machine instructions-retired counter
    Minstret,
    /// Cycle register
    Cycle,
    /// Time register
    Time,
    /// Instret register
    Instret,
    /// Machine performance-monitoring counter
    Mhpmcounter(usize),
    /// Machine counter-inhibit register
    Mcountinhibit,
    /// Machine performance-monitoring event selector
    Mhpmevent(usize),
    /// Machine counter enable
    Mcounteren,
    /// Machine environment configuration register
    Menvcfg,
    /// Machine security configuration register
    Mseccfg,
    /// Ponter to configuration data structure
    Mconfigptr,
    /// Machine exception delegation register
    Medeleg,
    /// Machine interrupt delegation register
    Mideleg,
    /// Machine trap instruction (transformed)
    Mtinst,
    /// Machine bad guest physical address
    Mtval2,
    /// Debug/Trace trigger register select
    Tselect,
    /// First Debug/Trace trigger data register
    Tdata1,
    /// Second Debug/Trace trigger data register
    Tdata2,
    /// Third Debug/Trace trigger data register
    Tdata3,
    /// Machine-mode context register
    Mcontext,
    /// Debug control and status register
    Dcsr,
    /// Debug PC
    Dpc,
    /// Debug scratch register 0
    Dscratch0,
    /// Debug scratch register 1
    Dscratch1,
    /// Machine exception program counter
    Mepc,
    /// Machine trap cause
    Mcause,
    /// Machine bad address or instruction
    Mtval,

    // Supervisor mode CSRs
    //
    /// Supervisor status register
    Sstatus,
    /// Supervisor interrupt-enable register
    Sie,
    /// Supervisor trap handler base address
    Stvec,
    /// Supervisor counter enable
    Scounteren,
    /// Supervisor environment configuration register
    Senvcfg,
    /// Scratch register for supervisor trap handlers
    Sscratch,
    /// Supervisor exception program counter
    Sepc,
    /// Supervisor trap cause
    Scause,
    /// Supervisor bad address or instruction
    Stval,
    /// Supervisor interrupt pending
    Sip,
    /// Supervisor address translation and protection
    Satp,
    /// Supervisor-mode context register
    Scontext,
    /// Supervisor timer compare
    Stimecmp,

    // Hypervisor and Virtual Supervisor CSRs
    //
    /// Hypervisor Status Register
    Hstatus,
    /// Hypervisor Exception Delegation Register
    Hedeleg,
    /// Hypervisor Interrupt Delegation Register
    Hideleg,
    /// Hypervisor Virtual Interrupt Pending Register
    Hvip,
    /// Hypervisor Interrupt Pending Register
    Hip,
    /// Hypervisor Interrupt Enable Register
    Hie,
    /// Hypervisor Guest External Interrupt Pending Register
    Hgeip,
    /// Hypervisor Guest External Interrupt Enable Register
    Hgeie,
    /// Hypervisor Environment Configuration Register
    Henvcfg,
    /// Hypervisor Counter-Enable Register
    Hcounteren,
    /// Hypervisor Time Delta Register
    Htimedelta,
    /// Hypervisor Trap Value Register
    Htval,
    /// Hypervisor Trap Instruction Register
    Htinst,
    /// Hypervisor Guest Address Translation and Protection Register
    Hgatp,

    /// Virtual Supervisor Status Register
    Vsstatus,
    /// Virtual Supervisor Interrupt Enable
    Vsie,
    /// Virtual Supervisor Trap-Vector Base Address
    Vstvec,
    /// Virtual Supervisor Scratch Register
    Vsscratch,
    /// Virtual Supervisor Exception Program Counter
    Vsepc,
    /// Virtual Supervisor Cause Register
    Vscause,
    /// Virtual Supervisor Trap Value Register
    Vstval,
    /// Virtual Supervisor Interrupt Pending Register
    Vsip,
    /// Virtual Supervisor Address Translation and Protection
    Vsatp,

    /// Vector extension
    ///
    /// Vector Start Index CSR
    Vstart,
    /// Vector Fixed-Point Saturation Flag
    Vxsat,
    /// Vector Fixed-Point Rounding Mode Register
    Vxrm,
    /// Vector Control and Status Register
    Vcsr,
    /// Vector Length Register
    Vl,
    /// Vector Type Register
    Vtype,
    /// Vector Byte Length
    Vlenb,

    /// Crypto extension
    ///
    /// Seed register
    Seed,

    /// Custom
    ///
    /// Those CSRs are specific to each SoC, refer to the corresponding manual for details.
    /// The `usize` fields encode the CSR ID.
    Custom(usize),

    /// An unknown CSR
    Unknown,
}

impl Csr {
    pub const PMP_CFG_LOCK_MASK: usize = (0b1 << 7)
        | (0b1 << 7) << 8
        | (0b1 << 7) << 16
        | (0b1 << 7) << 24
        | (0b1 << 7) << 32
        | (0b1 << 7) << 40
        | (0b1 << 7) << 48
        | (0b1 << 7) << 56;

    pub const PMP_CFG_LEGAL_MASK: usize = !((0b11 << 5)
        | (0b11 << 5) << 8
        | (0b11 << 5) << 16
        | (0b11 << 5) << 24
        | (0b11 << 5) << 32
        | (0b11 << 5) << 40
        | (0b11 << 5) << 48
        | (0b11 << 5) << 56);

    pub const PMP_ADDR_LEGAL_MASK: usize = !(0b1111111111 << 54);

    #[allow(unused)] // TODO: remove once used
    pub const MCOUNTINHIBIT_LEGAL_MASK: usize = !(0b10);

    pub fn is_unknown(self) -> bool {
        self == Csr::Unknown
    }
}

// —————————————————————————————— Conversions ——————————————————————————————— //

impl TryFrom<usize> for Register {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::X0),
            1 => Ok(Register::X1),
            2 => Ok(Register::X2),
            3 => Ok(Register::X3),
            4 => Ok(Register::X4),
            5 => Ok(Register::X5),
            6 => Ok(Register::X6),
            7 => Ok(Register::X7),
            8 => Ok(Register::X8),
            9 => Ok(Register::X9),
            10 => Ok(Register::X10),
            11 => Ok(Register::X11),
            12 => Ok(Register::X12),
            13 => Ok(Register::X13),
            14 => Ok(Register::X14),
            15 => Ok(Register::X15),
            16 => Ok(Register::X16),
            17 => Ok(Register::X17),
            18 => Ok(Register::X18),
            19 => Ok(Register::X19),
            20 => Ok(Register::X20),
            21 => Ok(Register::X21),
            22 => Ok(Register::X22),
            23 => Ok(Register::X23),
            24 => Ok(Register::X24),
            25 => Ok(Register::X25),
            26 => Ok(Register::X26),
            27 => Ok(Register::X27),
            28 => Ok(Register::X28),
            29 => Ok(Register::X29),
            30 => Ok(Register::X30),
            31 => Ok(Register::X31),
            _ => Err(()),
        }
    }
}
