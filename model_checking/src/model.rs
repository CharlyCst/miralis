//! RV64 model
//!
//! This file contains automatically translated fragments of the RISC-V 64 Sail specification.
//! We are currently in the process of migrating to softcore-rv64, i.e. a new version of the model
//! that we maintain as an external library. The new model is not yet complete, therefore we must
//! keep some extensions locally in this file until we can rely selely on the upsteam model.

#![allow(warnings)]

use softcore_rv64::prelude::*;
use softcore_rv64::raw::*;

pub fn execute_MRET(core_ctx: &mut Core) -> ExecutionResult {
    if { (core_ctx.cur_privilege != Privilege::Machine) } {
        ExecutionResult::Illegal_Instruction(())
    } else if { !(true) } {
        ExecutionResult::Ext_XRET_Priv_Failure(())
    } else {
        {
            let var_32 = {
                let var_33 = core_ctx.cur_privilege;
                let var_34 = core_ctx.PC;
                exception_handler(core_ctx, var_33, ctl_result::CTL_MRET(()), var_34)
            };
            set_next_pc(core_ctx, var_32)
        };
        RETIRE_SUCCESS
    }
}

pub fn execute_SRET(core_ctx: &mut Core) -> ExecutionResult {
    let sret_illegal: bool = match core_ctx.cur_privilege {
        Privilege::User => true,
        Privilege::Supervisor => {
            (!(currentlyEnabled(core_ctx, extension::Ext_S))
                || ({
                    let var_38 = core_ctx.mstatus;
                    _get_Mstatus_TSR(var_38)
                } == BitVector::<1>::new(0b1)))
        }
        Privilege::Machine => !(currentlyEnabled(core_ctx, extension::Ext_S)),
        _ => {
            panic!("Unreachable code")
        }
    };
    if { sret_illegal } {
        ExecutionResult::Illegal_Instruction(())
    } else if { !(true) } {
        ExecutionResult::Ext_XRET_Priv_Failure(())
    } else {
        {
            let var_35 = {
                let var_36 = core_ctx.cur_privilege;
                let var_37 = core_ctx.PC;
                exception_handler(core_ctx, var_36, ctl_result::CTL_SRET(()), var_37)
            };
            set_next_pc(core_ctx, var_35)
        };
        RETIRE_SUCCESS
    }
}

pub fn execute_WFI(core_ctx: &mut Core) -> ExecutionResult {
    match core_ctx.cur_privilege {
        Privilege::Machine => ExecutionResult::Wait_For_Interrupt(()),
        Privilege::Supervisor => {
            if {
                ({
                    let var_40 = core_ctx.mstatus;
                    _get_Mstatus_TW(var_40)
                } == BitVector::<1>::new(0b1))
            } {
                ExecutionResult::Illegal_Instruction(())
            } else {
                ExecutionResult::Wait_For_Interrupt(())
            }
        }
        Privilege::User => ExecutionResult::Illegal_Instruction(()),
        _ => {
            panic!("Unreachable code")
        }
    }
}

pub fn execute_SFENCE_VMA(core_ctx: &mut Core, rs1: regidx, rs2: regidx) -> ExecutionResult {
    let addr = if { (rs1 != zreg) } {
        Some(rX_bits(core_ctx, rs1))
    } else {
        None
    };
    let asid = if { (rs2 != zreg) } {
        Some(subrange_bits(rX_bits(core_ctx, rs2), 15, 0))
    } else {
        None
    };
    match core_ctx.cur_privilege {
        Privilege::User => ExecutionResult::Illegal_Instruction(()),
        Privilege::Supervisor => {
            match {
                let var_41 = core_ctx.mstatus;
                _get_Mstatus_TVM(var_41)
            } {
                b__0 if { (b__0 == BitVector::<1>::new(0b1)) } => {
                    ExecutionResult::Illegal_Instruction(())
                }
                _ => {
                    flush_TLB(asid, addr);
                    RETIRE_SUCCESS
                }
                _ => {
                    panic!("Unreachable code")
                }
            }
        }
        Privilege::Machine => {
            flush_TLB(asid, addr);
            RETIRE_SUCCESS
        }
        _ => {
            panic!("Unreachable code")
        }
    }
}

