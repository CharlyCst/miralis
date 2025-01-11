#![allow(
    unused,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    bindings_with_variant_name
)]

use sail_model::{ast, csrop, word_width, SailVirtCtx};
use sail_prelude::{bitvector_concat, lteq_int, BitVector};

pub fn bool_bits_backwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<1>) -> bool {
    match arg_hashtag_ {
        b__0 if { (b__0 == BitVector::<1>::new(0b1)) } => true,
        _ => false,
        _ => {
            panic!("Unreachable code")
        }
    }
}

pub fn bool_bits_backwards_matches(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<1>) -> bool {
    match arg_hashtag_ {
        b__0 if { (b__0 == BitVector::<1>::new(0b1)) } => true,
        b__1 if { (b__1 == BitVector::<1>::new(0b0)) } => true,
        _ => false,
        _ => {
            panic!("Unreachable code")
        }
    }
}

pub fn encdec_csrop_backwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<2>) -> csrop {
    match arg_hashtag_ {
        b__0 if { (b__0 == BitVector::<2>::new(0b01)) } => csrop::CSRRW,
        b__1 if { (b__1 == BitVector::<2>::new(0b10)) } => csrop::CSRRS,
        b__2 if { (b__2 == BitVector::<2>::new(0b11)) } => csrop::CSRRC,
        _ => {
            panic!("Unreachable code")
        }
    }
}

pub fn encdec_csrop_backwards_matches(
    sail_ctx: &mut SailVirtCtx,
    arg_hashtag_: BitVector<2>,
) -> bool {
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

pub fn size_enc_backwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<2>) -> word_width {
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

pub fn size_enc_backwards_matches(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<2>) -> bool {
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

pub fn size_bytes_forwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: word_width) -> usize {
    match arg_hashtag_ {
        word_width::BYTE => 1,
        word_width::HALF => 2,
        word_width::WORD => 4,
        word_width::DOUBLE => 8,
        _ => {
            panic!("Unreachable code")
        }
    }
}

pub fn encdec_backwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<32>) -> ast {
    let head_exp_hashtag_ = arg_hashtag_;
    match match head_exp_hashtag_ {
        v__37
            if {
                {
                    let mapping1_hashtag_: BitVector<2> = v__37.subrange::<12, 14, 2>();
                    let mapping0_hashtag_: BitVector<1> = v__37.subrange::<14, 15, 1>();
                    (bool_bits_backwards_matches(sail_ctx, mapping0_hashtag_)
                        && encdec_csrop_backwards_matches(sail_ctx, mapping1_hashtag_));
                    (mapping1_hashtag_.bits() != 0
                        && (v__37.subrange::<0, 7, 7>() == BitVector::<7>::new(0b1110011)))
                }
            } =>
        {
            let csr: BitVector<12> = v__37.subrange::<20, 32, 12>();
            let rs1: BitVector<5> = v__37.subrange::<15, 20, 5>();
            let rd: BitVector<5> = v__37.subrange::<7, 12, 5>();
            let mapping1_hashtag_: BitVector<2> = v__37.subrange::<12, 14, 2>();
            let mapping0_hashtag_: BitVector<1> = v__37.subrange::<14, 15, 1>();
            let csr: BitVector<12> = v__37.subrange::<20, 32, 12>();
            match (
                bool_bits_backwards(sail_ctx, mapping0_hashtag_),
                encdec_csrop_backwards(sail_ctx, mapping1_hashtag_),
            ) {
                (is_imm, op) => Some(ast::CSR((csr, rs1, rd, is_imm, op))),
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
            v__13 if { (v__13 == BitVector::<32>::new(0b00000000000000000000000001110011)) } => {
                ast::ECALL(())
            }
            v__19 if { (v__19 == BitVector::<32>::new(0b00000000000100000000000001110011)) } => {
                ast::EBREAK(())
            }
            v__25 if { (v__25 == BitVector::<32>::new(0b00010000001000000000000001110011)) } => {
                ast::SRET(())
            }
            v__32
                if {
                    ((v__32.subrange::<25, 32, 7>() == BitVector::<7>::new(0b0001001))
                        && (v__32.subrange::<0, 15, 15>()
                            == BitVector::<15>::new(0b000000001110011)))
                } =>
            {
                let rs2: BitVector<5> = v__32.subrange::<20, 25, 5>();
                let rs1: BitVector<5> = v__32.subrange::<15, 20, 5>();
                ast::SFENCE_VMA((rs1, rs2))
            }

            v__1408
                if {
                    {
                        let mapping5_hashtag_: BitVector<2> = v__1408.subrange::<12, 14, 2>();
                        size_enc_backwards_matches(sail_ctx, mapping5_hashtag_);
                        (mapping5_hashtag_.bits() != 0
                            && ((v__1408.subrange::<14, 15, 1>() == BitVector::<1>::new(0b0))
                                && (v__1408.subrange::<0, 7, 7>()
                                    == BitVector::<7>::new(0b0100011))))
                    }
                } =>
            {
                let imm7: BitVector<7> = v__1408.subrange::<25, 32, 7>();
                let rs2: BitVector<5> = v__1408.subrange::<20, 25, 5>();
                let rs1: BitVector<5> = v__1408.subrange::<15, 20, 5>();
                let mapping5_hashtag_: BitVector<2> = v__1408.subrange::<12, 14, 2>();
                let imm7: BitVector<7> = v__1408.subrange::<25, 32, 7>();
                let imm5: BitVector<5> = v__1408.subrange::<7, 12, 5>();
                match size_enc_backwards(sail_ctx, mapping5_hashtag_) {
                    size if { lteq_int(size_bytes_forwards(sail_ctx, size), 8) } => ast::STORE((
                        bitvector_concat::<7, 5>((imm7 as BitVector<7>), (imm5 as BitVector<5>)),
                        rs2,
                        rs1,
                        size,
                        false,
                        false,
                    )),
                    _ => {
                        panic!("Unreachable code")
                    }
                }
            }
            _ => ast::ILLEGAL(BitVector::new(0x0)),

            _ => {
                panic!("Unreachable code")
            }
        },
        _ => {
            panic!("Unreachable code")
        }
    }
}
