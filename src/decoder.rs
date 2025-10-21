//! RISC-V instruction decoder
use crate::arch::{csr, Csr, Register, Width};
use crate::host::MiralisContext;
use crate::logger;
use crate::platform::{Plat, Platform};
use crate::utils::bits_to_int;

const ILLEGAL_OPCODE_MASK: usize = 0b1110011;
const SFENCE_INSTR_VMA_MASK: usize = 0b0001001 << 25;
const HFENCE_INSTR_VVMA_MASK: usize = 0b0010001 << 25;
const HFENCE_INSTR_GVMA_MASK: usize = 0b0110001 << 25;

const RS1_RS1_INSTR_TYPE_MASK: usize = 0b1111111111000000001111111;
const FUNC3_MASK: usize = 0b111000000000000;

/// Compressed Load Word opcode
const C_LW: usize = 0b010;
/// Compressed Load Double word opcode
const C_LD: usize = 0b011;

/// Compressed Store word opcode
const C_SW: usize = 0b110;
/// Compressed Load Double word opcode
const C_SD: usize = 0b111;

/// A RISC-V privileged instruction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IllegalInst {
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
    Sret,
    /// Fence instructions
    Sfencevma {
        rs1: Register,
        rs2: Register,
    },
    Hfencevvma {
        rs1: Register,
        rs2: Register,
    },
    Hfencegvma {
        rs1: Register,
        rs2: Register,
    },
    Unknown,
}

/// A load (register-based) instruction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LoadInstr {
    pub rd: Register,
    pub rs1: Register,
    pub imm: isize,
    pub len: Width,
    pub is_compressed: bool,
    pub is_unsigned: bool,
}

/// A store (register-based) instruction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StoreInstr {
    pub rs2: Register,
    pub rs1: Register,
    pub imm: isize,
    pub len: Width,
    pub is_compressed: bool,
}

impl MiralisContext {
    /// Decodes a raw read RISC-V instruction.
    pub fn decode_load(&self, raw: usize) -> LoadInstr {
        match extract_last_two_bits(raw) {
            0b11 => self.decode_uncompressed_load(raw),
            // Register-based load and store instructions for C set start with 0b00
            0b00 => self.decode_register_based_compressed_load(raw),
            // Stack-based load and store instructions for C set start with 0b10
            0b10 => todo!("Decode stack based C load"),
            _ => unreachable!(),
        }
    }

    /// Decodes a raw write RISC-V instruction.
    pub fn decode_store(&self, raw: usize) -> StoreInstr {
        match extract_last_two_bits(raw) {
            0b11 => self.decode_uncompressed_store(raw),
            // Register-based load and store instructions for C set start with 0b00
            0b00 => self.decode_register_based_compressed_store(raw),
            // Stack-based load and store instructions for C set start with 0b10
            0b10 => todo!("Decode stack based C store"),
            _ => unreachable!(),
        }
    }

    /// Decodes a raw illegal instruction
    pub fn decode_illegal_instruction(&self, raw_instr: usize) -> IllegalInst {
        assert_eq!(
            raw_instr & 0b1111111,
            ILLEGAL_OPCODE_MASK,
            "Precondition violated, this is not an illegal instruction: 0x{:x}",
            raw_instr
        );

        match raw_instr {
            0b00010000010100000000000001110011 => return IllegalInst::Wfi,
            0b00110000001000000000000001110011 => return IllegalInst::Mret,
            0b00010000001000000000000001110011 => return IllegalInst::Sret,
            _ => {}
        }

        let uimm = (raw_instr >> 15) & 0b11111;
        let rs1 = Register::from(uimm);
        let rs2 = Register::from((raw_instr >> 20) & 0b11111);

        match raw_instr & !RS1_RS1_INSTR_TYPE_MASK {
            SFENCE_INSTR_VMA_MASK => return IllegalInst::Sfencevma { rs1, rs2 },
            HFENCE_INSTR_VVMA_MASK => return IllegalInst::Hfencevvma { rs1, rs2 },
            HFENCE_INSTR_GVMA_MASK => return IllegalInst::Hfencegvma { rs1, rs2 },
            _ => {}
        }

        let csr = self.decode_csr((raw_instr >> 20) & 0b111111111111);
        let rd = Register::from((raw_instr >> 7) & 0b11111);

        match raw_instr & FUNC3_MASK {
            0x1000 => IllegalInst::Csrrw { csr, rd, rs1 },
            0x2000 => IllegalInst::Csrrs { csr, rd, rs1 },
            0x3000 => IllegalInst::Csrrc { csr, rd, rs1 },
            0x5000 => IllegalInst::Csrrwi { csr, rd, uimm },
            0x6000 => IllegalInst::Csrrsi { csr, rd, uimm },
            0x7000 => IllegalInst::Csrrci { csr, rd, uimm },
            _ => IllegalInst::Unknown,
        }
    }