/// exception_handler
///
/// Generated from the Sail sources at `riscv_sys_control.sail` L277-321.
pub fn exception_handler(
    core_ctx: &mut Core,
    cur_priv: Privilege,
    ctl: ctl_result,
    pc: BitVector<64>,
) -> BitVector<{ 64 }> {
    match (cur_priv, ctl) {
        (_, ctl_result::CTL_TRAP(e)) => {
            let del_priv = exception_delegatee(core_ctx, e.trap, cur_priv);
            trap_handler(
                core_ctx,
                del_priv,
                false,
                exceptionType_to_bits(e.trap),
                pc,
                e.excinfo,
                e.ext,
            )
        }
        (_, ctl_result::CTL_MRET(())) => {
            let prev_priv = core_ctx.cur_privilege;
            core_ctx.mstatus.bits = {
                let var_1 = {
                    let var_2 = core_ctx.mstatus;
                    _get_Mstatus_MPIE(var_2)
                };
                core_ctx.mstatus.bits.set_subrange::<3, 4, 1>(var_1)
            };
            core_ctx.mstatus.bits = core_ctx
                .mstatus
                .bits
                .set_subrange::<7, 8, 1>(BitVector::<1>::new(0b1));
            core_ctx.cur_privilege = {
                let var_3 = {
                    let var_4 = core_ctx.mstatus;
                    _get_Mstatus_MPP(var_4)
                };
                privLevel_of_bits(var_3)
            };
            core_ctx.mstatus.bits = {
                let var_5 = {
                    let var_6 = if { currentlyEnabled(core_ctx, extension::Ext_U) } {
                        Privilege::User
                    } else {
                        Privilege::Machine
                    };
                    privLevel_to_bits(var_6)
                };
                core_ctx.mstatus.bits.set_subrange::<11, 13, 2>(var_5)
            };
            if { (core_ctx.cur_privilege != Privilege::Machine) } {
                core_ctx.mstatus.bits = core_ctx
                    .mstatus
                    .bits
                    .set_subrange::<17, 18, 1>(BitVector::<1>::new(0b0))
            } else {
                ()
            };
            prepare_xret_target(core_ctx, Privilege::Machine)
        }
        (_, ctl_result::CTL_SRET(())) => {
            let prev_priv = core_ctx.cur_privilege;
            core_ctx.mstatus.bits = {
                let var_7 = {
                    let var_8 = core_ctx.mstatus;
                    _get_Mstatus_SPIE(var_8)
                };
                core_ctx.mstatus.bits.set_subrange::<1, 2, 1>(var_7)
            };
            core_ctx.mstatus.bits = core_ctx
                .mstatus
                .bits
                .set_subrange::<5, 6, 1>(BitVector::<1>::new(0b1));
            core_ctx.cur_privilege = if {
                ({
                    let var_9 = core_ctx.mstatus;
                    _get_Mstatus_SPP(var_9)
                } == BitVector::<1>::new(0b1))
            } {
                Privilege::Supervisor
            } else {
                Privilege::User
            };
            core_ctx.mstatus.bits = core_ctx
                .mstatus
                .bits
                .set_subrange::<8, 9, 1>(BitVector::<1>::new(0b0));
            if { (core_ctx.cur_privilege != Privilege::Machine) } {
                core_ctx.mstatus.bits = core_ctx
                    .mstatus
                    .bits
                    .set_subrange::<17, 18, 1>(BitVector::<1>::new(0b0))
            } else {
                ()
            };
            prepare_xret_target(core_ctx, Privilege::Supervisor)
        }
        _ => {
            panic!("Unreachable code")
        }
    }
}

