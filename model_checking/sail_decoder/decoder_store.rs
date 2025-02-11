#![allow(
    unused,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    bindings_with_variant_name
)]

pub mod sail_decoder_store {

    use sail_model::*;
    use sail_prelude::*;

    pub fn size_enc_backwards(
        sail_ctx: &mut SailVirtCtx,
        arg_hashtag_: BitVector<2>,
    ) -> word_width {
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

    pub fn size_enc_backwards_matches(
        sail_ctx: &mut SailVirtCtx,
        arg_hashtag_: BitVector<2>,
    ) -> bool {
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

    pub fn encdec_compressed_backwards(
        sail_ctx: &mut SailVirtCtx,
        arg_hashtag_: BitVector<16>,
    ) -> ast {
        match arg_hashtag_ {
            v__2 if {
                ((v__2.subrange::<13, 16, 3>() == BitVector::<3>::new(0b110))
                    && (v__2.subrange::<0, 2, 2>() == BitVector::<2>::new(0b00)))
            } =>
            {
                let ui6: BitVector<1> = v__2.subrange::<5, 6, 1>();
                let ui53: BitVector<3> = v__2.subrange::<10, 13, 3>();
                let ui2: BitVector<1> = v__2.subrange::<6, 7, 1>();
                let rs2: cregidx = v__2.subrange::<2, 5, 3>();
                let rs1: cregidx = v__2.subrange::<7, 10, 3>();
                ast::C_SW((
                    bitvector_concat::<1, 4>(
                        (ui6 as BitVector<1>),
                        bitvector_concat::<3, 1>((ui53 as BitVector<3>), (ui2 as BitVector<1>)),
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
                let rs2: BitVector<3> = v__5.subrange::<2, 5, 3>();
                let rs1: BitVector<3> = v__5.subrange::<7, 10, 3>();
                ast::C_SD((
                    bitvector_concat::<2, 3>((ui76 as BitVector<2>), (ui53 as BitVector<3>)),
                    rs1,
                    rs2,
                ))
            }
            _ => ast::ILLEGAL(BitVector::new(0)),
        }
    }

    pub fn encdec_backwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<32>) -> ast {
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
                let rs2: BitVector<5> = v__17.subrange::<20, 25, 5>();
                let rs1: BitVector<5> = v__17.subrange::<15, 20, 5>();
                let mapping0_hashtag_: BitVector<2> = v__17.subrange::<12, 14, 2>();
                let imm7: BitVector<7> = v__17.subrange::<25, 32, 7>();
                let imm5: BitVector<5> = v__17.subrange::<7, 12, 5>();
                match size_enc_backwards(sail_ctx, mapping0_hashtag_) {
                    size => Some(ast::STORE((
                        bitvector_concat::<7, 5>((imm7 as BitVector<7>), (imm5 as BitVector<5>)),
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
