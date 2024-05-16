//! RISC-V instruction decoder
use crate::arch::{Csr, Register};
use crate::platform::{Plat, Platform};

const OPCODE_MASK: usize = 0b1111111 << 0;

/// A RISC-V instruction.
#[derive(Debug, PartialEq, Eq)]
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
    Mret,
    Vfencevma,
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
            0b001100000010 => Instr::Mret,
            _ if (imm & 0b111111111111) == 0b000100100000 => Instr::Vfencevma,
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
        0x301 => Csr::Misa,
        0x304 => Csr::Mie,
        0x305 => Csr::Mtvec,
        0x340 => Csr::Mscratch,
        0x344 => Csr::Mip,
        0xF14 => Csr::Mhartid,
        0xF11 => Csr::Mvendorid,
        0xF12 => Csr::Marchid,
        0xF13 => Csr::Mimpid,
        0x3A0..=0x3AF => Csr::Pmpcfg(csr - 0x3A0),
        0x3B0..=0x3EF => Csr::Pmpaddr(csr - 0x3B0),
        0xB00 => Csr::Mcycle,
        0xB02 => Csr::Minstret,
        0xB03..=0xB1F => Csr::Mhpmcounter(csr - 0xB03), // Mhpm counters start at 3 and end at 31 : we shift them by 3 to start at 0 and end at 29
        0x320 => Csr::Mcountinhibit,
        0x323..=0x33F => Csr::Mhpmevent(csr - 0x323),
        0x306 => Csr::Mcounteren,
        0x30a => Csr::Menvcgf,
        0x747 => Csr::Mseccfg,
        0xF15 => Csr::Mconfigptr,
        0x302 => {
            if !Plat::HAS_S_MODE {
                log::info!(
                    "Unknown CSR: 0x{:x}, Medeleg should not exist in a system without S-mode",
                    csr
                );
                Csr::Unknown
            } else {
                Csr::Medeleg
            }
        }
        0x303 => {
            if !Plat::HAS_S_MODE {
                log::info!(
                    "Unknown CSR: 0x{:x}, Mideleg should not exist in a system without S-mode",
                    csr
                );
                Csr::Unknown
            } else {
                Csr::Mideleg
            }
        }
        0x34A => {
            log::info!(
                "Unknown CSR: 0x{:x}, Mtisnt should not exist in a system without without hypervisor extension",
                csr
            );
            // TODO: add support for platform misa
            if true {
                Csr::Unknown
            } else {
                Csr::Mtinst
            }
        }
        0x34B => {
            log::info!(
                "Unknown CSR: 0x{:x}, Mtval2 should not exist in a system without hypervisor extension",
                csr
            );
            // TODO: add support for platform misa
            if true {
                Csr::Unknown
            } else {
                Csr::Mtval2
            }
        }
        0x7A0 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Tselect
            }
        }
        0x7A1 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Tdata1
            }
        }
        0x7A2 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Tdata2
            }
        }
        0x7A3 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Tdata3
            }
        }
        0x7A8 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Mcontext
            }
        }
        0x7B0 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Dcsr
            }
        }
        0x7B1 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Dpc
            }
        }
        0x7B2 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Dscratch0
            }
        }
        0x7B3 => {
            if true {
                Csr::Unknown
            } else {
                Csr::Dscratch1
            }
        }
        0x342 => Csr::Mcause,
        0x341 => Csr::Mepc,
        0x343 => Csr::Mtval,
        // Supervisor-level CSRs
        0x100 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Sstatus
            }
        }
        0x104 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Sie
            }
        }
        0x105 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Stvec
            }
        }
        0x106 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Scounteren
            }
        }
        0x10A => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Senvcfg
            }
        }
        0x140 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Sscratch
            }
        }
        0x141 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Sepc
            }
        }
        0x142 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Scause
            }
        }
        0x143 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Stval
            }
        }
        0x144 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Sip
            }
        }
        0x180 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Satp
            }
        }
        0x5A8 => {
            if !Plat::HAS_S_MODE {
                Csr::Unknown
            } else {
                Csr::Scontext
            }
        }
        _ => {
            log::info!("Unknown CSR: 0x{:x}", csr);
            Csr::Unknown
        }
    }
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use super::*;

    /// Decodes a few basic instructions, just to check the bare minimum functionalities.
    ///
    /// Here is an handy tool to double check:
    /// https://luplab.gitlab.io/rvcodecjs/
    #[test]
    fn simple_decode() {
        assert_eq!(decode(0x10500073), Instr::Wfi);
        assert_eq!(decode(0x00000073), Instr::Ecall);
        assert_eq!(decode(0x30200073), Instr::Mret);
        assert_eq!(
            decode(0x34071473),
            Instr::Csrrw {
                csr: Csr::Mscratch,
                rd: Register::X8,
                rs1: Register::X14
            }
        );
    }
}
