//! Tools to convert between Sail and Miralis structures
//!
//! Sail and Miralis each develop their own internal representation of a RISC-V machine
//! independently. For the purpose of checking the equivalence between the reference Sail
//! implementation and Miralis we need to be able to compare their internal representation. Hence
//! this module exposing functions to convert from one representation to the other.

use miralis::arch::{Csr, Mode, Register, Width};
use miralis::decoder::{IllegalInst, LoadInstr, StoreInstr};
use miralis::host::MiralisContext;
use miralis::virt::VirtContext;
use softcore_rv64::config::U74;
use softcore_rv64::prelude::{bv, BitVector};
use softcore_rv64::raw::{ast, csrop, word_width, Core, Pmpcfg_ent, Privilege};
use softcore_rv64::{new_core, raw, registers as reg};

pub fn miralis_to_rv_core(ctx: &VirtContext) -> Core {
    let mut core = new_core(U74);
    core.reset();

    core.nextPC = bv(ctx.pc as u64);
    core.PC = bv(ctx.pc as u64);

    core.cur_privilege = match ctx.mode {
        Mode::U => Privilege::User,
        Mode::S => Privilege::Supervisor,
        Mode::M => Privilege::Machine,
    };

    // Transfer hart id
    core.mhartid = bv(ctx.hart_id as u64);

    // Transfer all csr
    core.mstatus = raw::Mstatus {
        bits: bv(ctx.csr.mstatus as u64),
    };
    core.misa = raw::Misa {
        bits: bv(ctx.csr.misa as u64),
    };
    core.mie = raw::Minterrupts {
        bits: bv(ctx.csr.mie as u64),
    };
    core.mip = raw::Minterrupts {
        bits: bv(ctx.csr.mip as u64),
    };
    core.mtvec = raw::Mtvec {
        bits: bv(ctx.csr.mtvec as u64),
    };
    core.mscratch = bv(ctx.csr.mscratch as u64);
    core.mvendorid = bv(ctx.csr.mvendorid as u64);
    core.marchid = bv(ctx.csr.marchid as u64);
    core.mimpid = bv(ctx.csr.mimpid as u64);
    core.mcycle = bv(ctx.csr.mcycle as u64);
    core.minstret = bv(ctx.csr.minstret as u64);
    core.mcountinhibit = raw::Counterin {
        bits: bv(ctx.csr.mcountinhibit as u64),
    };
    core.mcounteren = raw::Counteren {
        bits: bv(ctx.csr.mcounteren as u64),
    };
    core.menvcfg = raw::MEnvcfg {
        bits: bv(ctx.csr.menvcfg as u64),
    };
    // sail_ctx.mseccfg = BitField::new(ctx.csr.mseccfg as u64);
    core.mcause = raw::Mcause {
        bits: bv(ctx.csr.mcause as u64),
    };
    core.mepc = bv(ctx.csr.mepc as u64);
    core.mtval = bv(ctx.csr.mtval as u64);
    // sail_ctx.mtval2 = BitField::new(ctx.csr.mtval2 as u64);
    core.mstatus = raw::Mstatus {
        bits: bv(ctx.csr.mstatus as u64),
    };
    // sail_ctx.mtinst = BitField::new(ctx.csr.mtinst as u64);

    core.mconfigptr = bv(ctx.csr.mconfigptr as u64);
    core.stvec = raw::Mtvec {
        bits: bv(ctx.csr.stvec as u64),
    };
    core.scounteren = raw::Counteren {
        bits: bv(ctx.csr.scounteren as u64),
    };
    core.senvcfg = raw::SEnvcfg {
        bits: bv(ctx.csr.senvcfg as u64),
    };

    core.sscratch = bv(ctx.csr.sscratch as u64);
    core.sepc = bv(ctx.csr.sepc as u64);
    core.scause = raw::Mcause {
        bits: bv(ctx.csr.scause as u64),
    };
    core.stval = bv(ctx.csr.stval as u64);
    core.satp = bv(ctx.csr.satp as u64);
    // sail_ctx.scontext = BitField::new(ctx.csr.scontext as u64);
    core.medeleg = raw::Medeleg {
        bits: bv(ctx.csr.medeleg as u64),
    };
    core.mideleg = raw::Minterrupts {
        bits: bv(ctx.csr.mideleg as u64),
    };

    core.pmpcfg_n = pmpcfg_miralis_to_sail(ctx.csr.pmpcfg);
    core.pmpaddr_n = pmpaddr_miralis_to_sail(ctx.csr.pmpaddr);
    // ctx.csr.mhpmcounter=  [kani::any(); 29]; TODO: What should we do with this?
    // ctx.csr.mhpmevent=  [kani::any(); 29]; TODO: What should we do with this?

    // New added
    core.tselect = BitVector::<64>::new(ctx.csr.tselect as u64);
    core.vstart = BitVector::<16>::new(ctx.csr.vstart as u64);
    core.vcsr = raw::Vcsr {
        bits: bv(ctx.csr.vcsr as u64),
    };
    core.vl = bv(ctx.csr.vl as u64);
    core.vtype = raw::Vtype {
        bits: bv(ctx.csr.vtype as u64),
    };

    // Transfer the general purpose registers
    core.x1 = bv(ctx.regs[1] as u64);
    core.x2 = bv(ctx.regs[2] as u64);
    core.x3 = bv(ctx.regs[3] as u64);
    core.x4 = bv(ctx.regs[4] as u64);
    core.x5 = bv(ctx.regs[5] as u64);
    core.x6 = bv(ctx.regs[6] as u64);
    core.x7 = bv(ctx.regs[7] as u64);
    core.x8 = bv(ctx.regs[8] as u64);
    core.x9 = bv(ctx.regs[9] as u64);
    core.x10 = bv(ctx.regs[10] as u64);
    core.x11 = bv(ctx.regs[11] as u64);
    core.x12 = bv(ctx.regs[12] as u64);
    core.x13 = bv(ctx.regs[13] as u64);
    core.x14 = bv(ctx.regs[14] as u64);
    core.x15 = bv(ctx.regs[15] as u64);
    core.x16 = bv(ctx.regs[16] as u64);
    core.x17 = bv(ctx.regs[17] as u64);
    core.x18 = bv(ctx.regs[18] as u64);
    core.x19 = bv(ctx.regs[19] as u64);
    core.x20 = bv(ctx.regs[20] as u64);
    core.x21 = bv(ctx.regs[21] as u64);
    core.x22 = bv(ctx.regs[22] as u64);
    core.x23 = bv(ctx.regs[23] as u64);
    core.x24 = bv(ctx.regs[24] as u64);
    core.x25 = bv(ctx.regs[25] as u64);
    core.x26 = bv(ctx.regs[26] as u64);
    core.x27 = bv(ctx.regs[27] as u64);
    core.x28 = bv(ctx.regs[28] as u64);
    core.x29 = bv(ctx.regs[29] as u64);
    core.x30 = bv(ctx.regs[30] as u64);
    core.x31 = bv(ctx.regs[31] as u64);

    core
}

