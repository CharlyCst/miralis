//! Tools to convert between Sail and Miralis structures
//!
//! Sail and Miralis each develop their own internal representation of a RISC-V machine
//! independently. For the purpose of checking the equivalence between the reference Sail
//! implementation and Miralis we need to be able to compare their internal representation. Hence
//! this module exposing functions to convert from one representation to the other.

use miralis::arch::{ExtensionsCapability, Mode};
use miralis::virt::VirtContext;
use sail_model::{Privilege, SailVirtCtx};
use sail_prelude::{BitField, BitVector};

pub fn miralis_to_sail(ctx: &VirtContext) -> SailVirtCtx {
    let mut sail_ctx = new_sail_ctx();

    sail_ctx.nextPC = BitVector::new(ctx.pc as u64);
    sail_ctx.PC = BitVector::new(ctx.pc as u64);

    sail_ctx.cur_privilege = match ctx.mode {
        Mode::U => Privilege::User,
        Mode::S => Privilege::Supervisor,
        Mode::M => Privilege::Machine,
    };

    // Transfer hart id
    sail_ctx.mhartid = BitVector::new(ctx.hart_id as u64);

    // Transfer all csr
    sail_ctx.mstatus = BitField::new(ctx.csr.mstatus as u64);
    sail_ctx.misa = BitField::new(ctx.csr.misa as u64);
    sail_ctx.mie = BitField::new(ctx.csr.mie as u64);
    sail_ctx.mip = BitField::new(ctx.csr.mip as u64);
    sail_ctx.mtvec = BitField::new(ctx.csr.mtvec as u64);
    sail_ctx.mscratch = BitVector::new(ctx.csr.mscratch as u64);
    sail_ctx.mvendorid = BitVector::new(ctx.csr.mvendorid as u64);
    sail_ctx.marchid = BitVector::new(ctx.csr.marchid as u64);
    sail_ctx.mimpid = BitVector::new(ctx.csr.mimpid as u64);
    sail_ctx.mcycle = BitVector::new(ctx.csr.mcycle as u64);
    sail_ctx.minstret = BitVector::new(ctx.csr.minstret as u64);
    sail_ctx.mcountinhibit = BitField::new(ctx.csr.mcountinhibit as u64);
    sail_ctx.mcounteren = BitField::new(ctx.csr.mcounteren as u64);
    sail_ctx.menvcfg = BitField::new(ctx.csr.menvcfg as u64);
    // sail_ctx.mseccfg = BitField::new(ctx.csr.mseccfg as u64);
    sail_ctx.mcause = BitField::new(ctx.csr.mcause as u64);
    sail_ctx.mepc = BitVector::new(ctx.csr.mepc as u64);
    sail_ctx.mtval = BitVector::new(ctx.csr.mtval as u64);
    // sail_ctx.mtval2 = BitField::new(ctx.csr.mtval2 as u64);
    sail_ctx.mstatus = BitField::new(ctx.csr.mstatus as u64);
    // sail_ctx.mtinst = BitField::new(ctx.csr.mtinst as u64);

    sail_ctx.mconfigptr = BitVector::new(ctx.csr.mconfigptr as u64);
    sail_ctx.stvec = BitField::new(ctx.csr.stvec as u64);
    sail_ctx.scounteren = BitField::new(ctx.csr.scounteren as u64);
    sail_ctx.senvcfg = BitField::new(ctx.csr.senvcfg as u64);

    sail_ctx.sscratch = BitVector::new(ctx.csr.sscratch as u64);
    sail_ctx.sepc = BitVector::new(ctx.csr.sepc as u64);
    sail_ctx.scause = BitField::new(ctx.csr.scause as u64);
    sail_ctx.stval = BitVector::new(ctx.csr.stval as u64);
    sail_ctx.satp = BitVector::new(ctx.csr.satp as u64);
    // sail_ctx.scontext = BitField::new(ctx.csr.scontext as u64);
    sail_ctx.medeleg = BitField::new(ctx.csr.medeleg as u64);
    sail_ctx.mideleg = BitField::new(ctx.csr.mideleg as u64);

    sail_ctx.pmpcfg_n = pmpcfg_miralis_to_sail(ctx.csr.pmpcfg);
    sail_ctx.pmpaddr_n = pmpaddr_miralis_to_sail(ctx.csr.pmpaddr);
    // ctx.csr.mhpmcounter=  [kani::any(); 29]; TODO: What should we do with this?
    // ctx.csr.mhpmevent=  [kani::any(); 29]; TODO: What should we do with this?

    // New added
    sail_ctx.tselect = BitVector::<64>::new(ctx.csr.tselect as u64);
    sail_ctx.vstart = BitVector::<16>::new(ctx.csr.vstart as u64);
    sail_ctx.vxsat = BitVector::new(if ctx.csr.vxsat { 1 } else { 0 });
    sail_ctx.vxrm = BitVector::new(ctx.csr.vxrm as u64);
    sail_ctx.vcsr = BitField::new(ctx.csr.vcsr as u64);
    sail_ctx.vl = BitVector::new(ctx.csr.vl as u64);
    sail_ctx.vtype = BitField::new(ctx.csr.vtype as u64);
    sail_ctx.vlenb = BitVector::new(ctx.csr.vlenb as u64);

    sail_ctx
}