/// set_next_pc
///
/// Generated from the Sail sources at `riscv_pc_access.sail` L24-27.
pub fn set_next_pc(core_ctx: &mut Core, pc: BitVector<64>) {
    core_ctx.nextPC = pc
}

/// rX_bits
///
/// Generated from the Sail sources at `riscv_regs.sail` L128.
pub fn rX_bits(core_ctx: &mut Core, i: regidx) -> BitVector<{ 64 }> {
    rX(core_ctx, regidx_to_regno(i))
}

/// flush_TLB
///
/// Generated from the Sail sources at `riscv_vmem_tlb.sail` L165-173.
pub fn flush_TLB(asid: Option<BitVector<16>>, addr: Option<BitVector<{ 64 }>>) {
    // todo!("E_for")
}

/// privLevel_of_bits
///
/// Generated from the Sail sources at `riscv_types.sail` L73.
pub fn privLevel_of_bits(b: BitVector<2>) -> Privilege {
    privLevel_bits_backwards(b)
}

/// privLevel_bits_backwards
///
/// Generated from the Sail sources.
pub fn privLevel_bits_backwards(arg_hashtag_: BitVector<2>) -> Privilege {
    match arg_hashtag_ {
        b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => Privilege::User,
        b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => Privilege::Supervisor,
        b__2 if { (b__2 == BitVector::<2>::new(0b11)) } => Privilege::Machine,
        _ => {
            panic!(
                "{}, l {}: {}",
                "riscv_types.sail",
                69,
                format!(
                    "{}{}",
                    "Invalid privilege level: ",
                    bits_str(BitVector::<2>::new(0b10))
                )
            )
        }
        _ => {
            panic!("Unreachable code")
        }
    }
}

/// prepare_xret_target
///
/// Generated from the Sail sources at `riscv_sys_exceptions.sail` L60-61.
pub fn prepare_xret_target(core_ctx: &mut Core, p: Privilege) -> BitVector<{ 64 }> {
    get_xepc(core_ctx, p)
}

pub mod sail_decoder_illegal {
    use super::*;

