#![allow(
    unused,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    bindings_with_variant_name
)]

use sail_model::*;
use sail_prelude::*;

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

pub fn encdec_backwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<32>) -> ast {
    let head_exp_hashtag_ = arg_hashtag_;
    match match head_exp_hashtag_ {
        v__35
            if {
                {
                    let mapping1_hashtag_: BitVector<2> = v__35.subrange::<12, 14, 2>();
                    let mapping0_hashtag_: BitVector<1> = v__35.subrange::<14, 15, 1>();
                    (bool_bits_backwards_matches(sail_ctx, mapping0_hashtag_)
                        && encdec_csrop_backwards_matches(sail_ctx, mapping1_hashtag_));
                    (mapping1_hashtag_.bits() != 0
                        && (v__35.subrange::<0, 7, 7>() == BitVector::<7>::new(0b1110011)))
                }
            } =>
        {
            let csr: BitVector<12> = v__35.subrange::<20, 32, 12>();
            let rs1: BitVector<5> = v__35.subrange::<15, 20, 5>();
            let rd: BitVector<5> = v__35.subrange::<7, 12, 5>();
            let mapping1_hashtag_: BitVector<2> = v__35.subrange::<12, 14, 2>();
            let mapping0_hashtag_: BitVector<1> = v__35.subrange::<14, 15, 1>();
            let csr: BitVector<12> = v__35.subrange::<20, 32, 12>();
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
            v__7 if { (v__7 == BitVector::<32>::new(0b00010000001000000000000001110011)) } => {
                ast::SRET(())
            }
            v__14 if { (v__14 == BitVector::<32>::new(0b00010000010100000000000001110011)) } => {
                ast::WFI(())
            }
            v__20
                if {
                    ((v__20.subrange::<25, 32, 7>() == BitVector::<7>::new(0b0001001))
                        && (v__20.subrange::<0, 15, 15>()
                            == BitVector::<15>::new(0b000000001110011)))
                } =>
            {
                let rs2: BitVector<5> = v__20.subrange::<20, 25, 5>();
                let rs1: BitVector<5> = v__20.subrange::<15, 20, 5>();
                ast::SFENCE_VMA((rs1, rs2))
            }
            v__25
                if {
                    ((v__25.subrange::<25, 32, 7>() == BitVector::<7>::new(0b0010001))
                        && (v__25.subrange::<0, 15, 15>()
                            == BitVector::<15>::new(0b000000001110011)))
                } =>
            {
                let rs2: BitVector<5> = v__25.subrange::<20, 25, 5>();
                let rs1: BitVector<5> = v__25.subrange::<15, 20, 5>();
                ast::HFENCE_VVMA((rs1, rs2))
            }
            v__30
                if {
                    ((v__30.subrange::<25, 32, 7>() == BitVector::<7>::new(0b0110001))
                        && (v__30.subrange::<0, 15, 15>()
                            == BitVector::<15>::new(0b000000001110011)))
                } =>
            {
                let rs2: BitVector<5> = v__30.subrange::<20, 25, 5>();
                let rs1: BitVector<5> = v__30.subrange::<15, 20, 5>();
                ast::HFENCE_GVMA((rs1, rs2))
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
