use miralis::arch::Register;
use miralis::debug;
use miralis::decoder::Instr;
use miralis::host::MiralisContext;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter, RegisterContextSetter};
use sail_decoder::encdec_backwards;
use sail_model::{ast, check_CSR, Privilege, SailVirtCtx};
use sail_prelude::{BitVector};

use crate::adapters::{ast_to_miralis_instr, decode_csr_register, sail_to_miralis};

#[macro_use]
mod symbolic;
mod adapters;
mod execute;

pub fn generate_csr_register(sail_virt_ctx: &mut SailVirtCtx, isWrite: bool) -> u64 {
    // We want only 12 bits
    let mut csr: u64 = any!(u64) & 0xFFF;

    // Ignore sedeleg and sideleg
    if csr == 0b000100000010 || csr == 0b000100000011 {
        csr = 0x341;
    }

    // Odd pmpcfg indices configs are not allowed
    if 0x3A0 <= csr && csr <= 0x3AF {
        csr &= !0b1;
    }

    if !check_CSR(sail_virt_ctx, BitVector::new(csr), Privilege::Machine, isWrite) {
        csr = 0x341;
    }

    if 0x303 == csr {
        csr = 0x341;
    }


    return 0x7A0;

    return csr;
}


fn generate_raw_instruction(mctx: &mut MiralisContext, sail_virt_ctx: &mut SailVirtCtx) -> usize {
    const SYSTEM_MASK: u32 = 0b1110011;
    const DEFAULT_INSTRUCTION: usize = 0b00110000001000000000000001110011;

    // Generate instruction to decode and emulate
    let mut instr: usize = ((any!(u32) & !0b1111111) | SYSTEM_MASK) as usize;

    instr = match mctx.decode_illegal_instruction(instr) {
        /*Instr::Csrrw {.. } | Instr::Csrrwi {  ..} => {
            ((generate_csr_register(sail_virt_ctx, true) << 20) | (instr & 0xfffff) as u64) as usize
        }*/
        /*Instr::Csrrc { csr, rd, rs1 } | Instr::Csrrs { csr, rd, rs1 }=> {
            ((generate_csr_register(sail_virt_ctx, rs1 != Register::X0) << 20) | (instr & 0xfffff) as u64) as usize
        }*/
        /*Instr::Csrrci { csr, rd, uimm } |*/  Instr::Csrrsi { csr, rd, uimm } => {
            ((generate_csr_register(sail_virt_ctx, uimm != 0) << 20) | (instr & 0xfffff) as u64) as usize
        }
        _ => {0b00110000001000000000000001110011}
    };


    return instr;
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn formally_verify_emulation_privileged_instructions() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    let mut instr = generate_raw_instruction(&mut mctx, &mut sail_ctx);

    // Decode the instructions
    let decoded_instruction = mctx.decode_illegal_instruction(instr);
    let decoded_sail_instruction = encdec_backwards(&mut sail_ctx, BitVector::new(instr as u64));

    let is_unknown_sail = decoded_sail_instruction == ast::ILLEGAL(BitVector::new(0));
    let is_unknown_miralis = decoded_instruction == Instr::Unknown;

    assert_eq!(is_unknown_sail, is_unknown_miralis, "Both decoder don't decode the same instruction set");

    if !is_unknown_miralis {
        assert_eq!(decoded_instruction, ast_to_miralis_instr(decoded_sail_instruction), "instruction are decoded not similar");

        // Emulate instruction in Miralis
        ctx.emulate_illegal_instruction(&mut mctx, instr);

        // Execute value in sail
        execute::execute_ast(&mut sail_ctx, instr);

        assert_eq!(sail_to_miralis(sail_ctx).csr.tselect, ctx.csr.tselect, "tselect");

        /*assert_eq!(sail_to_miralis(sail_ctx).csr.misa, ctx.csr.misa, "misa");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mie, ctx.csr.mie, "mie");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mip, ctx.csr.mip, "mip");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mtvec, ctx.csr.mtvec, "mtvec");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mvendorid, ctx.csr.mvendorid, "mvendorid");
        assert_eq!(sail_to_miralis(sail_ctx).csr.marchid, ctx.csr.marchid, "marchid");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mimpid, ctx.csr.mimpid, "mimpid");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mcycle, ctx.csr.mcycle, "mcycle");
        assert_eq!(sail_to_miralis(sail_ctx).csr.minstret, ctx.csr.minstret, "minstret");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mscratch, ctx.csr.mscratch, "mscratch");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mcountinhibit, ctx.csr.mcountinhibit, "mcountinhibit");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mcounteren, ctx.csr.mcounteren, "mcounteren");
        assert_eq!(sail_to_miralis(sail_ctx).csr.menvcfg, ctx.csr.menvcfg, "menvcfg");
        assert_eq!(
            sail_to_miralis(sail_ctx).csr.mseccfg,
            ctx.csr.mseccfg,
            "mseccfg"
        );
        assert_eq!(sail_to_miralis(sail_ctx).csr.mcause, ctx.csr.mcause, "mcause");
        assert_eq!(sail_to_miralis(sail_ctx).csr.tselect, ctx.csr.tselect, "tselect");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mepc, ctx.csr.mepc, "mepc");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mtval, ctx.csr.mtval, "mtval");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mtval2, ctx.csr.mtval2, "mtval2");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mstatus, ctx.csr.mstatus, "mstatus");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mtinst, ctx.csr.mtinst, "mtinst");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mconfigptr, ctx.csr.mconfigptr, "mconfigptr");
        assert_eq!(sail_to_miralis(sail_ctx).csr.stvec, ctx.csr.stvec, "stvec");
        assert_eq!(sail_to_miralis(sail_ctx).csr.scounteren, ctx.csr.scounteren, "scounteren");
        assert_eq!(sail_to_miralis(sail_ctx).csr.senvcfg, ctx.csr.senvcfg, "senvcfg");
        assert_eq!(sail_to_miralis(sail_ctx).csr.sscratch, ctx.csr.sscratch, "sscratch");
        assert_eq!(sail_to_miralis(sail_ctx).csr.sepc, ctx.csr.sepc, "sepc");
        assert_eq!(sail_to_miralis(sail_ctx).csr.scause, ctx.csr.scause, "scause");
        assert_eq!(sail_to_miralis(sail_ctx).csr.stval, ctx.csr.stval, "stval");
        assert_eq!(sail_to_miralis(sail_ctx).csr.satp, ctx.csr.satp, "satp");
        assert_eq!(sail_to_miralis(sail_ctx).csr.scontext, ctx.csr.scontext, "scontext");
        assert_eq!(sail_to_miralis(sail_ctx).csr.stimecmp, ctx.csr.stimecmp, "stimecmp");
        assert_eq!(sail_to_miralis(sail_ctx).csr.medeleg, ctx.csr.medeleg, "medeleg");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mideleg, ctx.csr.mideleg, "mideleg");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hstatus, ctx.csr.hstatus, "hstatus");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hedeleg, ctx.csr.hedeleg, "hedeleg");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hideleg, ctx.csr.hideleg, "hideleg");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hvip, ctx.csr.hvip, "hvip");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hip, ctx.csr.hip, "hip");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hie, ctx.csr.hie, "hie");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hgeip, ctx.csr.hgeip, "hgeip");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hgeie, ctx.csr.hgeie, "hgeie");
        assert_eq!(sail_to_miralis(sail_ctx).csr.henvcfg, ctx.csr.henvcfg, "henvcfg");
        assert_eq!(sail_to_miralis(sail_ctx).csr.henvcfgh, ctx.csr.henvcfgh, "henvcfgh");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hcounteren, ctx.csr.hcounteren, "hcounteren");
        assert_eq!(sail_to_miralis(sail_ctx).csr.htimedelta, ctx.csr.htimedelta, "htimedelta");
        assert_eq!(sail_to_miralis(sail_ctx).csr.htimedeltah, ctx.csr.htimedeltah, "htimedeltah");
        assert_eq!(sail_to_miralis(sail_ctx).csr.htval, ctx.csr.htval, "htval");
        assert_eq!(sail_to_miralis(sail_ctx).csr.htinst, ctx.csr.htinst, "htinst");
        assert_eq!(sail_to_miralis(sail_ctx).csr.hgatp, ctx.csr.hgatp, "hgatp");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vsstatus, ctx.csr.vsstatus, "vsstatus");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vsie, ctx.csr.vsie, "vsie");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vstvec, ctx.csr.vstvec, "vstvec");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vsscratch, ctx.csr.vsscratch, "vsscratch");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vsepc, ctx.csr.vsepc, "vsepc");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vscause, ctx.csr.vscause, "vscause");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vstval, ctx.csr.vstval, "vstval");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vsip, ctx.csr.vsip, "vsip");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vsatp, ctx.csr.vsatp, "vsatp");
        assert_eq!(sail_to_miralis(sail_ctx).csr.pmpcfg, ctx.csr.pmpcfg, "pmpcfg");
        assert_eq!(sail_to_miralis(sail_ctx).csr.pmpaddr, ctx.csr.pmpaddr, "pmpaddr");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mhpmcounter, ctx.csr.mhpmcounter, "mhpmcounter");
        assert_eq!(sail_to_miralis(sail_ctx).csr.mhpmevent, ctx.csr.mhpmevent, "mhpmevent");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vstart, ctx.csr.vstart, "vstart");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vxsat, ctx.csr.vxsat, "vxsat");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vxrm, ctx.csr.vxrm, "vxrm");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vcsr, ctx.csr.vcsr, "vcsr");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vl, ctx.csr.vl, "vl");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vtype, ctx.csr.vtype, "vtype");
        assert_eq!(sail_to_miralis(sail_ctx).csr.vlenb, ctx.csr.vlenb, "vlenb");*/


    }
}