pub fn pmpcfg_miralis_to_sail(cfgs: [usize; 8]) -> [Pmpcfg_ent; 64] {
    let mut output: [Pmpcfg_ent; 64] = [Pmpcfg_ent { bits: bv(0) }; 64];

    for i in 0..64 {
        let idx = i / 8;
        let offset = i % 8;
        output[i].bits = bv(((cfgs[idx] >> (8 * offset)) & 0xFF) as u64);
    }

    output
}

pub fn pmpaddr_miralis_to_sail(addresses: [usize; 64]) -> [BitVector<64>; 64] {
    let mut output: [BitVector<64>; 64] = [BitVector::<64>::new(0); 64];
    for i in 0..64 {
        output[i] = bv(addresses[i] as u64);
    }

    output
}

pub fn rv_core_to_miralis(mut sail_ctx: Core, mctx: &MiralisContext) -> VirtContext {
    let mut ctx = VirtContext::new(0, 0, mctx.hw.extensions.clone());

    ctx.mode = match sail_ctx.cur_privilege {
        Privilege::User => Mode::U,
        Privilege::Supervisor => Mode::S,
        Privilege::Machine => Mode::M,
    };

    ctx.pc = sail_ctx.nextPC.bits() as usize;

    // Transfer hart id
    ctx.hart_id = sail_ctx.mhartid.bits() as usize;

    ctx.nb_pmp = sail_ctx.config.memory.pmp.count as usize;
    ctx.pmp_grain = sail_ctx.config.memory.pmp.grain as usize;

    // Transfer all csr
    ctx.csr.mstatus = sail_ctx.mstatus.bits.bits() as usize;
    ctx.csr.misa = sail_ctx.misa.bits.bits() as usize;
    ctx.csr.mie = sail_ctx.mie.bits.bits() as usize;
    ctx.csr.mip = sail_ctx.mip.bits.bits() as usize;
    ctx.csr.mtvec = sail_ctx.mtvec.bits.bits() as usize;
    ctx.csr.mscratch = sail_ctx.mscratch.bits() as usize;
    ctx.csr.mvendorid = sail_ctx.mvendorid.bits() as u32;
    ctx.csr.marchid = sail_ctx.marchid.bits() as usize;
    ctx.csr.mimpid = sail_ctx.mimpid.bits() as usize;
    ctx.csr.mcycle = sail_ctx.mcycle.bits() as usize;
    ctx.csr.minstret = sail_ctx.minstret.bits() as usize;
    ctx.csr.mcountinhibit = sail_ctx.mcountinhibit.bits.bits() as u32;
    ctx.csr.mcounteren = sail_ctx.mcounteren.bits.bits() as u32;
    ctx.csr.menvcfg = sail_ctx.menvcfg.bits.bits() as usize;
    // ctx.csr.mseccfg= sail_ctx.mseccfg.bits.bits() as usize;
    ctx.csr.mcause = sail_ctx.mcause.bits.bits() as usize;
    ctx.csr.mepc = sail_ctx.mepc.bits() as usize;
    ctx.csr.mtval = sail_ctx.mtval.bits() as usize;
    ctx.csr.mconfigptr = sail_ctx.mconfigptr.bits() as usize;
    ctx.csr.stvec = sail_ctx.stvec.bits.bits() as usize;
    ctx.csr.scounteren = sail_ctx.scounteren.bits.bits() as u32;
    ctx.csr.senvcfg = sail_ctx.senvcfg.bits.bits() as usize;
    ctx.csr.sscratch = sail_ctx.sscratch.bits() as usize;
    ctx.csr.sepc = sail_ctx.sepc.bits() as usize;
    ctx.csr.scause = sail_ctx.scause.bits.bits() as usize;
    ctx.csr.stval = sail_ctx.stval.bits() as usize;
    ctx.csr.satp = sail_ctx.satp.bits() as usize;
    // ctx.csr.scontext= sail_ctx.scontext.bits.bits() as usize;
    ctx.csr.medeleg = sail_ctx.medeleg.bits.bits() as usize;
    ctx.csr.mideleg = sail_ctx.mideleg.bits.bits() as usize;
    ctx.csr.pmpcfg = pmpcfg_sail_to_miralis(sail_ctx.pmpcfg_n);
    ctx.csr.pmpaddr = pmpaddr_sail_to_miralis(sail_ctx.pmpaddr_n);
    // ctx.csr.mhpmcounter=  [kani::any(); 29]; todo: what should we do?
    // ctx.csr.mhpmevent=  [kani::any(); 29]; todo: what should we do?

    // New added
    ctx.csr.tselect = sail_ctx.tselect.bits() as usize;
    ctx.csr.vstart = sail_ctx.vstart.bits() as u16;
    ctx.csr.vcsr = sail_ctx.vcsr.bits.bits() as u8;
    ctx.csr.vl = sail_ctx.vl.bits() as usize;
    ctx.csr.vtype = sail_ctx.vtype.bits.bits() as usize;

    // Transfer the general purpose registers
    ctx.regs[1] = sail_ctx.get(reg::X1) as usize;
    ctx.regs[2] = sail_ctx.get(reg::X2) as usize;
    ctx.regs[3] = sail_ctx.get(reg::X3) as usize;
    ctx.regs[4] = sail_ctx.get(reg::X4) as usize;
    ctx.regs[5] = sail_ctx.get(reg::X5) as usize;
    ctx.regs[6] = sail_ctx.get(reg::X6) as usize;
    ctx.regs[7] = sail_ctx.get(reg::X7) as usize;
    ctx.regs[8] = sail_ctx.get(reg::X8) as usize;
    ctx.regs[9] = sail_ctx.get(reg::X9) as usize;
    ctx.regs[10] = sail_ctx.get(reg::X10) as usize;
    ctx.regs[11] = sail_ctx.get(reg::X11) as usize;
    ctx.regs[12] = sail_ctx.get(reg::X12) as usize;
    ctx.regs[13] = sail_ctx.get(reg::X13) as usize;
    ctx.regs[14] = sail_ctx.get(reg::X14) as usize;
    ctx.regs[15] = sail_ctx.get(reg::X15) as usize;
    ctx.regs[16] = sail_ctx.get(reg::X16) as usize;
    ctx.regs[17] = sail_ctx.get(reg::X17) as usize;
    ctx.regs[18] = sail_ctx.get(reg::X18) as usize;
    ctx.regs[19] = sail_ctx.get(reg::X19) as usize;
    ctx.regs[20] = sail_ctx.get(reg::X20) as usize;
    ctx.regs[21] = sail_ctx.get(reg::X21) as usize;
    ctx.regs[22] = sail_ctx.get(reg::X22) as usize;
    ctx.regs[23] = sail_ctx.get(reg::X23) as usize;
    ctx.regs[24] = sail_ctx.get(reg::X24) as usize;
    ctx.regs[25] = sail_ctx.get(reg::X25) as usize;
    ctx.regs[26] = sail_ctx.get(reg::X26) as usize;
    ctx.regs[27] = sail_ctx.get(reg::X27) as usize;
    ctx.regs[28] = sail_ctx.get(reg::X28) as usize;
    ctx.regs[29] = sail_ctx.get(reg::X29) as usize;
    ctx.regs[30] = sail_ctx.get(reg::X30) as usize;
    ctx.regs[31] = sail_ctx.get(reg::X31) as usize;

    ctx
}