    pub fn bool_bits_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<1>) -> bool {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<1>::new(0b1)) } => true,
            _ => false,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn bool_bits_backwards_matches(sail_ctx: &mut Core, arg_hashtag_: BitVector<1>) -> bool {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<1>::new(0b1)) } => true,
            b__1 if { (b__1 == BitVector::<1>::new(0b0)) } => true,
            _ => false,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn encdec_csrop_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<2>) -> csrop {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<2>::new(0b01)) } => csrop::CSRRW,
            b__1 if { (b__1 == BitVector::<2>::new(0b10)) } => csrop::CSRRS,
            b__2 if { (b__2 == BitVector::<2>::new(0b11)) } => csrop::CSRRC,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn encdec_csrop_backwards_matches(sail_ctx: &mut Core, arg_hashtag_: BitVector<2>) -> bool {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<2>::new(0b01)) } => true,
            b__1 if { (b__1 == BitVector::<2>::new(0b10)) } => true,
            b__2 if { (b__2 == BitVector::<2>::new(0b11)) } => true,
            _ => false,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn encdec_backwards(core_ctx: &mut Core, arg_hashtag_: BitVector<32>) -> ast {
        let head_exp_hashtag_ = arg_hashtag_;
        match match head_exp_hashtag_ {
            v__40
                if {
                    {
                        let mapping1_hashtag_: BitVector<2> = v__40.subrange::<12, 14, 2>();
                        let mapping0_hashtag_: BitVector<1> = v__40.subrange::<14, 15, 1>();
                        (bool_bits_backwards_matches(core_ctx, mapping0_hashtag_)
                            && encdec_csrop_backwards_matches(core_ctx, mapping1_hashtag_));
                        (mapping1_hashtag_.bits() != 0
                            && (v__40.subrange::<0, 7, 7>() == BitVector::<7>::new(0b1110011)))
                    }
                } =>
            {
                let csr: BitVector<12> = v__40.subrange::<20, 32, 12>();
                let rs1: BitVector<5> = v__40.subrange::<15, 20, 5>();
                let rd: BitVector<5> = v__40.subrange::<7, 12, 5>();
                let mapping1_hashtag_: BitVector<2> = v__40.subrange::<12, 14, 2>();
                let mapping0_hashtag_: BitVector<1> = v__40.subrange::<14, 15, 1>();
                let csr: BitVector<12> = v__40.subrange::<20, 32, 12>();
                match (
                    bool_bits_backwards(core_ctx, mapping0_hashtag_),
                    encdec_csrop_backwards(core_ctx, mapping1_hashtag_),
                ) {
                    (true, op) => Some(ast::CSRImm((csr, rs1, encdec_reg_backwards(rd), op))),
                    (false, op) => Some(ast::CSRReg((
                        csr,
                        encdec_reg_backwards(rs1),
                        encdec_reg_backwards(rd),
                        op,
                    ))),
                    _ => None,
                    _ => {
                        panic!("Unreachable code")
                    }
                }
            }
            _ => None,
            _ => {
                panic!("Unreachable code")
            }
        } {
            Some(result) => result,
            None => match head_exp_hashtag_ {
                v__0 if { (v__0 == BitVector::<32>::new(0b00110000001000000000000001110011)) } => {
                    ast::MRET(())
                }
                v__7 if { (v__7 == BitVector::<32>::new(0b00010000010100000000000001110011)) } => {
                    ast::WFI(())
                }
                v__25
                    if {
                        ((v__25.subrange::<25, 32, 7>() == BitVector::<7>::new(0b0001001))
                            && (v__25.subrange::<0, 15, 15>()
                                == BitVector::<15>::new(0b000000001110011)))
                    } =>
                {
                    let rs2: BitVector<5> = v__25.subrange::<20, 25, 5>();
                    let rs1: BitVector<5> = v__25.subrange::<15, 20, 5>();
                    ast::SFENCE_VMA((encdec_reg_backwards(rs1), encdec_reg_backwards(rs2)))
                }
                _ => ast::ILLEGAL(BitVector::new(0)),
            },
            _ => {
                panic!("Unreachable code")
            }
        }
    }
}

pub mod sail_decoder_load {
    use super::*;