fn pmpcfg_miralis_to_sail(cfgs: [usize; 8]) -> [BitField<8>; 64] {
    let mut output: [BitField<8>; 64] = [BitField::<8>::new(0); 64];

    for i in 0..64 {
        let idx = i / 8;
        let offset = i % 8;
        output[i] = BitField::<8>::new(((cfgs[idx] >> (8 * offset)) & 0xFF) as u64);
    }

    output
}

fn pmpaddr_miralis_to_sail(addresses: [usize; 64]) -> [BitVector<64>; 64] {
    let mut output: [BitVector<64>; 64] = [BitVector::<64>::new(0); 64];
    for i in 0..64 {
        output[i] = BitVector::new(addresses[i] as u64);
    }

    output
}

/// Creates a fresh Sail context
///
/// NOTE: in the future we hope to replace it with a [Default] implementation, but there are some
/// blockers for now (e.g. arrays of 64 elements do not yet implement [Default] and we can't set
/// default values for members).
pub fn new_sail_ctx() -> SailVirtCtx {
    SailVirtCtx {
        elen: BitVector::default(),
        vlen: BitVector { bits: 3 },
        __monomorphize_reads: false,
        __monomorphize_writes: false,
        PC: BitVector::default(),
        nextPC: BitVector::default(),
        instbits: BitVector::default(),
        x1: BitVector::default(),
        x2: BitVector::default(),
        x3: BitVector::default(),
        x4: BitVector::default(),
        x5: BitVector::default(),
        x6: BitVector::default(),
        x7: BitVector::default(),
        x8: BitVector::default(),
        x9: BitVector::default(),
        x10: BitVector::default(),
        x11: BitVector::default(),
        x12: BitVector::default(),
        x13: BitVector::default(),
        x14: BitVector::default(),
        x15: BitVector::default(),
        x16: BitVector::default(),
        x17: BitVector::default(),
        x18: BitVector::default(),
        x19: BitVector::default(),
        x20: BitVector::default(),
        x21: BitVector::default(),
        x22: BitVector::default(),
        x23: BitVector::default(),
        x24: BitVector::default(),
        x25: BitVector::default(),
        x26: BitVector::default(),
        x27: BitVector::default(),
        x28: BitVector::default(),
        x29: BitVector::default(),
        x30: BitVector::default(),
        x31: BitVector::default(),
        cur_privilege: Privilege::User,
        cur_inst: BitVector::default(),
        misa: BitField::default(),
        mstatush: BitField::default(),
        mstatus: BitField::default(),
        mip: BitField::default(),
        mie: BitField::default(),
        mideleg: BitField::default(),
        medeleg: BitField::default(),
        mtvec: BitField::default(),
        mcause: BitField::default(),
        mepc: BitVector::default(),
        mtval: BitVector::default(),
        mscratch: BitVector::default(),
        mcounteren: BitField::default(),
        scounteren: BitField::default(),
        mcountinhibit: BitField::default(),
        mcycle: BitVector::default(),
        mtime: BitVector::default(),
        minstret: BitVector::default(),
        minstret_increment: false,
        mvendorid: BitVector::default(),
        mimpid: BitVector::default(),
        marchid: BitVector::default(),
        mhartid: BitVector::default(),
        mconfigptr: BitVector::default(),
        sedeleg: BitField::default(),
        sideleg: BitField::default(),
        stvec: BitField::default(),
        sscratch: BitVector::default(),
        sepc: BitVector::default(),
        scause: BitField::default(),
        stval: BitVector::default(),
        tselect: BitVector::default(),
        menvcfg: BitField::default(),
        senvcfg: BitField::default(),
        vstart: BitVector::default(),
        vxsat: BitVector::default(),
        vxrm: BitVector::default(),
        vl: BitVector::default(),
        vlenb: BitVector::default(),
        vtype: BitField::default(),
        pmpcfg_n: [BitField::default(); 64],
        pmpaddr_n: [BitVector::default(); 64],
        vr0: BitVector::default(),
        vr1: BitVector::default(),
        vr2: BitVector::default(),
        vr3: BitVector::default(),
        vr4: BitVector::default(),
        vr5: BitVector::default(),
        vr6: BitVector::default(),
        vr7: BitVector::default(),
        vr8: BitVector::default(),
        vr9: BitVector::default(),
        vr10: BitVector::default(),
        vr11: BitVector::default(),
        vr12: BitVector::default(),
        vr13: BitVector::default(),
        vr14: BitVector::default(),
        vr15: BitVector::default(),
        vr16: BitVector::default(),
        vr17: BitVector::default(),
        vr18: BitVector::default(),
        vr19: BitVector::default(),
        vr20: BitVector::default(),
        vr21: BitVector::default(),
        vr22: BitVector::default(),
        vr23: BitVector::default(),
        vr24: BitVector::default(),
        vr25: BitVector::default(),
        vr26: BitVector::default(),
        vr27: BitVector::default(),
        vr28: BitVector::default(),
        vr29: BitVector::default(),
        vr30: BitVector::default(),
        vr31: BitVector::default(),
        vcsr: BitField::default(),
        utvec: BitField::default(),
        uscratch: BitVector::default(),
        uepc: BitVector::default(),
        ucause: BitField::default(),
        utval: BitVector::default(),
        float_result: BitVector::default(),
        float_fflags: BitVector::default(),
        f0: BitVector::default(),
        f1: BitVector::default(),
        f2: BitVector::default(),
        f3: BitVector::default(),
        f4: BitVector::default(),
        f5: BitVector::default(),
        f6: BitVector::default(),
        f7: BitVector::default(),
        f8: BitVector::default(),
        f9: BitVector::default(),
        f10: BitVector::default(),
        f11: BitVector::default(),
        f12: BitVector::default(),
        f13: BitVector::default(),
        f14: BitVector::default(),
        f15: BitVector::default(),
        f16: BitVector::default(),
        f17: BitVector::default(),
        f18: BitVector::default(),
        f19: BitVector::default(),
        f20: BitVector::default(),
        f21: BitVector::default(),
        f22: BitVector::default(),
        f23: BitVector::default(),
        f24: BitVector::default(),
        f25: BitVector::default(),
        f26: BitVector::default(),
        f27: BitVector::default(),
        f28: BitVector::default(),
        f29: BitVector::default(),
        f30: BitVector::default(),
        f31: BitVector::default(),
        fcsr: BitField::default(),
        mtimecmp: BitVector::default(),
        htif_tohost: BitVector::default(),
        htif_done: false,
        htif_exit_code: BitVector::default(),
        htif_cmd_write: false,
        htif_payload_writes: BitVector::default(),
        tlb: None,
        satp: BitVector::default(),
    }
}