    fn decode_register_based_compressed_load(&self, raw: usize) -> LoadInstr {
        let rd = (raw >> 2) & 0b111;
        let rs1 = (raw >> 7) & 0b111;

        let rd = Register::from(rd + 8);
        let rs1 = Register::from(rs1 + 8);

        let func3 = (raw >> 13) & 0b111;
        match func3 {
            C_LW => {
                let imm_2 = ((raw >> 6) & 0b1) << 2;
                let imm_5_3 = ((raw >> 10) & 0b111) << 3;
                let imm_6 = ((raw >> 5) & 0b1) << 6;
                LoadInstr {
                    rd,
                    rs1,
                    imm: (imm_6 | imm_5_3 | imm_2) as isize,
                    len: Width::from(32),
                    is_compressed: true,
                    is_unsigned: false,
                }
            }
            C_LD => {
                let imm = (raw >> 7) & 0b111000 | ((raw << 1) & 0b11000000);
                LoadInstr {
                    rd,
                    rs1,
                    imm: imm as isize,
                    len: Width::from(64),
                    is_compressed: true,
                    is_unsigned: false,
                }
            }
            _ => unreachable!(),
        }
    }

    fn decode_register_based_compressed_store(&self, raw: usize) -> StoreInstr {
        let func3 = (raw >> 13) & 0b111;
        let rs2 = (raw >> 2) & 0b111;
        let rs1 = (raw >> 7) & 0b111;

        let rs2 = Register::from(rs2 + 8);
        let rs1 = Register::from(rs1 + 8);

        match func3 {
            C_SW => {
                let imm_2 = ((raw >> 6) & 0b1) << 2;
                let imm_5_3 = ((raw >> 10) & 0b111) << 3;
                let imm_6 = ((raw >> 5) & 0b1) << 6;
                StoreInstr {
                    rs2,
                    rs1,
                    imm: (imm_6 | imm_5_3 | imm_2) as isize,
                    len: Width::from(32),
                    is_compressed: true,
                }
            }
            C_SD => {
                let imm = (raw >> 7) & 0b111000 | ((raw << 1) & 0b11000000);
                StoreInstr {
                    rs2,
                    rs1,
                    imm: imm as isize,
                    len: Width::from(64),
                    is_compressed: true,
                }
            }
            _ => unreachable!(),
        }
    }

    fn decode_uncompressed_load(&self, raw: usize) -> LoadInstr {
        let func3 = (raw >> 12) & 0b111;
        let rd = (raw >> 7) & 0b11111;
        let rs1 = (raw >> 15) & 0b11111;
        let imm = bits_to_int(raw, 20, 31);

        let rs1 = Register::from(rs1);
        let rd = Register::from(rd);

        match func3 {
            0b000 => LoadInstr {
                rd,
                rs1,
                imm,
                len: Width::from(8),
                is_compressed: false,
                is_unsigned: false,
            },
            0b001 => LoadInstr {
                rd,
                rs1,
                imm,
                len: Width::from(16),
                is_compressed: false,
                is_unsigned: false,
            },
            0b010 => LoadInstr {
                rd,
                rs1,
                imm,
                len: Width::from(32),
                is_compressed: false,
                is_unsigned: false,
            },
            0b011 => LoadInstr {
                rd,
                rs1,
                imm,
                len: Width::from(64),
                is_compressed: false,
                is_unsigned: false,
            },
            0b100 => LoadInstr {
                rd,
                rs1,
                imm,
                len: Width::from(8),
                is_compressed: false,
                is_unsigned: true,
            },
            0b101 => LoadInstr {
                rd,
                rs1,
                imm,
                len: Width::from(16),
                is_compressed: false,
                is_unsigned: true,
            },
            0b110 => LoadInstr {
                rd,
                rs1,
                imm,
                len: Width::from(32),
                is_compressed: false,
                is_unsigned: true,
            },
            0b111 => LoadInstr {
                rd,
                rs1,
                imm,
                len: Width::from(64),
                is_compressed: false,
                is_unsigned: true,
            },
            _ => unreachable!(),
        }
    }

