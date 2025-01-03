use sail_decoder::encdec_backwards;
use sail_model::{ast, SailVirtCtx};
use sail_prelude::BitVector;

pub fn execute_ast(sail_virt_ctx: &mut SailVirtCtx, instr: usize) {
    match encdec_backwards(sail_virt_ctx, BitVector::new(instr as u64)) {
        ast::MRET(_) => {
            sail_model::execute_MRET(sail_virt_ctx);
        }
        ast::WFI(()) => {
            sail_model::execute_WFI(sail_virt_ctx);
        }
        ast::SFENCE_VMA((_rs1, _rs2)) => {
            // TODO: Implement this part
        }
        ast::CSR((csr, rs1, rd, is_imm, op)) => {
            sail_model::execute_CSR(sail_virt_ctx, csr, rs1, rd, is_imm, op);
        }
        _ => {}
    }
}