pub fn pmpcfg_sail_to_miralis(cfgs: [Pmpcfg_ent; 64]) -> [usize; 8] {
    let mut output: [usize; 8] = [0; 8];

    for i in 0..64 {
        let idx = i / 8;
        let offset = i % 8;
        output[idx] |= ((cfgs[i].bits.bits() & 0xff) << (8 * offset)) as usize;
    }

    output
}

pub fn pmpaddr_sail_to_miralis(addresses: [BitVector<64>; 64]) -> [usize; 64] {
    let mut output: [usize; 64] = [0; 64];
    for i in 0..64 {
        output[i] = addresses[i].bits() as usize;
    }

    output
}

pub fn decode_csr_register(arg_hashtag_: BitVector<12>) -> Csr {
    if 0b001110100000 <= arg_hashtag_.bits()
        && arg_hashtag_.bits() <= (0b001110100000 + 15)
        && arg_hashtag_.bits() % 2 == 0
    {
        return Csr::Pmpcfg((arg_hashtag_.bits() - 0b001110100000) as usize);
    }

    if 0b001110110000 <= arg_hashtag_.bits() && arg_hashtag_.bits() <= (0b001110110000 + 63) {
        return Csr::Pmpaddr((arg_hashtag_.bits() - 0b001110110000) as usize);
    }

    match arg_hashtag_ {
        b_18 if { b_18 == BitVector::<12>::new(0b000100000000) } => Csr::Sstatus,
        b_21 if { b_21 == BitVector::<12>::new(0b000100000100) } => Csr::Sie,
        b_22 if { b_22 == BitVector::<12>::new(0b000100000101) } => Csr::Stvec,
        b_23 if { b_23 == BitVector::<12>::new(0b000100000110) } => Csr::Scounteren,
        b_24 if { b_24 == BitVector::<12>::new(0b000101000000) } => Csr::Sscratch,
        b_25 if { b_25 == BitVector::<12>::new(0b000101000001) } => Csr::Sepc,
        b_26 if { b_26 == BitVector::<12>::new(0b000101000010) } => Csr::Scause,
        b_27 if { b_27 == BitVector::<12>::new(0b000101000011) } => Csr::Stval,
        b_28 if { b_28 == BitVector::<12>::new(0b000101000100) } => Csr::Sip,
        b_29 if { b_29 == BitVector::<12>::new(0b000110000000) } => Csr::Satp,
        b_30 if { b_30 == BitVector::<12>::new(0b000100001010) } => Csr::Senvcfg,
        b_31 if { b_31 == BitVector::<12>::new(0b111100010001) } => Csr::Mvendorid,
        b_32 if { b_32 == BitVector::<12>::new(0b111100010010) } => Csr::Marchid,
        b_34 if { b_34 == BitVector::<12>::new(0b111100010100) } => Csr::Mhartid,
        b_35 if { b_35 == BitVector::<12>::new(0b111100010101) } => Csr::Mconfigptr,
        b_36 if { b_36 == BitVector::<12>::new(0b001100000000) } => Csr::Mstatus,
        b_37 if { b_37 == BitVector::<12>::new(0b001100000001) } => Csr::Misa,
        b_38 if { b_38 == BitVector::<12>::new(0b001100000010) } => Csr::Medeleg,
        b_39 if { b_39 == BitVector::<12>::new(0b001100000011) } => Csr::Mideleg,
        b_40 if { b_40 == BitVector::<12>::new(0b001100000100) } => Csr::Mie,
        b_41 if { b_41 == BitVector::<12>::new(0b001100000101) } => Csr::Mtvec,
        b_42 if { b_42 == BitVector::<12>::new(0b001100000110) } => Csr::Mcounteren,
        b_43 if { b_43 == BitVector::<12>::new(0b001100100000) } => Csr::Mcountinhibit,
        b_44 if { b_44 == BitVector::<12>::new(0b001100001010) } => Csr::Menvcfg,
        b_45 if { b_45 == BitVector::<12>::new(0b001101000000) } => Csr::Mscratch,
        b_46 if { b_46 == BitVector::<12>::new(0b001101000001) } => Csr::Mepc,
        b_47 if { b_47 == BitVector::<12>::new(0b001101000010) } => Csr::Mcause,
        b_48 if { b_48 == BitVector::<12>::new(0b001101000011) } => Csr::Mtval,
        b_49 if { b_49 == BitVector::<12>::new(0b001101000100) } => Csr::Mip,
        b_134 if { b_134 == BitVector::<12>::new(0b011110100000) } => Csr::Tselect,
        b_12 if { b_12 == BitVector::<12>::new(0b110000000000) } => Csr::Cycle,
        b_13 if { b_13 == BitVector::<12>::new(0b110000000001) } => Csr::Time,
        b_14 if { b_14 == BitVector::<12>::new(0b110000000010) } => Csr::Instret,
        b_130 if { b_130 == BitVector::<12>::new(0b101100000000) } => Csr::Mcycle,
        b_131 if { b_131 == BitVector::<12>::new(0b101100000010) } => Csr::Minstret,
        // Manually removed: the new version of softcore disables the corresponding features
        // or the core model we use does not have them.
        // b_135 if { b_135 == BitVector::<12>::new(0b011110100001) } => Csr::Tdata1,
        // b_136 if { b_136 == BitVector::<12>::new(0b011110100010) } => Csr::Tdata2,
        // b_137 if { b_137 == BitVector::<12>::new(0b011110100011) } => Csr::Tdata3,
        // b_138 if { b_138 == BitVector::<12>::new(0b000000001000) } => Csr::Vstart,
        // b_139 if { b_139 == BitVector::<12>::new(0b000000001001) } => Csr::Vxsat,
        // b_140 if { b_140 == BitVector::<12>::new(0b000000001010) } => Csr::Vxrm,
        // b_141 if { b_141 == BitVector::<12>::new(0b000000001111) } => Csr::Vcsr,
        // b_142 if { b_142 == BitVector::<12>::new(0b110000100000) } => Csr::Vl,
        // b_143 if { b_143 == BitVector::<12>::new(0b110000100001) } => Csr::Vtype,
        // b_144 if { b_144 == BitVector::<12>::new(0b110000100010) } => Csr::Vlenb,
        // b_11 if { b_11 == BitVector::<12>::new(0b000000010101) } => Csr::Seed,
        // Manually added
        b_155 if { bv(0x5A8) == b_155 } => Csr::Scontext,
        b_156 if { bv(0xf13) == b_156 } => Csr::Mimpid,
        // End manually added
        _ => Csr::Unknown,
    }
}

