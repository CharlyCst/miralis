use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::arch::MCause::IllegalInstr;
use miralis::arch::{mie, write_pmp, MCause, Register};
use miralis::decoder::Instr;
use miralis::host::MiralisContext;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use miralis::virt::VirtContext;
use sail_decoder::encdec_backwards;
use sail_model::{
    ast, check_CSR, execute_HFENCE_GVMA, execute_HFENCE_VVMA, execute_MRET, execute_SFENCE_VMA,
    execute_SRET, execute_WFI, pmpCheck, readCSR, set_next_pc, step_interrupts_only, trap_handler,
    writeCSR, AccessType, ExceptionType, Privilege, SailVirtCtx,
};
use sail_prelude::{sys_pmp_count, BitField, BitVector};

use crate::adapters::{ast_to_miralis_instr, miralis_to_sail, sail_to_miralis};

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

    if !check_CSR(
        sail_virt_ctx,
        BitVector::new(csr),
        Privilege::Machine,
        is_write,
    ) {
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
        Instr::Csrrw { .. } | Instr::Csrrwi { .. } => {
            ((generate_csr_register_fancy(sail_virt_ctx, true) << 20) | (instr & 0xfffff) as u64)
                as usize
        }
        Instr::Csrrc { csr: _, rd: _, rs1 } | Instr::Csrrs { csr: _, rd: _, rs1 } => {
            ((generate_csr_register_fancy(sail_virt_ctx, rs1 != Register::X0) << 20)
                | (instr & 0xfffff) as u64) as usize
        }
        Instr::Csrrci {
            csr: _,
            rd: _,
            uimm,
        }
        | Instr::Csrrsi {
            csr: _,
            rd: _,
            uimm,
        } => {
            ((generate_csr_register_fancy(sail_virt_ctx, uimm != 0) << 20)
                | (instr & 0xfffff) as u64) as usize
        }
        _ => instr,
    };

    return instr;
}

fn generate_trap_cause() -> usize {
    let code = any!(usize) & 0xF;
    if MCause::new(code) == MCause::UnknownException {
        0
    } else {
        code
    }
}

fn fill_trap_info_structure(ctx: &mut VirtContext, cause: MCause) {
    let mut sail_ctx = miralis_to_sail(ctx);

    // Inject trap
    let pc_argument = sail_ctx.PC;
    trap_handler(
        &mut sail_ctx,
        Privilege::Machine,
        false,
        BitVector::new(cause as u64),
        pc_argument,
        None,
        None,
    );

    let new_miralis_ctx = sail_to_miralis(sail_ctx);

    ctx.trap_info.mcause = new_miralis_ctx.csr.mcause;
    ctx.trap_info.mstatus = new_miralis_ctx.csr.mstatus;
    ctx.trap_info.mtval = new_miralis_ctx.csr.mtval;
    ctx.trap_info.mepc = new_miralis_ctx.csr.mepc;
}

// #[cfg_attr(kani, kani::proof)]
// #[cfg_attr(test, test)]
pub fn verify_trap_logic() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    let trap_cause = generate_trap_cause();

    // Generate the trap handler
    fill_trap_info_structure(&mut ctx, MCause::new(trap_cause as usize));

    ctx.emulate_jump_trap_handler();

    {
        let pc = sail_ctx.PC;
        let new_pc = trap_handler(
            &mut sail_ctx,
            Privilege::Machine,
            false,
            BitVector::new(trap_cause as u64),
            pc,
            None,
            None,
        );
        set_next_pc(&mut sail_ctx, new_pc);
    }

    let mut sail_ctx_generated = adapters::sail_to_miralis(sail_ctx);

    sail_ctx_generated.is_wfi = ctx.is_wfi.clone();
    sail_ctx_generated.trap_info = ctx.trap_info.clone();

    assert_eq!(
        sail_ctx_generated, ctx,
        "Injection of trap doesn't work properly"
    );
}

// #[cfg_attr(kani, kani::proof)]
// #[cfg_attr(test, test)]
pub fn formally_verify_emulation_privileged_instructions() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // We don't delegate any interrupts in the formal verification
    sail_ctx.mideleg = BitField::new(0);
    ctx.csr.mideleg = 0;

    // Generate the trap handler
    fill_trap_info_structure(&mut ctx, IllegalInstr);

    let instr = generate_raw_instruction(&mut mctx, &mut sail_ctx);

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

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_decoder() {
    let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = (any!(u32) & !0b1111111) | 0b1110011;

    // Decode values
    let decoded_value_sail = ast_to_miralis_instr(encdec_backwards(
        &mut sail_ctx,
        BitVector::new(instr as u64),
    ));
    let decoded_value_miralis = mctx.decode_illegal_instruction(instr as usize);

    // We verify the equivalence with the following decomposition
    // A <--> B <==> A --> B && B --> A

    // For the moment, we ignore the values that are not decoded by the sail reference
    if decoded_value_sail != Instr::Unknown {
        assert_eq!(
            decoded_value_sail, decoded_value_miralis,
            "decoders are not equivalent"
        );
    }

    if decoded_value_miralis != Instr::Unknown {
        assert_eq!(
            decoded_value_sail, decoded_value_sail,
            "decoders are not equivalent"
        )
    }
}
