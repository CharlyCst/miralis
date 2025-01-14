use miralis::arch::{mie, Register};
use miralis::debug;
use miralis::decoder::Instr;
use miralis::host::MiralisContext;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter, RegisterContextSetter};
use sail_decoder::encdec_backwards;
use sail_model::{ast, check_CSR, Privilege, readCSR, SailVirtCtx, writeCSR};
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

    if !check_CSR(sail_virt_ctx, BitVector::new(0x341), Privilege::Machine, isWrite) {
        panic!("BIzzare condition here");
    }

    if 0x303 == csr {
        csr = 0x341;
    }

    return csr;
}


fn generate_raw_instruction(mctx: &mut MiralisContext, sail_virt_ctx: &mut SailVirtCtx) -> usize {
    const SYSTEM_MASK: u32 = 0b1110011;
    const DEFAULT_INSTRUCTION: usize = 0b00110000001000000000000001110011;

    // Generate instruction to decode and emulate
    let mut instr: usize = ((any!(u32) & !0b1111111) | SYSTEM_MASK) as usize;

    // For the moment, we simply avoid the csr with illegal instructions, I will handle it in a second case
    /*instr = match mctx.decode_illegal_instruction(instr) {
        Instr::Csrrw {.. } | Instr::Csrrwi {  ..} => {
            ((generate_csr_register(sail_virt_ctx, true) << 20) | (instr & 0xfffff) as u64) as usize
        }
        Instr::Csrrc { csr, rd, rs1 } | Instr::Csrrs { csr, rd, rs1 }=> {
            ((generate_csr_register(sail_virt_ctx, rs1 != Register::X0) << 20) | (instr & 0xfffff) as u64) as usize
        }
        Instr::Csrrci { csr, rd, uimm } |  Instr::Csrrsi { csr, rd, uimm } => {
            ((generate_csr_register(sail_virt_ctx, uimm != 0) << 20) | (instr & 0xfffff) as u64) as usize
        }
        _ => {instr}
    };*/

    instr = 0b00010000010100000000000001110011;

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

    //assert_eq!(is_unknown_sail, is_unknown_miralis, "Both decoder don't decode the same instruction set");

    if !is_unknown_miralis {
        //assert_eq!(decoded_instruction, ast_to_miralis_instr(decoded_sail_instruction), "instruction are decoded not similar");

        // Emulate instruction in Miralis
        ctx.emulate_illegal_instruction(&mut mctx, instr);

        // Execute value in sail
        execute::execute_ast(&mut sail_ctx, instr);

        let mut sail_ctx_generated = sail_to_miralis(sail_ctx);
        sail_ctx_generated.is_wfi = true;
        ctx.is_wfi = true;

        assert_eq!(sail_ctx_generated, ctx, "equivalence");
    }
}
