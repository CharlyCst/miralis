use miralis::arch::{mie, Register, write_pmp};
use miralis::arch::MCause::IllegalInstr;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::decoder::Instr;
use miralis::host::MiralisContext;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use sail_decoder::encdec_backwards;
use sail_model::{AccessType, ast, check_CSR, ExceptionType, execute_HFENCE_GVMA, execute_HFENCE_VVMA, execute_MRET, execute_SFENCE_VMA, execute_SRET, execute_WFI, pmpCheck, Privilege, readCSR, SailVirtCtx, step_interrupts_only, writeCSR};
use sail_prelude::{BitField, BitVector, sys_pmp_count};
use crate::adapters::sail_to_miralis;

#[macro_use]
mod symbolic;
mod adapters;
mod execute;


pub fn generate_csr_register_fancy(sail_virt_ctx: &mut SailVirtCtx, is_write: bool) -> u64 {
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

    if !check_CSR(sail_virt_ctx, BitVector::new(csr), Privilege::Machine, is_write) {
        csr = 0x341;
    }

    if 0x303 == csr {
        csr = 0x341;
    }

    return csr;
}

fn generate_raw_instruction(mctx: &mut MiralisContext, sail_virt_ctx: &mut SailVirtCtx) -> usize {
    const SYSTEM_MASK: u32 = 0b1110011;

    // Generate instruction to decode and emulate
    let mut instr: usize = ((any!(u32) & !0b1111111) | SYSTEM_MASK) as usize;

    // For the moment, we simply avoid the csr with illegal instructions, I will handle it in a second case
    instr = match mctx.decode_illegal_instruction(instr) {
        Instr::Csrrw {.. } | Instr::Csrrwi {  ..} => {
            ((generate_csr_register_fancy(sail_virt_ctx, true) << 20) | (instr & 0xfffff) as u64) as usize
        }
        Instr::Csrrc { csr: _, rd: _, rs1 } | Instr::Csrrs { csr: _, rd: _, rs1 }=> {
            ((generate_csr_register_fancy(sail_virt_ctx, rs1 != Register::X0) << 20) | (instr & 0xfffff) as u64) as usize
        }
        Instr::Csrrci { csr: _, rd: _, uimm } |  Instr::Csrrsi { csr: _, rd: _, uimm } => {
            ((generate_csr_register_fancy(sail_virt_ctx, uimm != 0) << 20) | (instr & 0xfffff) as u64) as usize
        }
        _ => {instr}
    };

    return instr;
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn formally_verify_emulation_privileged_instructions() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // In this scenario, we are facing illegal instructions
    ctx.trap_info.mcause = IllegalInstr as usize;

    // We don't delegate any interrupts in the formal verification
    sail_ctx.mideleg = BitField::new(0);
    ctx.csr.mideleg = 0;

    // Create the precondition for the trap handler
    ctx.trap_info.mcause = IllegalInstr as usize;
    ctx.trap_info.mepc = ctx.pc;

    ctx.trap_info.mstatus = ctx.trap_info.mstatus & !((1 << 7) - 1) | ((ctx.trap_info.mstatus & 0b1000) << 4);
    ctx.trap_info.mstatus &= !(1<<3);


    // let instr = generate_raw_instruction(&mut mctx, &mut sail_ctx);

    let instr = 0x00001073;
    // Decode the instructions
    let decoded_instruction = mctx.decode_illegal_instruction(instr);
    let decoded_sail_instruction = encdec_backwards(&mut sail_ctx, BitVector::new(instr as u64));

    let is_unknown_sail = decoded_sail_instruction == ast::ILLEGAL(BitVector::new(0));
    let is_unknown_miralis = decoded_instruction == Instr::Unknown;

    // assert_eq!(is_unknown_sail, is_unknown_miralis, "Both decoder don't decode the same instruction set");

    if !is_unknown_miralis {
        // assert_eq!(decoded_instruction, adapters::ast_to_miralis_instr(decoded_sail_instruction), "instruction are decoded not similar");

        // Emulate instruction in Miralis
        ctx.emulate_illegal_instruction(&mut mctx, instr);

        // Execute value in sail
        execute::execute_ast(&mut sail_ctx, instr);

        let mut sail_ctx_generated = adapters::sail_to_miralis(sail_ctx);

        // We don't care about this field
        sail_ctx_generated.is_wfi = true;
        ctx.is_wfi = true;

        // assert_eq!(sail_ctx_generated, ctx, "equivalence");
        // assert_eq!(sail_ctx_generated.csr, ctx.csr, "mcause");
        assert_eq!(sail_ctx_generated.csr.mstatus, ctx.csr.mstatus, "mstatus");

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
