use miralis::arch::{MCause, mie, Register, write_pmp};
use miralis::arch::MCause::IllegalInstr;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::decoder::Instr;
use miralis::host::MiralisContext;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use miralis::virt::VirtContext;
use sail_decoder::encdec_backwards;
use sail_model::{AccessType, ast, check_CSR, ExceptionType, execute_HFENCE_GVMA, execute_HFENCE_VVMA, execute_MRET, execute_SFENCE_VMA, execute_SRET, execute_WFI, pmpCheck, Privilege, readCSR, SailVirtCtx, step_interrupts_only, trap_handler, writeCSR};
use sail_prelude::{BitField, BitVector, sys_pmp_count};
use crate::adapters::{miralis_to_sail, sail_to_miralis};

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


fn fill_trap_info_structure(ctx: &mut VirtContext, cause: MCause) {

    let mut sail_ctx = miralis_to_sail(ctx);

    // Inject trap
    // TODO: adapt cause
    let pc_argument = sail_ctx.PC;
    trap_handler(&mut sail_ctx, Privilege::Machine, false, BitVector::new(cause as u64), pc_argument, None, None);

    let new_miralis_ctx =  sail_to_miralis(sail_ctx);

    // TODO: Add
    ctx.trap_info.mcause = new_miralis_ctx.csr.mcause;
    ctx.trap_info.mstatus = new_miralis_ctx.csr.mstatus;
    ctx.trap_info.mtval  = new_miralis_ctx.csr.mtval;
    ctx.trap_info.mepc = new_miralis_ctx.csr.mepc;
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn formally_verify_emulation_privileged_instructions() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // We don't delegate any interrupts in the formal verification
    sail_ctx.mideleg = BitField::new(0);
    ctx.csr.mideleg = 0;

    // Generate the trap handler
    fill_trap_info_structure(&mut ctx, IllegalInstr);

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

        // These fields are used only in the miralis context and are irrelevant
        sail_ctx_generated.is_wfi = ctx.is_wfi.clone();
        sail_ctx_generated.trap_info = ctx.trap_info.clone();

        assert_eq!(sail_ctx_generated, ctx, "Overall");
    }
}
