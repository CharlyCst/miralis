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
    /// a6 - Function argument - Encodes FID
    X16 = 16,
    /// a7 - Function argument - Encodes EID
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

/// Module with the CSR registers indexes.
pub mod csr {
    // Machine mode CSRs
    pub const MSTATUS: usize = 0x300;
    pub const MISA: usize = 0x301;
    pub const MEDELEG: usize = 0x302;
    pub const MIDELEG: usize = 0x303;
    pub const MIE: usize = 0x304;
    pub const MTVEC: usize = 0x305;
    pub const MCOUNTEREN: usize = 0x306;
    pub const MENVCFG: usize = 0x30A;
    pub const MCOUNTINHIBIT: usize = 0x320;
    pub const MHPMEVENT3: usize = 0x323;
    pub const MHPMEVENT31: usize = 0x33F;
    pub const MSCRATCH: usize = 0x340;
    pub const MEPC: usize = 0x341;
    pub const MCAUSE: usize = 0x342;
    pub const MTVAL: usize = 0x343;
    pub const MIP: usize = 0x344;
    pub const MTINST: usize = 0x34A;
    pub const MTVAL2: usize = 0x34B;
    pub const PMPCFG0: usize = 0x3A0;
    pub const PMPCFG15: usize = 0x3AF;
    pub const PMPADDR0: usize = 0x3B0;
    pub const PMPADDR63: usize = 0x3EF;
    pub const MSECCFG: usize = 0x747;
    pub const TSELECT: usize = 0x7A0;
    pub const TDATA1: usize = 0x7A1;
    pub const TDATA2: usize = 0x7A2;
    pub const TDATA3: usize = 0x7A3;
    pub const MCONTEXT: usize = 0x7A8;
    pub const DCSR: usize = 0x7B0;
    pub const DPC: usize = 0x7B1;
    pub const DSCRATCH0: usize = 0x7B2;
    pub const DSCRATCH1: usize = 0x7B3;
    pub const MCYCLE: usize = 0xB00;
    pub const MINSTRET: usize = 0xB02;
    pub const MHPMCOUNTER3: usize = 0xB03;
    pub const MHPMCOUNTER31: usize = 0xB1F;
    pub const CYCLE: usize = 0xC00;
    pub const TIME: usize = 0xC01;
    pub const INSTRET: usize = 0xC02;
    pub const VL: usize = 0xC20;
    pub const VTYPE: usize = 0xC21;
    pub const VLENB: usize = 0xC22;
    pub const MVENDORID: usize = 0xF11;
    pub const MARCHID: usize = 0xF12;
    pub const MIMPID: usize = 0xF13;
    pub const MHARTID: usize = 0xF14;
    pub const MCONFIGPTR: usize = 0xF15;

    // Supervisor mode CSRs
    pub const SSTATUS: usize = 0x100;
    pub const SIE: usize = 0x104;
    pub const STVEC: usize = 0x105;
    pub const SCOUNTEREN: usize = 0x106;
    pub const SENVCFG: usize = 0x10A;
    pub const SSCRATCH: usize = 0x140;
    pub const SEPC: usize = 0x141;
    pub const SCAUSE: usize = 0x142;
    pub const STVAL: usize = 0x143;
    pub const SIP: usize = 0x144;
    pub const STIMECMP: usize = 0x14D;
    pub const SATP: usize = 0x180;
    pub const SCONTEXT: usize = 0x5A8;

    // Hypervisor and Virtual Supervisor CSRs
    pub const VSSTATUS: usize = 0x200;
    pub const VSIE: usize = 0x204;
    pub const VSTVEC: usize = 0x205;
    pub const VSSCRATCH: usize = 0x240;
    pub const VSEPC: usize = 0x241;
    pub const VSCAUSE: usize = 0x242;
    pub const VSTVAL: usize = 0x243;
    pub const VSIP: usize = 0x244;
    pub const VSATP: usize = 0x280;
    pub const HSTATUS: usize = 0x600;
    pub const HEDELEG: usize = 0x602;
    pub const HIDELEG: usize = 0x603;
    pub const HIE: usize = 0x604;
    pub const HTIMEDELTA: usize = 0x605;
    pub const HCOUNTEREN: usize = 0x606;
    pub const HGEIE: usize = 0x607;
    pub const HGEIP: usize = 0xE12;
    pub const HENVCFG: usize = 0x60A;
    pub const HTVAL: usize = 0x643;
    pub const HIP: usize = 0x644;
    pub const HVIP: usize = 0x645;
    pub const HTINST: usize = 0x64A;
    pub const HGATP: usize = 0x680;

    // Vector extension CSRs
    pub const VSTART: usize = 0x8;
    pub const VXSAT: usize = 0x9;
    pub const VXRM: usize = 0xA;
    pub const VCSR: usize = 0xF;

    // Crypto extension CSRs
    pub const SEED: usize = 0x15;
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