fn size_to_width(size: word_width) -> Width {
    match size {
        word_width::BYTE => Width::Byte,
        word_width::HALF => Width::Byte2,
        word_width::WORD => Width::Byte4,
        word_width::DOUBLE => Width::Byte8,
    }
}

pub fn ast_to_miralis_load(ast_entry: ast) -> LoadInstr {
    match ast_entry {
        ast::C_LW((imm, rs1, rd)) => LoadInstr {
            rd: Register::from(rd.bits() as usize + 8),
            rs1: Register::from(rs1.bits() as usize + 8),
            imm: (imm.bits() << 2) as isize,
            len: Width::Byte4,
            is_compressed: true,
            is_unsigned: false,
        },
        ast::C_LD((imm, rs1, rd)) => LoadInstr {
            rd: Register::from(rd.bits() as usize + 8),
            rs1: Register::from(rs1.bits() as usize + 8),
            imm: (imm.bits() << 3) as isize,
            len: Width::Byte8,
            is_compressed: true,
            is_unsigned: false,
        },
        ast::LOAD((imm, rs1, rd, is_unsigned, size, ..)) => LoadInstr {
            rd: Register::from(rd.bits() as usize),
            rs1: Register::from(rs1.bits() as usize),
            imm: imm.signed() as isize,
            len: size_to_width(size),
            is_compressed: false,
            is_unsigned,
        },
        _ => unreachable!(),
    }
}

