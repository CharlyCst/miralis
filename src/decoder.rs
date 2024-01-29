//! RISC-V instruction decoder

use crate::arch::{Csr, Register};

const OPCODE_MASK: usize = 0b1111111 << 0;

/// A RISC-V instruction.
#[derive(Debug)]
pub enum Instr {
    Ecall,
    Ebreak,
    Wfi,
    /// CSR Read/Write
    Csrrw {
        csr: Csr,
        rd: Register,
        rs1: Register,
    },
    /// CSR Read & Set
    Csrrs {
        csr: Csr,
        rd: Register,
        rs1: Register,
    },
    /// CSR Read & Clear
    Csrrc {
        csr: Csr,
        rd: Register,
        rs1: Register,
    },
    /// CSR Read/Write Immediate
    Csrrwi {
        csr: Csr,
        rd: Register,
        uimm: usize,
    },
    /// CSR Read & Set Immediate
    Csrrsi {
        csr: Csr,
        rd: Register,
        uimm: usize,
    },
    /// CSR Read & Clear Immediate
    Csrrci {
        csr: Csr,
        rd: Register,
        uimm: usize,
    },
    Unknown,
}

/// A RISC-V opcode.
#[derive(Debug)]
enum Opcode {
    Load,
    System,
    Unknown,
}

/// Decode a raw RISC-V instruction.
///
/// NOTE: for now this function  only support 32 bits instructions.
pub fn decode(raw: usize) -> Instr {
    let opcode = decode_opcode(raw);
    match opcode {
        Opcode::System => decode_system(raw),
        _ => Instr::Unknown,
    }
}

fn decode_opcode(raw: usize) -> Opcode {
    let opcode = raw & OPCODE_MASK;

    if opcode & 0b11 != 0b11 {
        // It seems all 32 bits instructions start with  0b11.
        return Opcode::Unknown;
    }

    match opcode >> 2 {
        0b00000 => Opcode::Load,
        0b11100 => Opcode::System,
        _ => Opcode::Unknown,
    }
}

fn decode_system(raw: usize) -> Instr {
    let rd = (raw >> 7) & 0b11111;
    let func3 = (raw >> 12) & 0b111;
    let rs1 = (raw >> 15) & 0b11111;
    let imm = (raw >> 20) & 0b111111111111;

    if func3 == 0b000 {
        return match imm {
            0b000000000000 => Instr::Ecall,
            0b000000000001 => Instr::Ebreak,
            0b000100000101 => Instr::Wfi,
            _ => Instr::Unknown,
        };
    }

    let csr = decode_csr(imm);
    let rd = Register::from(rd);
    match func3 {
        0b001 => Instr::Csrrw {
            csr,
            rd,
            rs1: Register::from(rs1),
        },
        0b010 => Instr::Csrrs {
            csr,
            rd,
            rs1: Register::from(rs1),
        },
        0b011 => Instr::Csrrc {
            csr,
            rd,
            rs1: Register::from(rs1),
        },
        0b101 => Instr::Csrrwi { csr, rd, uimm: rs1 },
        0b110 => Instr::Csrrsi { csr, rd, uimm: rs1 },
        0b111 => Instr::Csrrci { csr, rd, uimm: rs1 },
        _ => Instr::Unknown,
    }
}

fn decode_csr(csr: usize) -> Csr {
    match csr {
        0x300 => Csr::Mstatus,
        0x304 => Csr::Mie,
        0x305 => Csr::Mtvec,
        0x340 => Csr::Mscratch,
        _ => {
            log::info!("Unknown CSR: 0x{:x}", csr);
            Csr::Unknown
        }
    }
}
