use crate::model::{execute_MRET, execute_SFENCE_VMA, execute_WFI, sail_decoder_illegal};
use softcore_rv64::prelude::{bv, BitVector};
use softcore_rv64::raw;
use softcore_rv64::raw::ast;
use softcore_rv64::Core;

pub fn execute_ast(core_ctx: &mut Core, instr: usize) {
    match sail_decoder_illegal::encdec_backwards(core_ctx, BitVector::new(instr as u64)) {
        ast::MRET(_) => {
            execute_MRET(core_ctx);
        }
        ast::WFI(()) => {
            execute_WFI(core_ctx);
        }
        ast::SFENCE_VMA((rs1, rs2)) => {
            execute_SFENCE_VMA(core_ctx, rs1, rs2);
        }
        ast::CSRReg((csr, rs1, rd, op)) => {
            let rs1_val = raw::rX(core_ctx, raw::regidx_to_regno(rs1));
            raw::doCSR(
                core_ctx,
                csr,
                rs1_val,
                rd,
                op,
                (op == raw::csrop::CSRRW) | (rs1 != raw::zreg),
            );
        }
        ast::CSRImm((csr, imm, rd, op)) => {
            raw::doCSR(
                core_ctx,
                csr,
                imm.zero_extend(),
                rd,
                op,
                (op == raw::csrop::CSRRW) | (imm != bv(0)),
            );
        }
        _ => {}
    }
}