pub fn ast_to_miralis_store(ast_entry: ast) -> StoreInstr {
    match ast_entry {
        ast::C_SW((imm, rs1, rs2)) => StoreInstr {
            rs2: Register::from(rs2.bits() as usize + 8),
            rs1: Register::from(rs1.bits() as usize + 8),
            imm: (imm.bits() << 2) as isize,
            len: Width::Byte4,
            is_compressed: true,
        },
        ast::C_SD((imm, rs1, rs2)) => StoreInstr {
            rs2: Register::from(rs2.bits() as usize + 8),
            rs1: Register::from(rs1.bits() as usize + 8),
            imm: (imm.bits() << 3) as isize,
            len: Width::Byte8,
            is_compressed: true,
        },

        ast::STORE((imm, rs2, rs1, size, ..)) => StoreInstr {
            rs2: Register::from(rs2.bits() as usize),
            rs1: Register::from(rs1.bits() as usize),
            imm: imm.signed() as isize,
            len: size_to_width(size),
            is_compressed: false,
        },
        _ => unreachable!(),
    }
}

pub fn ast_to_miralis_instr(ast_entry: ast) -> IllegalInst {
    match ast_entry {
        ast::MRET(()) => IllegalInst::Mret,
        ast::WFI(()) => IllegalInst::Wfi,
        ast::ECALL(()) => IllegalInst::Unknown, // Miralis does not need to decode ecalls"
        ast::EBREAK(()) => IllegalInst::Unknown, // Miralis does not need to decode ebreaks
        ast::SFENCE_VMA((rs1, rs2)) => IllegalInst::Sfencevma {
            rs1: Register::from(rs1.bits() as usize),
            rs2: Register::from(rs2.bits() as usize),
        },
        // NOTE: Uncomment those once upstream softcore-rv64 adds support for H-mode
        //
        // ast::HFENCE_VVMA((rs1, rs2)) => IllegalInst::Hfencevvma {
        //     rs1: Register::from(rs1.bits as usize),
        //     rs2: Register::from(rs2.bits as usize),
        // },
        // ast::HFENCE_GVMA((rs1, rs2)) => IllegalInst::Hfencegvma {
        //     rs1: Register::from(rs1.bits as usize),
        //     rs2: Register::from(rs2.bits as usize),
        // },
        ast::CSRReg((csrreg, rs1, rd, op)) => {
            let csr_register: Csr = decode_csr_register(csrreg);
            let rs1_miralis = Register::from(rs1.bits() as usize);
            let rd_miralis = Register::from(rd.bits() as usize);

            match op {
                csrop::CSRRW => IllegalInst::Csrrw {
                    csr: csr_register,
                    rd: rd_miralis,
                    rs1: rs1_miralis,
                },
                csrop::CSRRC => IllegalInst::Csrrc {
                    csr: csr_register,
                    rd: rd_miralis,
                    rs1: rs1_miralis,
                },
                csrop::CSRRS => IllegalInst::Csrrs {
                    csr: csr_register,
                    rd: rd_miralis,
                    rs1: rs1_miralis,
                },
            }
        }
        ast::CSRImm((csrreg, imm, rd, op)) => {
            let csr_register: Csr = decode_csr_register(csrreg);
            let rd_miralis = Register::from(rd.bits() as usize);

            match op {
                csrop::CSRRW => IllegalInst::Csrrwi {
                    csr: csr_register,
                    rd: rd_miralis,
                    uimm: imm.bits() as usize,
                },
                csrop::CSRRC => IllegalInst::Csrrci {
                    csr: csr_register,
                    rd: rd_miralis,
                    uimm: imm.bits() as usize,
                },
                csrop::CSRRS => IllegalInst::Csrrsi {
                    csr: csr_register,
                    rd: rd_miralis,
                    uimm: imm.bits() as usize,
                },
            }
        }
        _ => IllegalInst::Unknown,
    }
}