    pub fn bool_bits_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<1>) -> bool {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<1>::new(0b1)) } => true,
            _ => false,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn bool_bits_backwards_matches(sail_ctx: &mut Core, arg_hashtag_: BitVector<1>) -> bool {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<1>::new(0b1)) } => true,
            b__1 if { (b__1 == BitVector::<1>::new(0b0)) } => true,
            _ => false,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn size_enc_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<2>) -> word_width {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => word_width::BYTE,
            b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => word_width::HALF,
            b__2 if { (b__2 == BitVector::<2>::new(0b10)) } => word_width::WORD,
            _ => word_width::DOUBLE,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn size_enc_backwards_matches(sail_ctx: &mut Core, arg_hashtag_: BitVector<2>) -> bool {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => true,
            b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => true,
            b__2 if { (b__2 == BitVector::<2>::new(0b10)) } => true,
            b__3 if { (b__3 == BitVector::<2>::new(0b11)) } => true,
            _ => false,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn encdec_compressed_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<16>) -> ast {
        match arg_hashtag_ {
            v__2 if {
                ((v__2.subrange::<13, 16, 3>() == BitVector::<3>::new(0b010))
                    && (v__2.subrange::<0, 2, 2>() == BitVector::<2>::new(0b00)))
            } =>
            {
                let ui6: BitVector<1> = v__2.subrange::<5, 6, 1>();
                let ui53: BitVector<3> = v__2.subrange::<10, 13, 3>();
                let ui2: BitVector<1> = v__2.subrange::<6, 7, 1>();
                let rs1: cregidx = cregidx::Cregidx(v__2.subrange::<7, 10, 3>());
                let rd: cregidx = cregidx::Cregidx(v__2.subrange::<2, 5, 3>());
                ast::C_LW((
                    bitvector_concat::<1, 4, 5>(
                        (ui6 as BitVector<1>),
                        bitvector_concat::<3, 1, 4>((ui53 as BitVector<3>), (ui2 as BitVector<1>)),
                    ),
                    rs1,
                    rd,
                ))
            }
            v__5 if {
                ((64 == 64)
                    && ((v__5.subrange::<13, 16, 3>() == BitVector::<3>::new(0b011))
                        && (v__5.subrange::<0, 2, 2>() == BitVector::<2>::new(0b00))))
            } =>
            {
                let ui76: BitVector<2> = v__5.subrange::<5, 7, 2>();
                let ui53: BitVector<3> = v__5.subrange::<10, 13, 3>();
                let rs1: cregidx = cregidx::Cregidx(v__5.subrange::<7, 10, 3>());
                let rd: cregidx = cregidx::Cregidx(v__5.subrange::<2, 5, 3>());
                ast::C_LD((
                    bitvector_concat::<2, 3, 5>((ui76 as BitVector<2>), (ui53 as BitVector<3>)),
                    rs1,
                    rd,
                ))
            }
            _ => ast::ILLEGAL(BitVector::new(0)),
        }
    }

    pub fn encdec_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<32>) -> ast {
        let head_exp_hashtag_ = arg_hashtag_;
        match match head_exp_hashtag_ {
            v__16
                if {
                    {
                        let mapping1_hashtag_: BitVector<2> = v__16.subrange::<12, 14, 2>();
                        let mapping0_hashtag_: BitVector<1> = v__16.subrange::<14, 15, 1>();
                        (bool_bits_backwards_matches(sail_ctx, mapping0_hashtag_)
                            && size_enc_backwards_matches(sail_ctx, mapping1_hashtag_));
                        (mapping1_hashtag_.bits() != 0
                            && (v__16.subrange::<0, 7, 7>() == BitVector::<7>::new(0b0000011)))
                    }
                } =>
            {
                let imm: BitVector<12> = v__16.subrange::<20, 32, 12>();
                let rs1: BitVector<5> = v__16.subrange::<15, 20, 5>();
                let rd: BitVector<5> = v__16.subrange::<7, 12, 5>();
                let mapping1_hashtag_: BitVector<2> = v__16.subrange::<12, 14, 2>();
                let mapping0_hashtag_: BitVector<1> = v__16.subrange::<14, 15, 1>();
                let imm: BitVector<12> = v__16.subrange::<20, 32, 12>();
                match (
                    bool_bits_backwards(sail_ctx, mapping0_hashtag_),
                    size_enc_backwards(sail_ctx, mapping1_hashtag_),
                ) {
                    (is_unsigned, size) => Some(ast::LOAD((
                        imm,
                        encdec_reg_backwards(rs1),
                        encdec_reg_backwards(rd),
                        is_unsigned,
                        size,
                        false,
                        false,
                    ))),
                    _ => None,
                    _ => {
                        panic!("Unreachable code")
                    }
                }
            }
            _ => None,
            _ => {
                panic!("Unreachable code")
            }
        } {
            Some(result) => result,
            _ => ast::ILLEGAL(BitVector::new(0)),
        }
    }
}

pub mod sail_decoder_store {
    use super::*;

