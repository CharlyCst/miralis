use sail_decoder::encdec_backwards;
use sail_model::{ast, SailVirtCtx};
use sail_prelude::BitVector;

pub fn execute_ast(sail_virt_ctx: &mut SailVirtCtx, instr: usize) {
    match encdec_backwards(sail_virt_ctx, BitVector::new(instr as u64)) {
        ast::MRET(_) => {
            sail_model::execute_MRET(sail_virt_ctx);
        }
        ast::SRET(()) => {
            sail_model::execute_SRET(sail_virt_ctx);
        }
        ast::WFI(()) => {
            sail_model::execute_WFI(sail_virt_ctx);
            sail_virt_ctx.nextPC = BitVector::new(sail_virt_ctx.nextPC.bits.wrapping_add(4));
        }
        ast::SFENCE_VMA((rs1, rs2)) => {
            sail_model::execute_SFENCE_VMA(sail_virt_ctx, rs1, rs2);
        }
        ast::HFENCE_VVMA((rs1, rs2)) => {
            sail_model::execute_HFENCE_VVMA(sail_virt_ctx, rs1, rs2);
        }
        ast::HFENCE_GVMA((rs1, rs2)) => {
            sail_model::execute_HFENCE_GVMA(sail_virt_ctx, rs1, rs2);
        }
        ast::CSR((csr, rs1, rd, is_imm, op)) => {
            sail_model::execute_CSR(sail_virt_ctx, csr, rs1, rd, is_imm, op);
            sail_virt_ctx.nextPC = BitVector::new(sail_virt_ctx.nextPC.bits.wrapping_add(4));
        }
        _ => {}
    }
}
