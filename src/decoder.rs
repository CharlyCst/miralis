//! RISC-V instruction decoder
use crate::arch::{Csr, Register, Width};
use crate::host::MiralisContext;

/// A RISC-V instruction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IllegalInstr {
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

/// Load (register-based)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LoadInstr {
    pub rd: Register,
    pub rs1: Register,
    pub imm: isize,
    pub len: Width,
    pub is_compressed: bool,
    pub is_unsigned: bool,
}

/// Store (register-based)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StoreInstr {
    pub rs2: Register,
    pub rs1: Register,
    pub imm: isize,
    pub len: Width,
    pub is_compressed: bool,
}

fn extract_last_two_bits(value: usize) -> usize {
    value & 0b11
}

impl MiralisContext {
    /// Decodes a raw read RISC-V instruction.
    pub fn decode_read(&self, raw: usize) -> LoadInstr {
        match extract_last_two_bits(raw) {
            0b11 => self.decode_load(raw),
            // Register-based load and store instructions for C set start with 0b00
            0b00 => self.decode_compressed_load(raw),
            _ => panic!("Unreachable statement"),
        }
    }

    /// Decodes a raw write RISC-V instruction.
    pub fn decode_write(&self, raw: usize) -> StoreInstr {
        match extract_last_two_bits(raw) {
            0b11 => self.decode_store(raw),
            // Register-based load and store instructions for C set start with 0b00
            0b00 => self.decode_compressed_store(raw),
            _ => panic!("unreachable code"),
        }
    }

    /// Decodes a raw illegal instruction
    ///
    /// NOTE: for now this function  only support 32 bits instructions.
    pub fn decode_illegal_instruction(&self, raw: usize) -> IllegalInstr {
        let rd = (raw >> 7) & 0b11111;
        let func3 = (raw >> 12) & 0b111;
        let rs1 = (raw >> 15) & 0b11111;
        let imm = (raw >> 20) & 0b111111111111;
        let func7 = (raw >> 25) & 0b1111111;
        if func3 == 0b000 {
            return match imm {
                0b000000000000 => IllegalInstr::Ecall,
                0b000000000001 => IllegalInstr::Ebreak,
                0b000100000101 => IllegalInstr::Wfi,
                0b001100000010 => IllegalInstr::Mret,
                _ if func7 == 0b0001001 => {
                    let rs1 = Register::from(rs1);
                    let rs2 = (raw >> 20) & 0b11111;
                    let rs2 = Register::from(rs2);
                    return IllegalInstr::Sfencevma { rs1, rs2 };
                }
                _ if func7 == 0b0010001 => {
                    let rs1 = Register::from(rs1);
                    let rs2 = (raw >> 20) & 0b11111;
                    let rs2 = Register::from(rs2);
                    return IllegalInstr::Hfencevvma { rs1, rs2 };
                }
                _ if func7 == 0b0110001 => {
                    let rs1 = Register::from(rs1);
                    let rs2 = (raw >> 20) & 0b11111;
                    let rs2 = Register::from(rs2);
                    return IllegalInstr::Hfencegvma { rs1, rs2 };
                }
                _ => IllegalInstr::Unknown,
            };
        }

        let csr = self.decode_csr(imm);
        let rd = Register::from(rd);
        match func3 {
            0b001 => IllegalInstr::Csrrw {
                csr,
                rd,
                rs1: Register::from(rs1),
            },
            0b010 => IllegalInstr::Csrrs {
                csr,
                rd,
                rs1: Register::from(rs1),
            },
            0b011 => IllegalInstr::Csrrc {
                csr,
                rd,
                rs1: Register::from(rs1),
            },
            0b101 => IllegalInstr::Csrrwi { csr, rd, uimm: rs1 },
            0b110 => IllegalInstr::Csrrsi { csr, rd, uimm: rs1 },
            0b111 => IllegalInstr::Csrrci { csr, rd, uimm: rs1 },
            _ => IllegalInstr::Unknown,
        }
    }

