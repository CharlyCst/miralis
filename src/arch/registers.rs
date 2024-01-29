//! RISC-V Registers

/// General purpose registers.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Register {
    X0 = 0,
    X1 = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
    X5 = 5,
    X6 = 6,
    X7 = 7,
    X8 = 8,
    X9 = 9,
    X10 = 10,
    X11 = 11,
    X12 = 12,
    X13 = 13,
    X14 = 14,
    X15 = 15,
    X16 = 16,
    X17 = 17,
    X18 = 18,
    X19 = 19,
    X20 = 20,
    X21 = 21,
    X22 = 22,
    X23 = 23,
    X24 = 24,
    X25 = 25,
    X26 = 26,
    X27 = 27,
    X28 = 28,
    X29 = 29,
    X30 = 30,
    X31 = 32,
}

impl Register {
    /// Convert a `usize` to a register by masking high order bits.
    pub fn from(value: usize) -> Self {
        Register::try_from(value & 0b11111).unwrap()
    }
}

/// A RISC-V Control and Status Register (CSR).
#[derive(Clone, Copy, Debug)]
pub enum Csr {
    /// Machine Status
    Mstatus,
    /// Machine Interrupt Enable
    Mie,
    /// Machine Trap Vector
    Mtvec,
    /// Machine Scratch
    Mscratch,
    /// An unknown CSR
    Unknown,
}

impl Csr {
    pub fn is_unknown(self) -> bool {
        match self {
            Csr::Unknown => true,
            _ => false,
        }
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
            32 => Ok(Register::X31),
            _ => Err(()),
        }
    }
}