    pub fn size_enc_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<2>) -> word_width {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => word_width::BYTE,
            b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => word_width::HALF,
            b__2 if { (b__2 == BitVector::<2>::new(0b10)) } => word_width::WORD,
            _ => word_width::DOUBLE,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn size_enc_backwards_matches(sail_ctx: &mut Core, arg_hashtag_: BitVector<2>) -> bool {
        match arg_hashtag_ {
            b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => true,
            b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => true,
            b__2 if { (b__2 == BitVector::<2>::new(0b10)) } => true,
            b__3 if { (b__3 == BitVector::<2>::new(0b11)) } => true,
            _ => false,
            _ => {
                panic!("Unreachable code")
            }
        }
    }

    pub fn encdec_compressed_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<16>) -> ast {
        match arg_hashtag_ {
            v__2 if {
                ((v__2.subrange::<13, 16, 3>() == BitVector::<3>::new(0b110))
                    && (v__2.subrange::<0, 2, 2>() == BitVector::<2>::new(0b00)))
            } =>
            {
                let ui6: BitVector<1> = v__2.subrange::<5, 6, 1>();
                let ui53: BitVector<3> = v__2.subrange::<10, 13, 3>();
                let ui2: BitVector<1> = v__2.subrange::<6, 7, 1>();
                let rs2: cregidx = cregidx::Cregidx(v__2.subrange::<2, 5, 3>());
                let rs1: cregidx = cregidx::Cregidx(v__2.subrange::<7, 10, 3>());
                ast::C_SW((
                    bitvector_concat::<1, 4, 5>(
                        (ui6 as BitVector<1>),
                        bitvector_concat::<3, 1, 4>((ui53 as BitVector<3>), (ui2 as BitVector<1>)),
                    ),
                    rs1,
                    rs2,
                ))
            }
            v__5 if {
                ((64 == 64)
                    && ((v__5.subrange::<13, 16, 3>() == BitVector::<3>::new(0b111))
                        && (v__5.subrange::<0, 2, 2>() == BitVector::<2>::new(0b00))))
            } =>
            {
                let ui76: BitVector<2> = v__5.subrange::<5, 7, 2>();
                let ui53: BitVector<3> = v__5.subrange::<10, 13, 3>();
                let rs2: cregidx = cregidx::Cregidx(v__5.subrange::<2, 5, 3>());
                let rs1: cregidx = cregidx::Cregidx(v__5.subrange::<7, 10, 3>());
                ast::C_SD((
                    bitvector_concat::<2, 3, 5>((ui76 as BitVector<2>), (ui53 as BitVector<3>)),
                    rs1,
                    rs2,
                ))
            }
            _ => ast::ILLEGAL(BitVector::new(0)),
        }
    }

    pub fn encdec_backwards(sail_ctx: &mut Core, arg_hashtag_: BitVector<32>) -> ast {
        let head_exp_hashtag_ = arg_hashtag_;
        match match head_exp_hashtag_ {
            v__17
                if {
                    {
                        let mapping0_hashtag_: BitVector<2> = v__17.subrange::<12, 14, 2>();
                        size_enc_backwards_matches(sail_ctx, mapping0_hashtag_);
                        (mapping0_hashtag_.bits() != 0
                            && ((v__17.subrange::<14, 15, 1>() == BitVector::<1>::new(0b0))
                                && (v__17.subrange::<0, 7, 7>() == BitVector::<7>::new(0b0100011))))
                    }
                } =>
            {
                let imm7: BitVector<7> = v__17.subrange::<25, 32, 7>();
                let rs2: regidx = encdec_reg_backwards(v__17.subrange::<20, 25, 5>());
                let rs1: regidx = encdec_reg_backwards(v__17.subrange::<15, 20, 5>());
                let mapping0_hashtag_: BitVector<2> = v__17.subrange::<12, 14, 2>();
                let imm7: BitVector<7> = v__17.subrange::<25, 32, 7>();
                let imm5: BitVector<5> = v__17.subrange::<7, 12, 5>();
                match size_enc_backwards(sail_ctx, mapping0_hashtag_) {
                    size => Some(ast::STORE((
                        bitvector_concat::<7, 5, 12>(
                            (imm7 as BitVector<7>),
                            (imm5 as BitVector<5>),
                        ),
                        rs2,
                        rs1,
                        size,
                        false,
                        false,
                    ))),
                    _ => None,
                    _ => {
                        panic!("Unreachable code")
                    }
                }
            }
            _ => None,
            _ => {
                panic!("Unreachable code")
            }
        } {
            Some(result) => result,
            _ => ast::ILLEGAL(BitVector::new(0)),
        }
    }
}