pub fn sail_to_miralis(sail_ctx: SailVirtCtx) -> VirtContext {
    let mut ctx = VirtContext::new(
        0,
        0,
        ExtensionsCapability {
            has_crypto_extension: true,
            has_sstc_extension: false,
            is_sstc_enabled: false,
            has_zicntr: true,
            has_h_extension: false,
            has_s_extension: false,
            has_v_extension: true,
        },
    );

    ctx.mode = match sail_ctx.cur_privilege {
        Privilege::User => Mode::U,
        Privilege::Supervisor => Mode::S,
        Privilege::Machine => Mode::M,
    };

    ctx.pc = sail_ctx.nextPC.bits() as usize;

    // Transfer hart id
    ctx.hart_id = sail_ctx.mhartid.bits() as usize;

    ctx.nb_pmp = 16; // Fixed for now

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
    ctx.csr.vxsat = sail_ctx.vxsat.bits() != 0;
    ctx.csr.vxrm = sail_ctx.vxrm.bits() as u8;
    ctx.csr.vcsr = sail_ctx.vcsr.bits.bits() as u8;
    ctx.csr.vl = sail_ctx.vl.bits() as usize;
    ctx.csr.vtype = sail_ctx.vtype.bits.bits() as usize;
    ctx.csr.vlenb = sail_ctx.vlenb.bits() as usize;

    ctx
}

fn pmpcfg_sail_to_miralis(cfgs: [BitField<8>; 64]) -> [usize; 8] {
    let mut output: [usize; 8] = [0; 8];

    for i in 0..64 {
        let idx = i / 8;
        let offset = i % 8;
        output[idx] |= ((cfgs[i].bits.bits() & 0xff) << (8 * offset)) as usize;
    }

    output
}

fn pmpaddr_sail_to_miralis(addresses: [BitVector<64>; 64]) -> [usize; 64] {
    let mut output: [usize; 64] = [0; 64];
    for i in 0..64 {
        output[i] = addresses[i].bits as usize;
    }

    output
}