    fn decode_compressed_load(&self, raw: usize) -> LoadInstr {
        let func3 = (raw >> 13) & 0b111;
        let rd_rs2 = (raw >> 2) & 0b111;
        let rs1 = (raw >> 7) & 0b111;

        let rd_rs2 = Register::from(rd_rs2 + 8);
        let rs1 = Register::from(rs1 + 8);

        match func3 {
            0b011 => {
                let rd = rd_rs2;
                let imm = (raw >> 7) & 0b111000 | ((raw << 1) & 0b11000000);
                LoadInstr {
                    rd,
                    rs1,
                    imm: (imm * 8).try_into().unwrap(),
                    len: Width::from(64),
                    is_compressed: true,
                    is_unsigned: false,
                }
            }
            0b010 => {
                let rd = rd_rs2;
                let imm = (raw >> 3) & 0b100 | (raw >> 7) & 0b111000 | (raw << 1) & 0b1000000;
                LoadInstr {
                    rd,
                    rs1,
                    imm: (imm * 4).try_into().unwrap(),
                    len: Width::from(32),
                    is_compressed: true,
                    is_unsigned: false,
                }
            }
            _ => panic!("unreachable statement, something is wrong"),
        }
    }

    fn decode_compressed_store(&self, raw: usize) -> StoreInstr {
        let func3 = (raw >> 13) & 0b111;
        let rd_rs2 = (raw >> 2) & 0b111;
        let rs1 = (raw >> 7) & 0b111;

        let rd_rs2 = Register::from(rd_rs2 + 8);
        let rs1 = Register::from(rs1 + 8);

        match func3 {
            0b111 => {
                let rs2 = rd_rs2;
                let imm = (raw >> 7) & 0b111000 | ((raw << 1) & 0b11000000);
                StoreInstr {
                    rs2,
                    rs1,
                    imm: (imm * 8).try_into().unwrap(),
                    len: Width::from(64),
                    is_compressed: true,
                }
            }
            0b110 => {
                let rs2 = rd_rs2;
                let imm = (raw >> 3) & 0b100 | (raw >> 7) & 0b111000 | (raw << 1) & 0b1000000;
                StoreInstr {
                    rs2,
                    rs1,
                    imm: (imm * 4).try_into().unwrap(),
                    len: Width::from(32),
                    is_compressed: true,
                }
            }
            _ => panic!("unreachable code"),
        }
    }

