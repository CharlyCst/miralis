//! RISC-V instruction decoder

use crate::arch::{Csr, Register};

const OPCODE_MASK: usize = 0b1111111 << 0;

/// A RISC-V instruction.
#[derive(Debug)]
pub enum Instr {
    Ecall,
    Ebreak,
    Wfi,
    Csrrw(Csr, Register),
    Csrrs(Csr, Register),
    Csrrc(Csr),
    Csrrwi(Csr),
    Csrrsi(Csr),
    Csrrci(Csr),
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

    match func3 {
        0b001 => Instr::Csrrw(csr, Register::try_from(rs1).unwrap()),
        0b010 => Instr::Csrrs(csr, Register::try_from(rd).unwrap()),
        0b011 => Instr::Csrrc(csr),
        0b101 => Instr::Csrrwi(csr),
        0b110 => Instr::Csrrsi(csr),
        0b111 => Instr::Csrrci(csr),
        _ => unreachable!(),
    }
}

fn decode_csr(csr: usize) -> Csr {
    match csr {
        0x300 => Csr::Mstatus,
        0x340 => Csr::Mscratch,
        _ => Csr::Unknown,
    }
}