    fn decode_uncompressed_store(&self, raw: usize) -> StoreInstr {
        let func3 = (raw >> 12) & 0b111;
        let rs1: usize = (raw >> 15) & 0b11111;
        let rs2 = (raw >> 20) & 0b11111;
        let imm = bits_to_int(
            ((raw >> 7) & 0b11111) | ((raw >> 20) & 0b111111100000),
            0,
            11,
        );

        let rs1 = Register::from(rs1);
        let rs2 = Register::from(rs2);

        match func3 {
            0b000 => StoreInstr {
                rs2,
                rs1,
                imm,
                len: Width::from(8),
                is_compressed: false,
            },
            0b001 => StoreInstr {
                rs2,
                rs1,
                imm,
                len: Width::from(16),
                is_compressed: false,
            },
            0b010 => StoreInstr {
                rs2,
                rs1,
                imm,
                len: Width::from(32),
                is_compressed: false,
            },
            0b011 => StoreInstr {
                rs2,
                rs1,
                imm,
                len: Width::from(64),
                is_compressed: false,
            },
            _ => unreachable!(),
        }
    }

    pub fn decode_csr(&self, csr: usize) -> Csr {
        match csr {
            csr::MSTATUS => Csr::Mstatus,
            csr::MISA => Csr::Misa,
            csr::MIE => Csr::Mie,
            csr::MTVEC => Csr::Mtvec,
            csr::MSCRATCH => Csr::Mscratch,
            csr::MIP => Csr::Mip,
            csr::MHARTID => Csr::Mhartid,
            csr::MVENDORID => Csr::Mvendorid,
            csr::MARCHID => Csr::Marchid,
            csr::MIMPID => Csr::Mimpid,
            csr::PMPCFG0..=csr::PMPCFG15 => {
                let id = csr - csr::PMPCFG0;
                if id.is_multiple_of(2) {
                    Csr::Pmpcfg(id)
                } else {
                    Csr::Unknown // Invalid on rv64
                }
            }
            csr::PMPADDR0..=csr::PMPADDR63 => Csr::Pmpaddr(csr - csr::PMPADDR0),
            csr::MCYCLE => {
                if self.hw.extensions.has_zicntr {
                    Csr::Mcycle
                } else {
                    Csr::Unknown
                }
            }
            csr::MINSTRET => {
                if self.hw.extensions.has_zicntr {
                    Csr::Minstret
                } else {
                    Csr::Unknown
                }
            }
            csr::CYCLE => {
                if self.hw.extensions.has_zicntr {
                    Csr::Cycle
                } else {
                    Csr::Unknown
                }
            }
            csr::TIME => {
                if self.hw.extensions.has_zicntr {
                    Csr::Time
                } else {
                    Csr::Unknown
                }
            }
            csr::INSTRET => {
                if self.hw.extensions.has_zicntr {
                    Csr::Instret
                } else {
                    Csr::Unknown
                }
            }
            csr::MHPMCOUNTER3..=csr::MHPMCOUNTER31 => {
                // Mhpm counters start at 3 and end at 31 : we shift them by 3 to start at 0 and end at 29
                if self.hw.extensions.has_zihpm_extension {
                    Csr::Mhpmcounter(csr - csr::MHPMCOUNTER3)
                } else {
                    Csr::Unknown
                }
            }
            csr::MCOUNTINHIBIT => Csr::Mcountinhibit,
            csr::MHPMEVENT3..=csr::MHPMEVENT31 => {
                if self.hw.extensions.has_zihpm_extension {
                    Csr::Mhpmevent(csr - csr::MHPMEVENT3)
                } else {
                    Csr::Unknown
                }
            }
            csr::MCOUNTEREN => Csr::Mcounteren,
            csr::MENVCFG => Csr::Menvcfg,
            csr::MSECCFG => {
                if !self.hw.extensions.has_tee_extension {
                    Csr::Unknown
                } else {
                    Csr::Mseccfg
                }
            }
            csr::MCONFIGPTR => Csr::Mconfigptr,
            csr::MEDELEG => {
                if !self.hw.extensions.has_s_extension {
                    log::warn!(
                        "Unknown CSR: 0x{:x}, Medeleg should not exist in a system without S-mode",
                        csr
                    );
                    Csr::Unknown
                } else {
                    Csr::Medeleg
                }
            }
            csr::MIDELEG => {
                if !self.hw.extensions.has_s_extension {
                    log::warn!(
                        "Unknown CSR: 0x{:x}, Mideleg should not exist in a system without S-mode",
                        csr
                    );
                    Csr::Unknown
                } else {
                    Csr::Mideleg
                }
            }
            csr::MTINST => {
                if !self.hw.extensions.has_h_extension {
                    log::warn!(
                    "Unknown CSR: 0x{:x}, Mtisnt should not exist in a system without without hypervisor extension",
                    csr
                );
                    Csr::Unknown
                } else {
                    Csr::Mtinst
                }
            }
            csr::MTVAL2 => {
                if !self.hw.extensions.has_h_extension {
                    log::warn!(
                    "Unknown CSR: 0x{:x}, Mtval2 should not exist in a system without hypervisor extension",
                    csr
                );
                    Csr::Unknown
                } else {
                    Csr::Mtval2
                }
            }
            csr::TSELECT => Csr::Tselect,
            csr::TDATA1 => {
                if true {
                    Csr::Unknown
                } else {
                    Csr::Tdata1
                }
            }
            csr::TDATA2 => {
                if true {
                    Csr::Unknown
                } else {
                    Csr::Tdata2
                }
            }
            csr::TDATA3 => {
                if true {
                    Csr::Unknown
                } else {
                    Csr::Tdata3
                }
            }
            csr::MCONTEXT => {
                if true {
                    Csr::Unknown
                } else {
                    Csr::Mcontext
                }
            }
            csr::DCSR => {
                if true {
                    Csr::Unknown
                } else {
                    Csr::Dcsr
                }
            }
            csr::DPC => {
                if true {
                    Csr::Unknown
                } else {
                    Csr::Dpc
                }
            }
            csr::DSCRATCH0 => {
                if true {
                    Csr::Unknown
                } else {
                    Csr::Dscratch0
                }
            }
            csr::DSCRATCH1 => {
                if true {
                    Csr::Unknown
                } else {
                    Csr::Dscratch1
                }
            }
            0x7C0..=0x7FF => {
                // Custom CSRs, do per-SoC validation
                if Plat::is_valid_custom_csr(csr) {
                    Csr::Custom(csr)
                } else {
                    Csr::Unknown
                }
            }
            csr::MCAUSE => Csr::Mcause,
            csr::MEPC => Csr::Mepc,
            csr::MTVAL => Csr::Mtval,
            // Supervisor-level CSRs
            csr::SSTATUS => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sstatus
                }
            }
            csr::SIE => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sie
                }
            }
            csr::STVEC => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Stvec
                }
            }
            csr::SCOUNTEREN => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Scounteren
                }
            }
            csr::SENVCFG => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Senvcfg
                }
            }
            csr::SSCRATCH => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sscratch
                }
            }
            csr::SEPC => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sepc
                }
            }
            csr::SCAUSE => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Scause
                }
            }
            csr::STVAL => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Stval
                }
            }
            csr::SIP => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sip
                }
            }
            csr::STIMECMP => {
                if !self.hw.extensions.is_sstc_enabled {
                    Csr::Unknown
                } else {
                    Csr::Stimecmp
                }
            }
            csr::SATP => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Satp
                }
            }
            csr::SCONTEXT => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Scontext
                }
            }

            // Hypervisor and Virtual Supervisor CSRs
            csr::HSTATUS => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hstatus
                }
            }
            csr::HEDELEG => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hedeleg
                }
            }
            csr::HIDELEG => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hideleg
                }
            }
            csr::HVIP => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hvip
                }
            }
            csr::HIP => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hip
                }
            }
            csr::HIE => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hie
                }
            }
            csr::HGEIP => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hgeip
                }
            }
            csr::HGEIE => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hgeie
                }
            }
            csr::HENVCFG => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Henvcfg
                }
            }
            csr::HCOUNTEREN => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hcounteren
                }
            }
            csr::HTIMEDELTA => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Htimedelta
                }
            }
            csr::HTVAL => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Htval
                }
            }
            csr::HTINST => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Htinst
                }
            }
            csr::HGATP => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hgatp
                }
            }
            csr::VSSTATUS => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsstatus
                }
            }
            csr::VSIE => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsie
                }
            }
            csr::VSTVEC => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vstvec
                }
            }
            csr::VSSCRATCH => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsscratch
                }
            }
            csr::VSEPC => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsepc
                }
            }
            csr::VSCAUSE => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vscause
                }
            }
            csr::VSTVAL => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vstval
                }
            }
            csr::VSIP => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsip
                }
            }
            csr::VSATP => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsatp
                }
            }

            // Vector extension
            csr::VSTART => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vstart
                }
            }
            csr::VXSAT => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vxsat
                }
            }
            csr::VXRM => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vxrm
                }
            }
            csr::VCSR => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vcsr
                }
            }
            csr::VL => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vl
                }
            }
            csr::VTYPE => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vtype
                }
            }
            csr::VLENB => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vlenb
                }
            }
            csr::SEED => {
                // Crypto extension
                if !self.hw.extensions.has_crypto_extension {
                    Csr::Unknown
                } else {
                    Csr::Seed
                }
            }

            _ => {
                logger::debug!("Unknown CSR: 0x{:x}", csr);
                Csr::Unknown
            }
        }
    }
}