    fn bits_to_int(&self, raw: usize, start_bit: isize, end_bit: isize) -> isize {
        let mask = (1 << (end_bit - start_bit + 1)) - 1;
        let value = (raw >> start_bit) & mask;

        // Check if the most significant bit is set (indicating a negative value)
        if value & (1 << (end_bit - start_bit)) != 0 {
            // Extend the sign bit to the left
            let sign_extension = !0 << (end_bit - start_bit);
            value as isize | sign_extension
        } else {
            value as isize
        }
    }
    fn decode_load(&self, raw: usize) -> LoadInstr {
        let func3 = (raw >> 12) & 0b111;
        let rd = (raw >> 7) & 0b11111;
        let rs1 = (raw >> 15) & 0b11111;
        let imm = self.bits_to_int(raw, 20, 31);

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
            _ => panic!("unreachable code"),
        }
    }

    fn decode_store(&self, raw: usize) -> StoreInstr {
        let func3 = (raw >> 12) & 0b111;
        let rs1: usize = (raw >> 15) & 0b11111;
        let rs2 = (raw >> 20) & 0b11111;
        let imm = self.bits_to_int(
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
            _ => panic!("unreachable code"),
        }
    }
    pub fn decode_csr(&self, csr: usize) -> Csr {
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
            0x3A0..=0x3AF => {
                let id = csr - 0x3A0;
                if id % 2 == 0 {
                    Csr::Pmpcfg(id)
                } else {
                    Csr::Unknown // Invalid on rv64
                }
            }
            0x3B0..=0x3EF => Csr::Pmpaddr(csr - 0x3B0),
            0xB00 => Csr::Mcycle,
            0xB02 => Csr::Minstret,
            0xC00 => {
                if self.hw.extensions.has_zicntr {
                    Csr::Cycle
                } else {
                    Csr::Unknown
                }
            }
            0xC01 => {
                if self.hw.extensions.has_zicntr {
                    Csr::Time
                } else {
                    Csr::Unknown
                }
            }
            0xC02 => {
                if self.hw.extensions.has_zicntr {
                    Csr::Instret
                } else {
                    Csr::Unknown
                }
            }
            0xB03..=0xB1F => {
                // Mhpm counters start at 3 and end at 31 : we shift them by 3 to start at 0 and end at 29
                if self.hw.extensions.has_zihpm_extension {
                    Csr::Mhpmcounter(csr - 0xB03)
                } else {
                    Csr::Unknown
                }
            }
            0x320 => Csr::Mcountinhibit,
            0x323..=0x33F => {
                if self.hw.extensions.has_zihpm_extension {
                    Csr::Mhpmevent(csr - 0x323)
                } else {
                    Csr::Unknown
                }
            }
            0x306 => Csr::Mcounteren,
            0x30a => Csr::Menvcfg,
            0x747 => Csr::Mseccfg,
            0xF15 => Csr::Mconfigptr,
            0x302 => {
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
            0x303 => {
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
            0x34A => {
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
            0x34B => {
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
            0x7A0 => Csr::Tselect,
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
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sstatus
                }
            }
            0x104 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sie
                }
            }
            0x105 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Stvec
                }
            }
            0x106 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Scounteren
                }
            }
            0x10A => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Senvcfg
                }
            }
            0x140 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sscratch
                }
            }
            0x141 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sepc
                }
            }
            0x142 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Scause
                }
            }
            0x143 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Stval
                }
            }
            0x144 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Sip
                }
            }
            0x14d => {
                if !self.hw.extensions.is_sstc_enabled {
                    Csr::Unknown
                } else {
                    Csr::Stimecmp
                }
            }
            0x180 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Satp
                }
            }
            0x5A8 => {
                if !self.hw.extensions.has_s_extension {
                    Csr::Unknown
                } else {
                    Csr::Scontext
                }
            }

            // Hypervisor and Virtual Supervisor CSRs
            0x600 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hstatus
                }
            }
            0x602 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hedeleg
                }
            }
            0x603 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hideleg
                }
            }
            0x645 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hvip
                }
            }
            0x644 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hip
                }
            }
            0x604 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hie
                }
            }
            0xe12 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hgeip
                }
            }
            0x607 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hgeie
                }
            }
            0x60a => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Henvcfg
                }
            }
            0x606 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hcounteren
                }
            }
            0x605 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Htimedelta
                }
            }
            0x643 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Htval
                }
            }
            0x64a => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Htinst
                }
            }
            0x680 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Hgatp
                }
            }
            0x200 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsstatus
                }
            }
            0x204 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsie
                }
            }
            0x205 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vstvec
                }
            }
            0x240 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsscratch
                }
            }
            0x241 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsepc
                }
            }
            0x242 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vscause
                }
            }
            0x243 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vstval
                }
            }
            0x244 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsip
                }
            }
            0x280 => {
                if !self.hw.extensions.has_h_extension {
                    Csr::Unknown
                } else {
                    Csr::Vsatp
                }
            }

            // Vector extension
            0x8 => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vstart
                }
            }
            0x9 => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vxsat
                }
            }
            0xa => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vxrm
                }
            }
            0xf => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vcsr
                }
            }
            0xc20 => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vl
                }
            }
            0xc21 => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vtype
                }
            }
            0xc22 => {
                if !self.hw.extensions.has_v_extension {
                    Csr::Unknown
                } else {
                    Csr::Vlenb
                }
            }
            0x15 => {
                // Crypto extension
                if !self.hw.extensions.has_crypto_extension {
                    Csr::Unknown
                } else {
                    Csr::Seed
                }
            }

            _ => {
                log::debug!("Unknown CSR: 0x{:x}", csr);
                Csr::Unknown
            }
        }
    }
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::{Arch, Architecture};

    /// Decodes priviledged instructions
    /// Here is an handy tool to double check:
    /// https://luplab.gitlab.io/rvcodecjs/
    #[test]
    fn system_instructions() {
        let mctx = MiralisContext::new(unsafe { Arch::detect_hardware() }, 0x100000, 0x2000);
        // ECALL: Environment call.
        assert_eq!(
            mctx.decode_illegal_instruction(0x00000073),
            IllegalInstr::Ecall
        );
        // EBREAK: Environment break.
        assert_eq!(
            mctx.decode_illegal_instruction(0x00100073),
            IllegalInstr::Ebreak
        );
        // MRET: Return from machine mode.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30200073),
            IllegalInstr::Mret
        );
        // WFI: Wait for interrupt.
        assert_eq!(
            mctx.decode_illegal_instruction(0x10500073),
            IllegalInstr::Wfi
        );
        // SFENCE.VMA: Supervisor memory-management fence.
        assert_eq!(
            mctx.decode_illegal_instruction(0x12000073),
            IllegalInstr::Sfencevma {
                rs1: Register::X0,
                rs2: Register::X0
            }
        );
        assert_eq!(
            mctx.decode_illegal_instruction(0x13300073),
            IllegalInstr::Sfencevma {
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
            IllegalInstr::Csrrw {
                csr: Csr::Mstatus,
                rd: Register::X0,
                rs1: Register::X0,
            }
        );

        // CSRRS: Atomic Read and Set Bits in CSR.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30002073),
            IllegalInstr::Csrrs {
                csr: Csr::Mstatus,
                rd: Register::X0,
                rs1: Register::X0,
            }
        );

        // CSRRC: Atomic Read and Clear Bits in CSR.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30003073),
            IllegalInstr::Csrrc {
                csr: Csr::Mstatus,
                rd: Register::X0,
                rs1: Register::X0,
            }
        );

        // CSRRWI: Atomic Read/Write CSR Immediate.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30005073),
            IllegalInstr::Csrrwi {
                csr: Csr::Mstatus,
                rd: Register::X0,
                uimm: 0x0,
            }
        );

        // CSRRSI: Atomic Read and Set Bits in CSR Immediate.
        assert_eq!(
            mctx.decode_illegal_instruction(0x30006073),
            IllegalInstr::Csrrsi {
                csr: Csr::Mstatus,
                rd: Register::X0,
                uimm: 0x0,
            }
        );

        // CSRRCI: Atomic Read and Clear Bits in CSR Immediate.s
        assert_eq!(
            mctx.decode_illegal_instruction(0x30007073),
            IllegalInstr::Csrrci {
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
            mctx.decode_read(0xff87b703),
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
            mctx.decode_write(0xfee7bc23),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(64),
                is_compressed: false,
            }
        );

        assert_eq!(
            mctx.decode_read(0xff87a703),
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
            mctx.decode_write(0xfee7ac23),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(32),
                is_compressed: false,
            }
        );

        assert_eq!(
            mctx.decode_read(0xff879703),
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
            mctx.decode_write(0xfee79c23),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(16),
                is_compressed: false,
            }
        );

        assert_eq!(
            mctx.decode_read(0xff878703),
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
            mctx.decode_write(0xfee78c23),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: -8,
                len: Width::from(8),
                is_compressed: false,
            }
        );

        // Note: For compressed instructions, the immediate value (imm) represents an offset
        // and is multiplied by the necessary factor (e.g., x4/8) during decoding.
        // Therefore, we actually decode an offset as the immediate value for compressed instructions.

        // Example:  0xffff4798 would decode into c.lw x14, 8(x15) (immediate value is 8),
        // but we assert imm as 32 (8*4), as the eventual offset value would be 32.
        // This helps to keep compressed and uncompressed instructions handlers more homogenous in other modules.

        assert_eq!(
            mctx.decode_write(0xffffe798),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: 64,
                len: Width::from(64),
                is_compressed: true,
            }
        );

        assert_eq!(
            mctx.decode_read(0xffff6798),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: 64,
                len: Width::from(64),
                is_compressed: true,
                is_unsigned: false,
            }
        );

        assert_eq!(
            mctx.decode_read(0xffff4798),
            LoadInstr {
                rd: Register::X14,
                rs1: Register::X15,
                imm: 32,
                len: Width::from(32),
                is_compressed: true,
                is_unsigned: false,
            }
        );

        assert_eq!(
            mctx.decode_write(0xffffc798),
            StoreInstr {
                rs2: Register::X14,
                rs1: Register::X15,
                imm: 32,
                len: Width::from(32),
                is_compressed: true,
            }
        );

        assert_eq!(
            mctx.decode_read(0xff87e703),
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
            mctx.decode_read(0xff87d703),
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
            mctx.decode_read(0xff87c703),
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
                IllegalInstr::Csrrw {
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
                IllegalInstr::Csrrw {
                    csr: Csr::Mstatus,
                    rd: Register::X0,
                    rs1: tuple.1,
                }
            );
        }
    }
}