fn extract_last_two_bits(value: usize) -> usize {
    value & 0b11
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::{Arch, Architecture};

    /// Decodes privileged instructions
    /// Here is a handy tool to double check:
    /// https://luplab.gitlab.io/rvcodecjs/
    #[test]
    fn system_instructions() {
        let mctx = MiralisContext::new(unsafe { Arch::detect_hardware() }, 0x100000, 0x2000);
        // MRET: Return from machine mode.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30200073),
            IllegalInst::Mret
        );
        // SRET: Return from supervisor mode.
        assert_eq!(
            mctx.decode_illegal_instruction(0x10200073),
            IllegalInst::Sret
        );
        // WFI: Wait for interrupt.
        assert_eq!(
            mctx.decode_illegal_instruction(0x10500073),
            IllegalInst::Wfi
        );
        // SFENCE.VMA: Supervisor memory-management fence.
        assert_eq!(
            mctx.decode_illegal_instruction(0x12000073),
            IllegalInst::Sfencevma {
                rs1: Register::X0,
                rs2: Register::X0
            }
        );
        assert_eq!(
            mctx.decode_illegal_instruction(0x13300073),
            IllegalInst::Sfencevma {
                rs1: Register::X0,
                rs2: Register::X19
            }
        );
    }

    #[test]
    fn csr_instructions() {
        let mctx = MiralisContext::new(unsafe { Arch::detect_hardware() }, 0x100000, 0x2000);

        // CSRRW: Atomic Read/Write CSR.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30001073),
            IllegalInst::Csrrw {
                csr: Csr::Mstatus,
                rd: Register::X0,
                rs1: Register::X0,
            }
        );

        // CSRRS: Atomic Read and Set Bits in CSR.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30002073),
            IllegalInst::Csrrs {
                csr: Csr::Mstatus,
                rd: Register::X0,
                rs1: Register::X0,
            }
        );

        // CSRRC: Atomic Read and Clear Bits in CSR.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30003073),
            IllegalInst::Csrrc {
                csr: Csr::Mstatus,
                rd: Register::X0,
                rs1: Register::X0,
            }
        );

        // CSRRWI: Atomic Read/Write CSR Immediate.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30005073),
            IllegalInst::Csrrwi {
                csr: Csr::Mstatus,
                rd: Register::X0,
                uimm: 0x0,
            }
        );

        // CSRRSI: Atomic Read and Set Bits in CSR Immediate.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30006073),
            IllegalInst::Csrrsi {
                csr: Csr::Mstatus,
                rd: Register::X0,
                uimm: 0x0,
            }
        );

        // CSRRCI: Atomic Read and Clear Bits in CSR Immediate.s
        assert_eq!(
            mctx.decode_illegal_instruction(0x30007073),
            IllegalInst::Csrrci {
                csr: Csr::Mstatus,
                rd: Register::X0,
                uimm: 0x0,
            }
        );
    }

    #[test]
    fn access_instructions() {
        let mctx = MiralisContext::new(unsafe { Arch::detect_hardware() }, 0x10000, 0x2000);

        assert_eq!(
            mctx.decode_load(0xff87b703),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(64),
                is_compressed: false,
                is_unsigned: false,
            }
        );

        assert_eq!(
            mctx.decode_store(0xfee7bc23),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(64),
                is_compressed: false,
            }
        );

        assert_eq!(
            mctx.decode_load(0xff87a703),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(32),
                is_compressed: false,
                is_unsigned: false,
            }
        );

        assert_eq!(
            mctx.decode_store(0xfee7ac23),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(32),
                is_compressed: false,
            }
        );

        assert_eq!(
            mctx.decode_load(0xff879703),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(16),
                is_compressed: false,
                is_unsigned: false,
            }
        );

        assert_eq!(
            mctx.decode_store(0xfee79c23),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(16),
                is_compressed: false,
            }
        );

        assert_eq!(
            mctx.decode_load(0xff878703),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(8),
                is_compressed: false,
                is_unsigned: false,
            }
        );

        assert_eq!(
            mctx.decode_store(0xfee78c23),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(8),
                is_compressed: false,
            }
        );

        assert_eq!(
            mctx.decode_store(0xffffe798),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: 8,
                len: Width::from(64),
                is_compressed: true,
            }
        );

        assert_eq!(
            mctx.decode_load(0xffff6798),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: 8,
                len: Width::from(64),
                is_compressed: true,
                is_unsigned: false,
            }
        );

        assert_eq!(
            mctx.decode_load(0xffff4798),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: 8,
                len: Width::from(32),
                is_compressed: true,
                is_unsigned: false,
            }
        );

        assert_eq!(
            mctx.decode_store(0xffffc798),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: 8,
                len: Width::from(32),
                is_compressed: true,
            }
        );

        assert_eq!(
            mctx.decode_load(0xff87e703),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(32),
                is_compressed: false,
                is_unsigned: true,
            }
        );

        assert_eq!(
            mctx.decode_load(0xff87d703),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(16),
                is_compressed: false,
                is_unsigned: true,
            }
        );

        assert_eq!(
            mctx.decode_load(0xff87c703),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(8),
                is_compressed: false,
                is_unsigned: true,
            }
        );
    }

    #[test]
    fn decode_rd() {
        let mctx = MiralisContext::new(unsafe { Arch::detect_hardware() }, 0x10000, 0x2000);

        let base_instruction: usize = 0x30001073;

        let instruction_builder_rd = |offset: usize| -> usize { base_instruction + (offset << 7) };

        let registers_to_test: [(usize, Register); 32] = [
            (instruction_builder_rd(0), Register::X0),
            (instruction_builder_rd(1), Register::X1),
            (instruction_builder_rd(2), Register::X2),
            (instruction_builder_rd(3), Register::X3),
            (instruction_builder_rd(4), Register::X4),
            (instruction_builder_rd(5), Register::X5),
            (instruction_builder_rd(6), Register::X6),
            (instruction_builder_rd(7), Register::X7),
            (instruction_builder_rd(8), Register::X8),
            (instruction_builder_rd(9), Register::X9),
            (instruction_builder_rd(10), Register::X10),
            (instruction_builder_rd(11), Register::X11),
            (instruction_builder_rd(12), Register::X12),
            (instruction_builder_rd(13), Register::X13),
            (instruction_builder_rd(14), Register::X14),
            (instruction_builder_rd(15), Register::X15),
            (instruction_builder_rd(16), Register::X16),
            (instruction_builder_rd(17), Register::X17),
            (instruction_builder_rd(18), Register::X18),
            (instruction_builder_rd(19), Register::X19),
            (instruction_builder_rd(20), Register::X20),
            (instruction_builder_rd(21), Register::X21),
            (instruction_builder_rd(22), Register::X22),
            (instruction_builder_rd(23), Register::X23),
            (instruction_builder_rd(24), Register::X24),
            (instruction_builder_rd(25), Register::X25),
            (instruction_builder_rd(26), Register::X26),
            (instruction_builder_rd(27), Register::X27),
            (instruction_builder_rd(28), Register::X28),
            (instruction_builder_rd(29), Register::X29),
            (instruction_builder_rd(30), Register::X30),
            (instruction_builder_rd(31), Register::X31),
        ];

        for tuple in registers_to_test.iter() {
            assert_eq!(
                mctx.decode_illegal_instruction(tuple.0),
                IllegalInst::Csrrw {
                    csr: Csr::Mstatus,
                    rd: tuple.1,
                    rs1: Register::X0,
                }
            );
        }
    }

    #[test]
    fn decode_rs1() {
        let mctx = MiralisContext::new(unsafe { Arch::detect_hardware() }, 0x10000, 0x2000);

        let base_instruction: usize = 0x30001073;

        let instruction_builder_rs1 =
            |offset: usize| -> usize { base_instruction + (offset << 15) };

        let registers_to_test: [(usize, Register); 32] = [
            (instruction_builder_rs1(0), Register::X0),
            (instruction_builder_rs1(1), Register::X1),
            (instruction_builder_rs1(2), Register::X2),
            (instruction_builder_rs1(3), Register::X3),
            (instruction_builder_rs1(4), Register::X4),
            (instruction_builder_rs1(5), Register::X5),
            (instruction_builder_rs1(6), Register::X6),
            (instruction_builder_rs1(7), Register::X7),
            (instruction_builder_rs1(8), Register::X8),
            (instruction_builder_rs1(9), Register::X9),
            (instruction_builder_rs1(10), Register::X10),
            (instruction_builder_rs1(11), Register::X11),
            (instruction_builder_rs1(12), Register::X12),
            (instruction_builder_rs1(13), Register::X13),
            (instruction_builder_rs1(14), Register::X14),
            (instruction_builder_rs1(15), Register::X15),
            (instruction_builder_rs1(16), Register::X16),
            (instruction_builder_rs1(17), Register::X17),
            (instruction_builder_rs1(18), Register::X18),
            (instruction_builder_rs1(19), Register::X19),
            (instruction_builder_rs1(20), Register::X20),
            (instruction_builder_rs1(21), Register::X21),
            (instruction_builder_rs1(22), Register::X22),
            (instruction_builder_rs1(23), Register::X23),
            (instruction_builder_rs1(24), Register::X24),
            (instruction_builder_rs1(25), Register::X25),
            (instruction_builder_rs1(26), Register::X26),
            (instruction_builder_rs1(27), Register::X27),
            (instruction_builder_rs1(28), Register::X28),
            (instruction_builder_rs1(29), Register::X29),
            (instruction_builder_rs1(30), Register::X30),
            (instruction_builder_rs1(31), Register::X31),
        ];

        for tuple in registers_to_test.iter() {
            assert_eq!(
                mctx.decode_illegal_instruction(tuple.0),
                IllegalInst::Csrrw {
                    csr: Csr::Mstatus,
                    rd: Register::X0,
                    rs1: tuple.1,
                }
            );
        }
    }
}
