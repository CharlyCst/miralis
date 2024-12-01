#![allow(
    unused,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    bindings_with_variant_name
)]

use sail_prelude::*;

use crate::sail::Retired::RETIRE_FAIL;

const xlen: usize = 64;

const xlen_bytes: usize = 8;

pub type xlenbits = BitVector<xlen>;

const flen: usize = 64;

const flen_bytes: usize = 8;

type flenbits = BitVector<flen>;

const vlenmax: usize = 65536;

type mem_meta = ();

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct Explicit_access_kind {
    variety: Access_variety,
    strength: Access_strength,
}

type RISCV_strong_access = Access_variety;

const max_mem_access: usize = 4096;

type exc_code = BitVector<8>;

type ext_ptw = ();

type ext_ptw_fail = ();

type ext_ptw_error = ();

type ext_exc_type = ();

type half = BitVector<16>;

type word = BitVector<32>;

type regidx = BitVector<5>;

type cregidx = BitVector<3>;

type csreg = BitVector<12>;

type regno = usize;

type opcode = BitVector<7>;

type imm12 = BitVector<12>;

type imm20 = BitVector<20>;

type amo = BitVector<1>;

type arch_xlen = BitVector<2>;

type priv_level = BitVector<2>;

type tv_mode = BitVector<2>;

type ext_status = BitVector<2>;

type satp_mode = BitVector<4>;

type csrRW = BitVector<2>;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct mul_op {
    high: bool,
    signed_rs1: bool,
    signed_rs2: bool,
}

type ext_access_type = ();

type regtype = xlenbits;

type fregtype = flenbits;

type Misa = BitField<64>;

type Mstatush = BitField<32>;

type Mstatus = BitField<64>;

type Minterrupts = BitField<64>;

type Medeleg = BitField<64>;

type Mtvec = BitField<64>;

type Mcause = BitField<64>;

type Counteren = BitField<32>;

type Counterin = BitField<32>;

type Sstatus = BitField<64>;

type Sedeleg = BitField<64>;

type Sinterrupts = BitField<64>;

type Satp64 = BitField<64>;

type Satp32 = BitField<32>;

type MEnvcfg = BitField<64>;

type SEnvcfg = BitField<64>;

type Vtype = BitField<64>;

type Pmpcfg_ent = BitField<8>;

type pmp_addr_range_in_words = Option<(xlenbits, xlenbits)>;

type ext_fetch_addr_error = ();

type ext_control_addr_error = ();

type ext_data_addr_error = ();

type vreglenbits = BitVector<vlenmax>;

type vregtype = vreglenbits;

type Vcsr = BitField<3>;

type Ustatus = BitField<64>;

type Uinterrupts = BitField<64>;

type ext_exception = ();

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct sync_exception {
    trap: ExceptionType,
    excinfo: Option<xlenbits>,
    ext: Option<ext_exception>,
}

type bits_rm = BitVector<3>;

type bits_fflags = BitVector<5>;

type bits_H = BitVector<16>;

type bits_S = BitVector<32>;

type bits_D = BitVector<64>;

type bits_W = BitVector<32>;

type bits_WU = BitVector<32>;

type bits_L = BitVector<64>;

type bits_LU = BitVector<64>;

type Fcsr = BitField<32>;

type htif_cmd = BitField<64>;

const PAGESIZE_BITS: usize = 12;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct SV_Params {
    va_size_bits: usize,
    vpn_size_bits: usize,
    levels: usize,
    log_pte_size_bytes: usize,
    pte_msbs_lsb_index: usize,
    pte_msbs_size_bits: usize,
    pte_PPNs_lsb_index: usize,
    pte_PPNs_size_bits: usize,
    pte_PPN_j_size_bits: usize,
}

type PTW_Level = usize;

type pte_flags_bits = BitVector<8>;

type extPte = BitVector<64>;

type PTE_Flags = BitField<8>;

type asidbits = BitVector<16>;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct TLB_Entry {
    asid: asidbits,
    global: bool,
    vAddr: BitVector<64>,
    pAddr: BitVector<64>,
    vMatchMask: BitVector<64>,
    vAddrMask: BitVector<64>,
    pte: BitVector<64>,
    pteAddr: BitVector<64>,
    age: BitVector<64>,
}

impl SailVirtCtx {
    pub fn new() -> Self {
        SailVirtCtx {
            elen: BitVector { bits: 0 },
            vlen: BitVector { bits: 3 },
            __monomorphize_reads: false,
            __monomorphize_writes: false,
            PC: BitVector { bits: 0 },
            nextPC: BitVector { bits: 0 },
            instbits: BitVector { bits: 0 },
            x1: BitVector { bits: 0 },
            x2: BitVector { bits: 0 },
            x3: BitVector { bits: 0 },
            x4: BitVector { bits: 0 },
            x5: BitVector { bits: 0 },
            x6: BitVector { bits: 0 },
            x7: BitVector { bits: 0 },
            x8: BitVector { bits: 0 },
            x9: BitVector { bits: 0 },
            x10: BitVector { bits: 0 },
            x11: BitVector { bits: 0 },
            x12: BitVector { bits: 0 },
            x13: BitVector { bits: 0 },
            x14: BitVector { bits: 0 },
            x15: BitVector { bits: 0 },
            x16: BitVector { bits: 0 },
            x17: BitVector { bits: 0 },
            x18: BitVector { bits: 0 },
            x19: BitVector { bits: 0 },
            x20: BitVector { bits: 0 },
            x21: BitVector { bits: 0 },
            x22: BitVector { bits: 0 },
            x23: BitVector { bits: 0 },
            x24: BitVector { bits: 0 },
            x25: BitVector { bits: 0 },
            x26: BitVector { bits: 0 },
            x27: BitVector { bits: 0 },
            x28: BitVector { bits: 0 },
            x29: BitVector { bits: 0 },
            x30: BitVector { bits: 0 },
            x31: BitVector { bits: 0 },
            cur_privilege: Privilege::User,
            cur_inst: BitVector { bits: 0 },
            misa: BitField {
                bits: BitVector { bits: 0 },
            },
            mstatush: BitField {
                bits: BitVector { bits: 0 },
            },
            mstatus: BitField {
                bits: BitVector { bits: 0 },
            },
            mip: BitField {
                bits: BitVector { bits: 0 },
            },
            mie: BitField {
                bits: BitVector { bits: 0 },
            },
            mideleg: BitField {
                bits: BitVector { bits: 0 },
            },
            medeleg: BitField {
                bits: BitVector { bits: 0 },
            },
            mtvec: BitField {
                bits: BitVector { bits: 0 },
            },
            mcause: BitField {
                bits: BitVector { bits: 0 },
            },
            mepc: BitVector { bits: 0 },
            mtval: BitVector { bits: 0 },
            mscratch: BitVector { bits: 0 },
            mcounteren: BitField {
                bits: BitVector { bits: 0 },
            },
            scounteren: BitField {
                bits: BitVector { bits: 0 },
            },
            mcountinhibit: BitField {
                bits: BitVector { bits: 0 },
            },
            mcycle: BitVector { bits: 0 },
            mtime: BitVector { bits: 0 },
            minstret: BitVector { bits: 0 },
            minstret_increment: false,
            mvendorid: BitVector { bits: 0 },
            mimpid: BitVector { bits: 0 },
            marchid: BitVector { bits: 0 },
            mhartid: BitVector { bits: 0 },
            mconfigptr: BitVector { bits: 0 },
            sedeleg: BitField {
                bits: BitVector { bits: 0 },
            },
            sideleg: BitField {
                bits: BitVector { bits: 0 },
            },
            stvec: BitField {
                bits: BitVector { bits: 0 },
            },
            sscratch: BitVector { bits: 0 },
            sepc: BitVector { bits: 0 },
            scause: BitField {
                bits: BitVector { bits: 0 },
            },
            stval: BitVector { bits: 0 },
            tselect: BitVector { bits: 0 },
            menvcfg: BitField {
                bits: BitVector { bits: 0 },
            },
            senvcfg: BitField {
                bits: BitVector { bits: 0 },
            },
            vstart: BitVector { bits: 0 },
            vxsat: BitVector { bits: 0 },
            vxrm: BitVector { bits: 0 },
            vl: BitVector { bits: 0 },
            vlenb: BitVector { bits: 0 },
            vtype: BitField {
                bits: BitVector { bits: 0 },
            },
            pmpcfg_n: [BitField {
                bits: BitVector { bits: 0 },
            }; 64],
            pmpaddr_n: [xlenbits { bits: 0 }; 64],
            vr0: BitVector { bits: 0 },
            vr1: BitVector { bits: 0 },
            vr2: BitVector { bits: 0 },
            vr3: BitVector { bits: 0 },
            vr4: BitVector { bits: 0 },
            vr5: BitVector { bits: 0 },
            vr6: BitVector { bits: 0 },
            vr7: BitVector { bits: 0 },
            vr8: BitVector { bits: 0 },
            vr9: BitVector { bits: 0 },
            vr10: BitVector { bits: 0 },
            vr11: BitVector { bits: 0 },
            vr12: BitVector { bits: 0 },
            vr13: BitVector { bits: 0 },
            vr14: BitVector { bits: 0 },
            vr15: BitVector { bits: 0 },
            vr16: BitVector { bits: 0 },
            vr17: BitVector { bits: 0 },
            vr18: BitVector { bits: 0 },
            vr19: BitVector { bits: 0 },
            vr20: BitVector { bits: 0 },
            vr21: BitVector { bits: 0 },
            vr22: BitVector { bits: 0 },
            vr23: BitVector { bits: 0 },
            vr24: BitVector { bits: 0 },
            vr25: BitVector { bits: 0 },
            vr26: BitVector { bits: 0 },
            vr27: BitVector { bits: 0 },
            vr28: BitVector { bits: 0 },
            vr29: BitVector { bits: 0 },
            vr30: BitVector { bits: 0 },
            vr31: BitVector { bits: 0 },
            vcsr: BitField {
                bits: BitVector { bits: 0 },
            },
            utvec: BitField {
                bits: BitVector { bits: 0 },
            },
            uscratch: BitVector { bits: 0 },
            uepc: BitVector { bits: 0 },
            ucause: BitField {
                bits: BitVector { bits: 0 },
            },
            utval: BitVector { bits: 0 },
            float_result: BitVector { bits: 0 },
            float_fflags: BitVector { bits: 0 },
            f0: BitVector { bits: 0 },
            f1: BitVector { bits: 0 },
            f2: BitVector { bits: 0 },
            f3: BitVector { bits: 0 },
            f4: BitVector { bits: 0 },
            f5: BitVector { bits: 0 },
            f6: BitVector { bits: 0 },
            f7: BitVector { bits: 0 },
            f8: BitVector { bits: 0 },
            f9: BitVector { bits: 0 },
            f10: BitVector { bits: 0 },
            f11: BitVector { bits: 0 },
            f12: BitVector { bits: 0 },
            f13: BitVector { bits: 0 },
            f14: BitVector { bits: 0 },
            f15: BitVector { bits: 0 },
            f16: BitVector { bits: 0 },
            f17: BitVector { bits: 0 },
            f18: BitVector { bits: 0 },
            f19: BitVector { bits: 0 },
            f20: BitVector { bits: 0 },
            f21: BitVector { bits: 0 },
            f22: BitVector { bits: 0 },
            f23: BitVector { bits: 0 },
            f24: BitVector { bits: 0 },
            f25: BitVector { bits: 0 },
            f26: BitVector { bits: 0 },
            f27: BitVector { bits: 0 },
            f28: BitVector { bits: 0 },
            f29: BitVector { bits: 0 },
            f30: BitVector { bits: 0 },
            f31: BitVector { bits: 0 },
            fcsr: BitField {
                bits: BitVector { bits: 0 },
            },
            mtimecmp: BitVector { bits: 0 },
            htif_tohost: BitVector { bits: 0 },
            htif_done: false,
            htif_exit_code: BitVector { bits: 0 },
            htif_cmd_write: false,
            htif_payload_writes: BitVector { bits: 0 },
            tlb: None,
            satp: BitVector { bits: 0 },
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct SailVirtCtx {
    pub elen: BitVector<1>,
    pub vlen: BitVector<4>,
    pub __monomorphize_reads: bool,
    pub __monomorphize_writes: bool,
    pub PC: xlenbits,
    pub nextPC: xlenbits,
    pub instbits: xlenbits,
    pub x1: regtype,
    pub x2: regtype,
    pub x3: regtype,
    pub x4: regtype,
    pub x5: regtype,
    pub x6: regtype,
    pub x7: regtype,
    pub x8: regtype,
    pub x9: regtype,
    pub x10: regtype,
    pub x11: regtype,
    pub x12: regtype,
    pub x13: regtype,
    pub x14: regtype,
    pub x15: regtype,
    pub x16: regtype,
    pub x17: regtype,
    pub x18: regtype,
    pub x19: regtype,
    pub x20: regtype,
    pub x21: regtype,
    pub x22: regtype,
    pub x23: regtype,
    pub x24: regtype,
    pub x25: regtype,
    pub x26: regtype,
    pub x27: regtype,
    pub x28: regtype,
    pub x29: regtype,
    pub x30: regtype,
    pub x31: regtype,
    pub cur_privilege: Privilege,
    pub cur_inst: xlenbits,
    pub misa: Misa,
    pub mstatush: Mstatush,
    pub mstatus: Mstatus,
    pub mip: Minterrupts,
    pub mie: Minterrupts,
    pub mideleg: Minterrupts,
    pub medeleg: Medeleg,
    pub mtvec: Mtvec,
    pub mcause: Mcause,
    pub mepc: xlenbits,
    pub mtval: xlenbits,
    pub mscratch: xlenbits,
    pub mcounteren: Counteren,
    pub scounteren: Counteren,
    pub mcountinhibit: Counterin,
    pub mcycle: BitVector<64>,
    pub mtime: BitVector<64>,
    pub minstret: BitVector<64>,
    pub minstret_increment: bool,
    pub mvendorid: BitVector<32>,
    pub mimpid: xlenbits,
    pub marchid: xlenbits,
    pub mhartid: xlenbits,
    pub mconfigptr: xlenbits,
    pub sedeleg: Sedeleg,
    pub sideleg: Sinterrupts,
    pub stvec: Mtvec,
    pub sscratch: xlenbits,
    pub sepc: xlenbits,
    pub scause: Mcause,
    pub stval: xlenbits,
    pub tselect: xlenbits,
    pub menvcfg: MEnvcfg,
    pub senvcfg: SEnvcfg,
    pub vstart: BitVector<16>,
    pub vxsat: BitVector<1>,
    pub vxrm: BitVector<2>,
    pub vl: xlenbits,
    pub vlenb: xlenbits,
    pub vtype: Vtype,
    pub pmpcfg_n: [Pmpcfg_ent; 64],
    pub pmpaddr_n: [xlenbits; 64],
    pub vr0: vregtype,
    pub vr1: vregtype,
    pub vr2: vregtype,
    pub vr3: vregtype,
    pub vr4: vregtype,
    pub vr5: vregtype,
    pub vr6: vregtype,
    pub vr7: vregtype,
    pub vr8: vregtype,
    pub vr9: vregtype,
    pub vr10: vregtype,
    pub vr11: vregtype,
    pub vr12: vregtype,
    pub vr13: vregtype,
    pub vr14: vregtype,
    pub vr15: vregtype,
    pub vr16: vregtype,
    pub vr17: vregtype,
    pub vr18: vregtype,
    pub vr19: vregtype,
    pub vr20: vregtype,
    pub vr21: vregtype,
    pub vr22: vregtype,
    pub vr23: vregtype,
    pub vr24: vregtype,
    pub vr25: vregtype,
    pub vr26: vregtype,
    pub vr27: vregtype,
    pub vr28: vregtype,
    pub vr29: vregtype,
    pub vr30: vregtype,
    pub vr31: vregtype,
    pub vcsr: Vcsr,
    pub utvec: Mtvec,
    pub uscratch: xlenbits,
    pub uepc: xlenbits,
    pub ucause: Mcause,
    pub utval: xlenbits,
    pub float_result: BitVector<64>,
    pub float_fflags: BitVector<64>,
    pub f0: fregtype,
    pub f1: fregtype,
    pub f2: fregtype,
    pub f3: fregtype,
    pub f4: fregtype,
    pub f5: fregtype,
    pub f6: fregtype,
    pub f7: fregtype,
    pub f8: fregtype,
    pub f9: fregtype,
    pub f10: fregtype,
    pub f11: fregtype,
    pub f12: fregtype,
    pub f13: fregtype,
    pub f14: fregtype,
    pub f15: fregtype,
    pub f16: fregtype,
    pub f17: fregtype,
    pub f18: fregtype,
    pub f19: fregtype,
    pub f20: fregtype,
    pub f21: fregtype,
    pub f22: fregtype,
    pub f23: fregtype,
    pub f24: fregtype,
    pub f25: fregtype,
    pub f26: fregtype,
    pub f27: fregtype,
    pub f28: fregtype,
    pub f29: fregtype,
    pub f30: fregtype,
    pub f31: fregtype,
    pub fcsr: Fcsr,
    pub mtimecmp: BitVector<64>,
    pub htif_tohost: BitVector<64>,
    pub htif_done: bool,
    pub htif_exit_code: BitVector<64>,
    pub htif_cmd_write: bool,
    pub htif_payload_writes: BitVector<4>,
    pub tlb: Option<TLB_Entry>,
    pub satp: xlenbits,
}

fn hex_bits_forwards<const N: usize>(
    sail_ctx: &mut SailVirtCtx,
    bv: BitVector<N>,
) -> (usize, String) {
    (bitvector_length(bv), hex_str(bv.as_usize()))
}

fn hex_bits_forwards_matches<const N: usize>(sail_ctx: &mut SailVirtCtx, bv: BitVector<N>) -> bool {
    true
}

fn get_config_print_reg(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> bool {
    false
}

fn get_config_print_platform(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> bool {
    false
}

fn ones<const N: usize>(sail_ctx: &mut SailVirtCtx, n: usize) -> BitVector<N> {
    sail_ones(n)
}

fn bool_to_bit(sail_ctx: &mut SailVirtCtx, x: bool) -> bool {
    if { x } {
        true
    } else {
        false
    }
}

fn bool_to_bits(sail_ctx: &mut SailVirtCtx, x: bool) -> BitVector<1> {
    let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
    {
        let var_3 = 0;
        let var_4 = bool_to_bit(sail_ctx, x);
        __generated_vector.set_vector_entry(var_3, var_4)
    };
    __generated_vector
}

fn bit_to_bool(sail_ctx: &mut SailVirtCtx, b: bool) -> bool {
    match b {
        true => true,
        false => false,
    }
}

fn _operator_smaller_s_<const N: usize>(
    sail_ctx: &mut SailVirtCtx,
    x: BitVector<N>,
    y: BitVector<N>,
) -> bool {
    (signed(x) < signed(y))
}

fn _operator_smaller_u_<const N: usize>(
    sail_ctx: &mut SailVirtCtx,
    x: BitVector<N>,
    y: BitVector<N>,
) -> bool {
    (x.as_usize() < y.as_usize())
}

fn _operator_smallerequal_u_<const N: usize>(
    sail_ctx: &mut SailVirtCtx,
    x: BitVector<N>,
    y: BitVector<N>,
) -> bool {
    lteq_int(x.as_usize(), y.as_usize())
}

fn _operator_biggerequal_u_<const N: usize>(
    sail_ctx: &mut SailVirtCtx,
    x: BitVector<N>,
    y: BitVector<N>,
) -> bool {
    (x.as_usize() >= y.as_usize())
}

fn get_vlen_pow(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> usize {
    match sail_ctx.vlen {
        b__0 if { (b__0 == BitVector::<4>::new(0b0000)) } => 5,
        b__1 if { (b__1 == BitVector::<4>::new(0b0001)) } => 6,
        b__2 if { (b__2 == BitVector::<4>::new(0b0010)) } => 7,
        b__3 if { (b__3 == BitVector::<4>::new(0b0011)) } => 8,
        b__4 if { (b__4 == BitVector::<4>::new(0b0100)) } => 9,
        b__5 if { (b__5 == BitVector::<4>::new(0b0101)) } => 10,
        b__6 if { (b__6 == BitVector::<4>::new(0b0110)) } => 11,
        b__7 if { (b__7 == BitVector::<4>::new(0b0111)) } => 12,
        b__8 if { (b__8 == BitVector::<4>::new(0b1000)) } => 13,
        b__9 if { (b__9 == BitVector::<4>::new(0b1001)) } => 14,
        b__10 if { (b__10 == BitVector::<4>::new(0b1010)) } => 15,
        _ => 16,
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum result {
    Ok(_tick_a),
    Err(_tick_b),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Access_variety {
    AV_plain,
    AV_exclusive,
    AV_atomic_rmw,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Access_strength {
    AS_normal,
    AS_rel_or_acq,
    AS_acq_rcpc,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Access_kind {
    AK_explicit(Explicit_access_kind),
    AK_ifetch(()),
    AK_ttw(()),
    AK_arch(_tick_arch_ak),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum write_kind {
    Write_plain,
    Write_RISCV_release,
    Write_RISCV_strong_release,
    Write_RISCV_conditional,
    Write_RISCV_conditional_release,
    Write_RISCV_conditional_strong_release,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum read_kind {
    Read_plain,
    Read_ifetch,
    Read_RISCV_acquire,
    Read_RISCV_strong_acquire,
    Read_RISCV_reserved,
    Read_RISCV_reserved_acquire,
    Read_RISCV_reserved_strong_acquire,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum barrier_kind {
    Barrier_RISCV_rw_rw,
    Barrier_RISCV_r_rw,
    Barrier_RISCV_r_r,
    Barrier_RISCV_rw_w,
    Barrier_RISCV_w_w,
    Barrier_RISCV_w_rw,
    Barrier_RISCV_rw_r,
    Barrier_RISCV_r_w,
    Barrier_RISCV_w_r,
    Barrier_RISCV_tso,
    Barrier_RISCV_i,
}

fn ext_exc_type_to_bits(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> BitVector<8> {
    BitVector::<8>::new(0b00011000)
}

fn num_of_ext_exc_type(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> usize {
    24
}

fn ext_exc_type_to_str(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> String {
    String::from("extension-exception")
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Architecture {
    RV32,
    RV64,
    RV128,
}

fn architecture(sail_ctx: &mut SailVirtCtx, a: BitVector<2>) -> Option<Architecture> {
    match a {
        b__0 if { (b__0 == BitVector::<2>::new(0b01)) } => Some(Architecture::RV32),
        b__1 if { (b__1 == BitVector::<2>::new(0b10)) } => Some(Architecture::RV64),
        b__2 if { (b__2 == BitVector::<2>::new(0b11)) } => Some(Architecture::RV128),
        _ => None,
    }
}

fn arch_to_bits(sail_ctx: &mut SailVirtCtx, a: Architecture) -> BitVector<2> {
    match a {
        Architecture::RV32 => BitVector::<2>::new(0b01),
        Architecture::RV64 => BitVector::<2>::new(0b10),
        Architecture::RV128 => BitVector::<2>::new(0b11),
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
enum exception {
    Error_not_implemented(String),
    Error_internal_error(()),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub(crate) enum Privilege {
    User,
    Supervisor,
    Machine,
}

fn privLevel_to_bits(sail_ctx: &mut SailVirtCtx, p: Privilege) -> BitVector<2> {
    match p {
        Privilege::User => BitVector::<2>::new(0b00),
        Privilege::Supervisor => BitVector::<2>::new(0b01),
        Privilege::Machine => BitVector::<2>::new(0b11),
    }
}

fn privLevel_of_bits(sail_ctx: &mut SailVirtCtx, p: BitVector<2>) -> Privilege {
    match p {
        b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => Privilege::User,
        b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => Privilege::Supervisor,
        b__2 if { (b__2 == BitVector::<2>::new(0b11)) } => Privilege::Machine,
        _ => internal_error(
            String::from("../sail-riscv/model/riscv_types.sail"),
            111,
            format!(
                "{}{}",
                String::from("Invalid privilege level: "),
                bits_str(p)
            ),
        ),
    }
}

fn privLevel_to_str(sail_ctx: &mut SailVirtCtx, p: Privilege) -> String {
    match p {
        Privilege::User => String::from("U"),
        Privilege::Supervisor => String::from("S"),
        Privilege::Machine => String::from("M"),
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Retired {
    RETIRE_SUCCESS,
    RETIRE_FAIL,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum AccessType {
    Read(_tick_a),
    Write(_tick_a),
    ReadWrite((_tick_a, _tick_a)),
    Execute(()),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum word_width {
    BYTE,
    HALF,
    WORD,
    DOUBLE,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum InterruptType {
    I_U_Software,
    I_S_Software,
    I_M_Software,
    I_U_Timer,
    I_S_Timer,
    I_M_Timer,
    I_U_External,
    I_S_External,
    I_M_External,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum ExceptionType {
    E_Fetch_Addr_Align(()),
    E_Fetch_Access_Fault(()),
    E_Illegal_Instr(()),
    E_Breakpoint(()),
    E_Load_Addr_Align(()),
    E_Load_Access_Fault(()),
    E_SAMO_Addr_Align(()),
    E_SAMO_Access_Fault(()),
    E_U_EnvCall(()),
    E_S_EnvCall(()),
    E_Reserved_10(()),
    E_M_EnvCall(()),
    E_Fetch_Page_Fault(()),
    E_Load_Page_Fault(()),
    E_Reserved_14(()),
    E_SAMO_Page_Fault(()),
    E_Extension(ext_exc_type),
}

fn exceptionType_to_bits(sail_ctx: &mut SailVirtCtx, e: ExceptionType) -> BitVector<8> {
    match e {
        ExceptionType::E_Fetch_Addr_Align(()) => BitVector::<8>::new(0b00000000),
        ExceptionType::E_Fetch_Access_Fault(()) => BitVector::<8>::new(0b00000001),
        ExceptionType::E_Illegal_Instr(()) => BitVector::<8>::new(0b00000010),
        ExceptionType::E_Breakpoint(()) => BitVector::<8>::new(0b00000011),
        ExceptionType::E_Load_Addr_Align(()) => BitVector::<8>::new(0b00000100),
        ExceptionType::E_Load_Access_Fault(()) => BitVector::<8>::new(0b00000101),
        ExceptionType::E_SAMO_Addr_Align(()) => BitVector::<8>::new(0b00000110),
        ExceptionType::E_SAMO_Access_Fault(()) => BitVector::<8>::new(0b00000111),
        ExceptionType::E_U_EnvCall(()) => BitVector::<8>::new(0b00001000),
        ExceptionType::E_S_EnvCall(()) => BitVector::<8>::new(0b00001001),
        ExceptionType::E_Reserved_10(()) => BitVector::<8>::new(0b00001010),
        ExceptionType::E_M_EnvCall(()) => BitVector::<8>::new(0b00001011),
        ExceptionType::E_Fetch_Page_Fault(()) => BitVector::<8>::new(0b00001100),
        ExceptionType::E_Load_Page_Fault(()) => BitVector::<8>::new(0b00001101),
        ExceptionType::E_Reserved_14(()) => BitVector::<8>::new(0b00001110),
        ExceptionType::E_SAMO_Page_Fault(()) => BitVector::<8>::new(0b00001111),
        ExceptionType::E_Extension(e) => ext_exc_type_to_bits(sail_ctx, e),
    }
}

fn num_of_ExceptionType(sail_ctx: &mut SailVirtCtx, e: ExceptionType) -> usize {
    match e {
        ExceptionType::E_Fetch_Addr_Align(()) => 0,
        ExceptionType::E_Fetch_Access_Fault(()) => 1,
        ExceptionType::E_Illegal_Instr(()) => 2,
        ExceptionType::E_Breakpoint(()) => 3,
        ExceptionType::E_Load_Addr_Align(()) => 4,
        ExceptionType::E_Load_Access_Fault(()) => 5,
        ExceptionType::E_SAMO_Addr_Align(()) => 6,
        ExceptionType::E_SAMO_Access_Fault(()) => 7,
        ExceptionType::E_U_EnvCall(()) => 8,
        ExceptionType::E_S_EnvCall(()) => 9,
        ExceptionType::E_Reserved_10(()) => 10,
        ExceptionType::E_M_EnvCall(()) => 11,
        ExceptionType::E_Fetch_Page_Fault(()) => 12,
        ExceptionType::E_Load_Page_Fault(()) => 13,
        ExceptionType::E_Reserved_14(()) => 14,
        ExceptionType::E_SAMO_Page_Fault(()) => 15,
        ExceptionType::E_Extension(e) => num_of_ext_exc_type(sail_ctx, e),
    }
}

fn exceptionType_to_str(sail_ctx: &mut SailVirtCtx, e: ExceptionType) -> String {
    match e {
        ExceptionType::E_Fetch_Addr_Align(()) => String::from("misaligned-fetch"),
        ExceptionType::E_Fetch_Access_Fault(()) => String::from("fetch-access-fault"),
        ExceptionType::E_Illegal_Instr(()) => String::from("illegal-instruction"),
        ExceptionType::E_Breakpoint(()) => String::from("breakpoint"),
        ExceptionType::E_Load_Addr_Align(()) => String::from("misaligned-load"),
        ExceptionType::E_Load_Access_Fault(()) => String::from("load-access-fault"),
        ExceptionType::E_SAMO_Addr_Align(()) => String::from("misaligned-store/amo"),
        ExceptionType::E_SAMO_Access_Fault(()) => String::from("store/amo-access-fault"),
        ExceptionType::E_U_EnvCall(()) => String::from("u-call"),
        ExceptionType::E_S_EnvCall(()) => String::from("s-call"),
        ExceptionType::E_Reserved_10(()) => String::from("reserved-0"),
        ExceptionType::E_M_EnvCall(()) => String::from("m-call"),
        ExceptionType::E_Fetch_Page_Fault(()) => String::from("fetch-page-fault"),
        ExceptionType::E_Load_Page_Fault(()) => String::from("load-page-fault"),
        ExceptionType::E_Reserved_14(()) => String::from("reserved-1"),
        ExceptionType::E_SAMO_Page_Fault(()) => String::from("store/amo-page-fault"),
        ExceptionType::E_Extension(e) => ext_exc_type_to_str(sail_ctx, e),
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum TrapVectorMode {
    TV_Direct,
    TV_Vector,
    TV_Reserved,
}

fn trapVectorMode_of_bits(sail_ctx: &mut SailVirtCtx, m: BitVector<2>) -> TrapVectorMode {
    match m {
        b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => TrapVectorMode::TV_Direct,
        b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => TrapVectorMode::TV_Vector,
        _ => TrapVectorMode::TV_Reserved,
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum ExtStatus {
    Off,
    Initial,
    Clean,
    Dirty,
}

fn extStatus_to_bits(sail_ctx: &mut SailVirtCtx, e: ExtStatus) -> BitVector<2> {
    match e {
        ExtStatus::Off => BitVector::<2>::new(0b00),
        ExtStatus::Initial => BitVector::<2>::new(0b01),
        ExtStatus::Clean => BitVector::<2>::new(0b10),
        ExtStatus::Dirty => BitVector::<2>::new(0b11),
    }
}

fn extStatus_of_bits(sail_ctx: &mut SailVirtCtx, e: BitVector<2>) -> ExtStatus {
    match e {
        b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => ExtStatus::Off,
        b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => ExtStatus::Initial,
        b__2 if { (b__2 == BitVector::<2>::new(0b10)) } => ExtStatus::Clean,
        _ => ExtStatus::Dirty,
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum SATPMode {
    Sbare,
    Sv32,
    Sv39,
    Sv48,
}

fn satp64Mode_of_bits(
    sail_ctx: &mut SailVirtCtx,
    a: Architecture,
    m: BitVector<4>,
) -> Option<SATPMode> {
    match (a, m) {
        (_, b__0) if { (b__0 == BitVector::<4>::new(0b0000)) } => Some(SATPMode::Sbare),
        (Architecture::RV32, b__1) if { (b__1 == BitVector::<4>::new(0b0001)) } => {
            Some(SATPMode::Sv32)
        }
        (Architecture::RV64, b__2) if { (b__2 == BitVector::<4>::new(0b1000)) } => {
            Some(SATPMode::Sv39)
        }
        (Architecture::RV64, b__3) if { (b__3 == BitVector::<4>::new(0b1001)) } => {
            Some(SATPMode::Sv48)
        }
        (_, _) => None,
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum uop {
    RISCV_LUI,
    RISCV_AUIPC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum bop {
    RISCV_BEQ,
    RISCV_BNE,
    RISCV_BLT,
    RISCV_BGE,
    RISCV_BLTU,
    RISCV_BGEU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum iop {
    RISCV_ADDI,
    RISCV_SLTI,
    RISCV_SLTIU,
    RISCV_XORI,
    RISCV_ORI,
    RISCV_ANDI,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum sop {
    RISCV_SLLI,
    RISCV_SRLI,
    RISCV_SRAI,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum rop {
    RISCV_ADD,
    RISCV_SUB,
    RISCV_SLL,
    RISCV_SLT,
    RISCV_SLTU,
    RISCV_XOR,
    RISCV_SRL,
    RISCV_SRA,
    RISCV_OR,
    RISCV_AND,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum ropw {
    RISCV_ADDW,
    RISCV_SUBW,
    RISCV_SLLW,
    RISCV_SRLW,
    RISCV_SRAW,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum sopw {
    RISCV_SLLIW,
    RISCV_SRLIW,
    RISCV_SRAIW,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum amoop {
    AMOSWAP,
    AMOADD,
    AMOXOR,
    AMOAND,
    AMOOR,
    AMOMIN,
    AMOMAX,
    AMOMINU,
    AMOMAXU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum csrop {
    CSRRW,
    CSRRS,
    CSRRC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum brop_zba {
    RISCV_SH1ADD,
    RISCV_SH2ADD,
    RISCV_SH3ADD,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum brop_zbb {
    RISCV_ANDN,
    RISCV_ORN,
    RISCV_XNOR,
    RISCV_MAX,
    RISCV_MAXU,
    RISCV_MIN,
    RISCV_MINU,
    RISCV_ROL,
    RISCV_ROR,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum brop_zbkb {
    RISCV_PACK,
    RISCV_PACKH,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum brop_zbs {
    RISCV_BCLR,
    RISCV_BEXT,
    RISCV_BINV,
    RISCV_BSET,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum bropw_zba {
    RISCV_ADDUW,
    RISCV_SH1ADDUW,
    RISCV_SH2ADDUW,
    RISCV_SH3ADDUW,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum bropw_zbb {
    RISCV_ROLW,
    RISCV_RORW,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum biop_zbs {
    RISCV_BCLRI,
    RISCV_BEXTI,
    RISCV_BINVI,
    RISCV_BSETI,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum extop_zbb {
    RISCV_SEXTB,
    RISCV_SEXTH,
    RISCV_ZEXTH,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum zicondop {
    RISCV_CZERO_EQZ,
    RISCV_CZERO_NEZ,
}

fn RegStr(sail_ctx: &mut SailVirtCtx, r: BitVector<64>) -> String {
    bits_str(r)
}

fn regval_from_reg(sail_ctx: &mut SailVirtCtx, r: BitVector<64>) -> BitVector<64> {
    r
}

fn regval_into_reg(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> BitVector<64> {
    v
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_madd_op_H {
    FMADD_H,
    FMSUB_H,
    FNMSUB_H,
    FNMADD_H,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_bin_rm_op_H {
    FADD_H,
    FSUB_H,
    FMUL_H,
    FDIV_H,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_un_rm_op_H {
    FSQRT_H,
    FCVT_W_H,
    FCVT_WU_H,
    FCVT_H_W,
    FCVT_H_WU,
    FCVT_H_S,
    FCVT_H_D,
    FCVT_S_H,
    FCVT_D_H,
    FCVT_L_H,
    FCVT_LU_H,
    FCVT_H_L,
    FCVT_H_LU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_un_op_H {
    FCLASS_H,
    FMV_X_H,
    FMV_H_X,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_bin_op_H {
    FSGNJ_H,
    FSGNJN_H,
    FSGNJX_H,
    FMIN_H,
    FMAX_H,
    FEQ_H,
    FLT_H,
    FLE_H,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum rounding_mode {
    RM_RNE,
    RM_RTZ,
    RM_RDN,
    RM_RUP,
    RM_RMM,
    RM_DYN,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_madd_op_S {
    FMADD_S,
    FMSUB_S,
    FNMSUB_S,
    FNMADD_S,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_bin_rm_op_S {
    FADD_S,
    FSUB_S,
    FMUL_S,
    FDIV_S,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_un_rm_op_S {
    FSQRT_S,
    FCVT_W_S,
    FCVT_WU_S,
    FCVT_S_W,
    FCVT_S_WU,
    FCVT_L_S,
    FCVT_LU_S,
    FCVT_S_L,
    FCVT_S_LU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_un_op_S {
    FCLASS_S,
    FMV_X_W,
    FMV_W_X,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_bin_op_S {
    FSGNJ_S,
    FSGNJN_S,
    FSGNJX_S,
    FMIN_S,
    FMAX_S,
    FEQ_S,
    FLT_S,
    FLE_S,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_madd_op_D {
    FMADD_D,
    FMSUB_D,
    FNMSUB_D,
    FNMADD_D,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_bin_rm_op_D {
    FADD_D,
    FSUB_D,
    FMUL_D,
    FDIV_D,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_un_rm_op_D {
    FSQRT_D,
    FCVT_W_D,
    FCVT_WU_D,
    FCVT_D_W,
    FCVT_D_WU,
    FCVT_S_D,
    FCVT_D_S,
    FCVT_L_D,
    FCVT_LU_D,
    FCVT_D_L,
    FCVT_D_LU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_bin_op_D {
    FSGNJ_D,
    FSGNJN_D,
    FSGNJX_D,
    FMIN_D,
    FMAX_D,
    FEQ_D,
    FLT_D,
    FLE_D,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum f_un_op_D {
    FCLASS_D,
    FMV_X_D,
    FMV_D_X,
}

fn rX(sail_ctx: &mut SailVirtCtx, r: usize) -> BitVector<64> {
    let v: regtype = match r {
        0 => zero_reg,
        1 => sail_ctx.x1,
        2 => sail_ctx.x2,
        3 => sail_ctx.x3,
        4 => sail_ctx.x4,
        5 => sail_ctx.x5,
        6 => sail_ctx.x6,
        7 => sail_ctx.x7,
        8 => sail_ctx.x8,
        9 => sail_ctx.x9,
        10 => sail_ctx.x10,
        11 => sail_ctx.x11,
        12 => sail_ctx.x12,
        13 => sail_ctx.x13,
        14 => sail_ctx.x14,
        15 => sail_ctx.x15,
        16 => sail_ctx.x16,
        17 => sail_ctx.x17,
        18 => sail_ctx.x18,
        19 => sail_ctx.x19,
        20 => sail_ctx.x20,
        21 => sail_ctx.x21,
        22 => sail_ctx.x22,
        23 => sail_ctx.x23,
        24 => sail_ctx.x24,
        25 => sail_ctx.x25,
        26 => sail_ctx.x26,
        27 => sail_ctx.x27,
        28 => sail_ctx.x28,
        29 => sail_ctx.x29,
        30 => sail_ctx.x30,
        31 => sail_ctx.x31,
        _ => {
            assert!(false, "todo_process_message");
            __exit()
        }
    };
    regval_from_reg(sail_ctx, v)
}

fn rvfi_wX(sail_ctx: &mut SailVirtCtx, r: usize, v: BitVector<64>) {
    ()
}

fn wX(sail_ctx: &mut SailVirtCtx, r: usize, in_v: BitVector<64>) {
    let v = regval_into_reg(sail_ctx, in_v);
    match r {
        0 => (),
        1 => sail_ctx.x1 = v,
        2 => sail_ctx.x2 = v,
        3 => sail_ctx.x3 = v,
        4 => sail_ctx.x4 = v,
        5 => sail_ctx.x5 = v,
        6 => sail_ctx.x6 = v,
        7 => sail_ctx.x7 = v,
        8 => sail_ctx.x8 = v,
        9 => sail_ctx.x9 = v,
        10 => sail_ctx.x10 = v,
        11 => sail_ctx.x11 = v,
        12 => sail_ctx.x12 = v,
        13 => sail_ctx.x13 = v,
        14 => sail_ctx.x14 = v,
        15 => sail_ctx.x15 = v,
        16 => sail_ctx.x16 = v,
        17 => sail_ctx.x17 = v,
        18 => sail_ctx.x18 = v,
        19 => sail_ctx.x19 = v,
        20 => sail_ctx.x20 = v,
        21 => sail_ctx.x21 = v,
        22 => sail_ctx.x22 = v,
        23 => sail_ctx.x23 = v,
        24 => sail_ctx.x24 = v,
        25 => sail_ctx.x25 = v,
        26 => sail_ctx.x26 = v,
        27 => sail_ctx.x27 = v,
        28 => sail_ctx.x28 = v,
        29 => sail_ctx.x29 = v,
        30 => sail_ctx.x30 = v,
        31 => sail_ctx.x31 = v,
        _ => {
            assert!(false, "todo_process_message")
        }
    };
    if { (r != 0) } {
        rvfi_wX(sail_ctx, r, in_v);
        if { get_config_print_reg(sail_ctx, ()) } {
            print_reg(format!(
                "{}{}",
                String::from("x"),
                format!(
                    "{}{}",
                    dec_str(r),
                    format!("{}{}", String::from(" <- "), RegStr(sail_ctx, v))
                )
            ))
        } else {
            ()
        }
    } else {
        ()
    }
}

fn rX_bits(sail_ctx: &mut SailVirtCtx, i: BitVector<5>) -> BitVector<64> {
    let var_5 = i.as_usize();
    rX(sail_ctx, var_5)
}

fn wX_bits(sail_ctx: &mut SailVirtCtx, i: BitVector<5>, data: BitVector<64>) {
    {
        let var_6 = i.as_usize();
        let var_7 = data;
        wX(sail_ctx, var_6, var_7)
    }
}

fn set_next_pc(sail_ctx: &mut SailVirtCtx, pc: BitVector<64>) {
    sail_branch_announce(64, pc);
    sail_ctx.nextPC = pc
}

fn Mk_Misa(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Misa {
    Misa { bits: v }
}

fn _get_Misa_C(sail_ctx: &mut SailVirtCtx, v: Misa) -> BitVector<1> {
    v.subrange::<2, 3, 1>()
}

fn _update_Misa_C(sail_ctx: &mut SailVirtCtx, v: Misa, x: BitVector<1>) -> Misa {
    BitField {
        bits: update_subrange_bits(v.bits, 2, 2, x),
    }
}

fn _get_Misa_D(sail_ctx: &mut SailVirtCtx, v: Misa) -> BitVector<1> {
    v.subrange::<3, 4, 1>()
}

fn _update_Misa_D(sail_ctx: &mut SailVirtCtx, v: Misa, x: BitVector<1>) -> Misa {
    BitField {
        bits: update_subrange_bits(v.bits, 3, 3, x),
    }
}

fn _get_Misa_F(sail_ctx: &mut SailVirtCtx, v: Misa) -> BitVector<1> {
    v.subrange::<5, 6, 1>()
}

fn _update_Misa_F(sail_ctx: &mut SailVirtCtx, v: Misa, x: BitVector<1>) -> Misa {
    BitField {
        bits: update_subrange_bits(v.bits, 5, 5, x),
    }
}

fn _get_Misa_MXL(sail_ctx: &mut SailVirtCtx, v: Misa) -> BitVector<2> {
    v.subrange::<62, 64, 2>()
}

fn _get_Misa_N(sail_ctx: &mut SailVirtCtx, v: Misa) -> BitVector<1> {
    v.subrange::<13, 14, 1>()
}

fn _get_Misa_S(sail_ctx: &mut SailVirtCtx, v: Misa) -> BitVector<1> {
    v.subrange::<18, 19, 1>()
}

fn _get_Misa_U(sail_ctx: &mut SailVirtCtx, v: Misa) -> BitVector<1> {
    v.subrange::<20, 21, 1>()
}

fn legalize_misa(sail_ctx: &mut SailVirtCtx, m: Misa, v: BitVector<64>) -> Misa {
    let v = Mk_Misa(sail_ctx, v);
    if {
        (!(sys_enable_writable_misa(()))
            || ((_get_Misa_C(sail_ctx, v) == BitVector::<1>::new(0b0))
                && ((bitvector_access(sail_ctx.nextPC, 1) == true)
                    || ext_veto_disable_C(sail_ctx, ()))))
    } {
        m
    } else {
        let m = if { !(sys_enable_rvc(())) } {
            m
        } else {
            {
                let var_12 = m;
                let var_13 = _get_Misa_C(sail_ctx, v);
                _update_Misa_C(sail_ctx, var_12, var_13)
            }
        };
        if { !(sys_enable_fdext(())) } {
            m
        } else {
            {
                let var_8 = {
                    let var_10 = m;
                    let var_11 = _get_Misa_F(sail_ctx, v);
                    _update_Misa_F(sail_ctx, var_10, var_11)
                };
                let var_9 = (_get_Misa_D(sail_ctx, v) & _get_Misa_F(sail_ctx, v));
                _update_Misa_D(sail_ctx, var_8, var_9)
            }
        }
    }
}

fn haveSupMode(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> bool {
    (_get_Misa_S(sail_ctx, sail_ctx.misa) == BitVector::<1>::new(0b1))
}

fn haveUsrMode(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> bool {
    (_get_Misa_U(sail_ctx, sail_ctx.misa) == BitVector::<1>::new(0b1))
}

fn haveNExt(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> bool {
    (_get_Misa_N(sail_ctx, sail_ctx.misa) == BitVector::<1>::new(0b1))
}

fn haveZkr(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> bool {
    true
}

fn lowest_supported_privLevel(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> Privilege {
    if { haveUsrMode(sail_ctx, ()) } {
        Privilege::User
    } else {
        Privilege::Machine
    }
}

fn have_privLevel(sail_ctx: &mut SailVirtCtx, _priv_: BitVector<2>) -> bool {
    match _priv_ {
        b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => haveUsrMode(sail_ctx, ()),
        b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => haveSupMode(sail_ctx, ()),
        b__2 if { (b__2 == BitVector::<2>::new(0b10)) } => false,
        _ => true,
    }
}

fn Mk_Mstatus(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Mstatus {
    Mstatus { bits: v }
}

fn _get_Mstatus_FS(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<2> {
    v.subrange::<13, 15, 2>()
}

fn _update_Mstatus_FS(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<2>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 14, 13, x),
    }
}

fn _get_Mstatus_MIE(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<3, 4, 1>()
}

fn _get_Mstatus_MPIE(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<7, 8, 1>()
}

fn _get_Mstatus_MPP(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<2> {
    v.subrange::<11, 13, 2>()
}

fn _update_Mstatus_MPP(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<2>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 12, 11, x),
    }
}

fn _update_Mstatus_MPRV(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 17, 17, x),
    }
}

fn _get_Mstatus_MXR(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<19, 20, 1>()
}

fn _update_Mstatus_MXR(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 19, 19, x),
    }
}

fn _get_Mstatus_SD(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<63, 64, 1>()
}

fn _update_Mstatus_SD(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 63, 63, x),
    }
}

fn _get_Mstatus_SIE(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<1, 2, 1>()
}

fn _update_Mstatus_SIE(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 1, 1, x),
    }
}

fn _get_Mstatus_SPIE(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<5, 6, 1>()
}

fn _update_Mstatus_SPIE(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 5, 5, x),
    }
}

fn _get_Mstatus_SPP(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<8, 9, 1>()
}

fn _update_Mstatus_SPP(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 8, 8, x),
    }
}

fn _get_Mstatus_SUM(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<18, 19, 1>()
}

fn _update_Mstatus_SUM(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 18, 18, x),
    }
}

fn _get_Mstatus_TVM(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<20, 21, 1>()
}

fn _get_Mstatus_TW(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<21, 22, 1>()
}

fn _get_Mstatus_UIE(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<0, 1, 1>()
}

fn _update_Mstatus_UIE(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn _get_Mstatus_UPIE(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<1> {
    v.subrange::<4, 5, 1>()
}

fn _update_Mstatus_UPIE(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<1>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 4, 4, x),
    }
}

fn _get_Mstatus_VS(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<2> {
    v.subrange::<9, 11, 2>()
}

fn _update_Mstatus_VS(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<2>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 10, 9, x),
    }
}

fn _get_Mstatus_XS(sail_ctx: &mut SailVirtCtx, v: Mstatus) -> BitVector<2> {
    v.subrange::<15, 17, 2>()
}

fn _update_Mstatus_XS(sail_ctx: &mut SailVirtCtx, v: Mstatus, x: BitVector<2>) -> Mstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 16, 15, x),
    }
}

fn get_mstatus_SXL(sail_ctx: &mut SailVirtCtx, m: Mstatus) -> BitVector<2> {
    if { (64 == 32) } {
        panic!("unreachable code")
    } else {
        m.subrange::<34, 36, 2>()
    }
}

fn set_mstatus_SXL(sail_ctx: &mut SailVirtCtx, m: Mstatus, a: BitVector<2>) -> Mstatus {
    if { (64 == 32) } {
        panic!("unreachable code")
    } else {
        let m = update_subrange_bits(m.bits, 35, 34, a);
        Mk_Mstatus(sail_ctx, m)
    }
}

fn get_mstatus_UXL(sail_ctx: &mut SailVirtCtx, m: Mstatus) -> BitVector<2> {
    if { (64 == 32) } {
        panic!("unreachable code")
    } else {
        m.subrange::<32, 34, 2>()
    }
}

fn set_mstatus_UXL(sail_ctx: &mut SailVirtCtx, m: Mstatus, a: BitVector<2>) -> Mstatus {
    if { (64 == 32) } {
        panic!("unreachable code")
    } else {
        let m = update_subrange_bits(m.bits, 33, 32, a);
        Mk_Mstatus(sail_ctx, m)
    }
}

fn legalize_mstatus(sail_ctx: &mut SailVirtCtx, o: Mstatus, v: BitVector<64>) -> Mstatus {
    let m: Mstatus = {
        let var_32 = zero_extend_64(bitvector_concat(
            v.subrange::<7, 23, 16>(),
            bitvector_concat(
                BitVector::<1>::new(0b0),
                bitvector_concat(
                    v.subrange::<3, 6, 3>(),
                    bitvector_concat(BitVector::<1>::new(0b0), v.subrange::<0, 2, 2>()),
                ),
            ),
        ));
        Mk_Mstatus(sail_ctx, var_32)
    };
    let m = {
        let var_28 = m;
        let var_29 = if {
            {
                let var_31 = _get_Mstatus_MPP(sail_ctx, m);
                have_privLevel(sail_ctx, var_31)
            }
        } {
            _get_Mstatus_MPP(sail_ctx, m)
        } else {
            {
                let var_30 = lowest_supported_privLevel(sail_ctx, ());
                privLevel_to_bits(sail_ctx, var_30)
            }
        };
        _update_Mstatus_MPP(sail_ctx, var_28, var_29)
    };
    let m = {
        let var_26 = m;
        let var_27 = extStatus_to_bits(sail_ctx, ExtStatus::Off);
        _update_Mstatus_XS(sail_ctx, var_26, var_27)
    };
    let m = if { sys_enable_zfinx(()) } {
        {
            let var_24 = m;
            let var_25 = extStatus_to_bits(sail_ctx, ExtStatus::Off);
            _update_Mstatus_FS(sail_ctx, var_24, var_25)
        }
    } else {
        m
    };
    let dirty = (({
        let var_23 = _get_Mstatus_FS(sail_ctx, m);
        extStatus_of_bits(sail_ctx, var_23)
    } == ExtStatus::Dirty)
        || (({
            let var_22 = _get_Mstatus_XS(sail_ctx, m);
            extStatus_of_bits(sail_ctx, var_22)
        } == ExtStatus::Dirty)
            || ({
                let var_21 = _get_Mstatus_VS(sail_ctx, m);
                extStatus_of_bits(sail_ctx, var_21)
            } == ExtStatus::Dirty)));
    let m = {
        let var_19 = m;
        let var_20 = bool_to_bits(sail_ctx, dirty);
        _update_Mstatus_SD(sail_ctx, var_19, var_20)
    };
    let m = {
        let var_17 = m;
        let var_18 = get_mstatus_SXL(sail_ctx, o);
        set_mstatus_SXL(sail_ctx, var_17, var_18)
    };
    let m = {
        let var_15 = m;
        let var_16 = get_mstatus_UXL(sail_ctx, o);
        set_mstatus_UXL(sail_ctx, var_15, var_16)
    };
    let m = if { (64 == 64) } {
        {
            let var_14 = update_subrange_bits(m.bits, 37, 36, BitVector::<2>::new(0b00));
            Mk_Mstatus(sail_ctx, var_14)
        }
    } else {
        m
    };
    let m = if { !(haveNExt(sail_ctx, ())) } {
        let m = _update_Mstatus_UPIE(sail_ctx, m, BitVector::<1>::new(0b0));
        let m = _update_Mstatus_UIE(sail_ctx, m, BitVector::<1>::new(0b0));
        m
    } else {
        m
    };
    if { !(haveUsrMode(sail_ctx, ())) } {
        let m = _update_Mstatus_MPRV(sail_ctx, m, BitVector::<1>::new(0b0));
        m
    } else {
        m
    }
}

fn cur_Architecture(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> Architecture {
    let a: arch_xlen = match sail_ctx.cur_privilege {
        Privilege::Machine => _get_Misa_MXL(sail_ctx, sail_ctx.misa),
        Privilege::Supervisor => get_mstatus_SXL(sail_ctx, sail_ctx.mstatus),
        Privilege::User => get_mstatus_UXL(sail_ctx, sail_ctx.mstatus),
    };
    match architecture(sail_ctx, a) {
        Some(a) => a,
        None => internal_error(
            String::from("../sail-riscv/model/riscv_sys_regs.sail"),
            323,
            String::from("Invalid current architecture"),
        ),
    }
}

fn Mk_Minterrupts(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Minterrupts {
    Minterrupts { bits: v }
}

fn _get_Minterrupts_MEI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<11, 12, 1>()
}

fn _update_Minterrupts_MEI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 11, 11, x),
    }
}

fn _get_Minterrupts_MSI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<3, 4, 1>()
}

fn _update_Minterrupts_MSI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 3, 3, x),
    }
}

fn _get_Minterrupts_MTI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<7, 8, 1>()
}

fn _update_Minterrupts_MTI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 7, 7, x),
    }
}

fn _get_Minterrupts_SEI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<9, 10, 1>()
}

fn _update_Minterrupts_SEI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 9, 9, x),
    }
}

fn _get_Minterrupts_SSI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<1, 2, 1>()
}

fn _update_Minterrupts_SSI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 1, 1, x),
    }
}

fn _get_Minterrupts_STI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<5, 6, 1>()
}

fn _update_Minterrupts_STI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 5, 5, x),
    }
}

fn _get_Minterrupts_UEI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<8, 9, 1>()
}

fn _update_Minterrupts_UEI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 8, 8, x),
    }
}

fn _get_Minterrupts_USI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<0, 1, 1>()
}

fn _update_Minterrupts_USI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn _get_Minterrupts_UTI(sail_ctx: &mut SailVirtCtx, v: Minterrupts) -> BitVector<1> {
    v.subrange::<4, 5, 1>()
}

fn _update_Minterrupts_UTI(
    sail_ctx: &mut SailVirtCtx,
    v: Minterrupts,
    x: BitVector<1>,
) -> Minterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 4, 4, x),
    }
}

fn legalize_mip(sail_ctx: &mut SailVirtCtx, o: Minterrupts, v: BitVector<64>) -> Minterrupts {
    let v = Mk_Minterrupts(sail_ctx, v);
    let m = {
        let var_39 = {
            let var_41 = {
                let var_43 = o;
                let var_44 = _get_Minterrupts_SEI(sail_ctx, v);
                _update_Minterrupts_SEI(sail_ctx, var_43, var_44)
            };
            let var_42 = _get_Minterrupts_STI(sail_ctx, v);
            _update_Minterrupts_STI(sail_ctx, var_41, var_42)
        };
        let var_40 = _get_Minterrupts_SSI(sail_ctx, v);
        _update_Minterrupts_SSI(sail_ctx, var_39, var_40)
    };
    if { (haveUsrMode(sail_ctx, ()) && haveNExt(sail_ctx, ())) } {
        {
            let var_33 = {
                let var_35 = {
                    let var_37 = m;
                    let var_38 = _get_Minterrupts_UEI(sail_ctx, v);
                    _update_Minterrupts_UEI(sail_ctx, var_37, var_38)
                };
                let var_36 = _get_Minterrupts_UTI(sail_ctx, v);
                _update_Minterrupts_UTI(sail_ctx, var_35, var_36)
            };
            let var_34 = _get_Minterrupts_USI(sail_ctx, v);
            _update_Minterrupts_USI(sail_ctx, var_33, var_34)
        }
    } else {
        m
    }
}

fn legalize_mie(sail_ctx: &mut SailVirtCtx, o: Minterrupts, v: BitVector<64>) -> Minterrupts {
    let v = Mk_Minterrupts(sail_ctx, v);
    let m = {
        let var_51 = {
            let var_53 = {
                let var_55 = {
                    let var_57 = {
                        let var_59 = {
                            let var_61 = o;
                            let var_62 = _get_Minterrupts_MEI(sail_ctx, v);
                            _update_Minterrupts_MEI(sail_ctx, var_61, var_62)
                        };
                        let var_60 = _get_Minterrupts_MTI(sail_ctx, v);
                        _update_Minterrupts_MTI(sail_ctx, var_59, var_60)
                    };
                    let var_58 = _get_Minterrupts_MSI(sail_ctx, v);
                    _update_Minterrupts_MSI(sail_ctx, var_57, var_58)
                };
                let var_56 = _get_Minterrupts_SEI(sail_ctx, v);
                _update_Minterrupts_SEI(sail_ctx, var_55, var_56)
            };
            let var_54 = _get_Minterrupts_STI(sail_ctx, v);
            _update_Minterrupts_STI(sail_ctx, var_53, var_54)
        };
        let var_52 = _get_Minterrupts_SSI(sail_ctx, v);
        _update_Minterrupts_SSI(sail_ctx, var_51, var_52)
    };
    if { (haveUsrMode(sail_ctx, ()) && haveNExt(sail_ctx, ())) } {
        {
            let var_45 = {
                let var_47 = {
                    let var_49 = m;
                    let var_50 = _get_Minterrupts_UEI(sail_ctx, v);
                    _update_Minterrupts_UEI(sail_ctx, var_49, var_50)
                };
                let var_48 = _get_Minterrupts_UTI(sail_ctx, v);
                _update_Minterrupts_UTI(sail_ctx, var_47, var_48)
            };
            let var_46 = _get_Minterrupts_USI(sail_ctx, v);
            _update_Minterrupts_USI(sail_ctx, var_45, var_46)
        }
    } else {
        m
    }
}

fn legalize_mideleg(
    sail_ctx: &mut SailVirtCtx,
    mideleg: Minterrupts,
    value: BitVector<64>,
) -> Minterrupts {
    {
        let var_63 = {
            let var_65 = {
                let var_67 = Mk_Minterrupts(sail_ctx, value);
                let var_68 = BitVector::<1>::new(0b0);
                _update_Minterrupts_MEI(sail_ctx, var_67, var_68)
            };
            let var_66 = BitVector::<1>::new(0b0);
            _update_Minterrupts_MTI(sail_ctx, var_65, var_66)
        };
        let var_64 = BitVector::<1>::new(0b0);
        _update_Minterrupts_MSI(sail_ctx, var_63, var_64)
    }
}

fn Mk_Medeleg(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Medeleg {
    Medeleg { bits: v }
}

fn _update_Medeleg_MEnvCall(sail_ctx: &mut SailVirtCtx, v: Medeleg, x: BitVector<1>) -> Medeleg {
    BitField {
        bits: update_subrange_bits(v.bits, 11, 11, x),
    }
}

fn legalize_medeleg(sail_ctx: &mut SailVirtCtx, o: Medeleg, v: BitVector<64>) -> Medeleg {
    {
        let var_69 = Mk_Medeleg(sail_ctx, v);
        let var_70 = BitVector::<1>::new(0b0);
        _update_Medeleg_MEnvCall(sail_ctx, var_69, var_70)
    }
}

fn Mk_Mtvec(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Mtvec {
    Mtvec { bits: v }
}

fn _get_Mtvec_Base(sail_ctx: &mut SailVirtCtx, v: Mtvec) -> BitVector<62> {
    v.subrange::<2, 64, 62>()
}

fn _get_Mtvec_Mode(sail_ctx: &mut SailVirtCtx, v: Mtvec) -> BitVector<2> {
    v.subrange::<0, 2, 2>()
}

fn _update_Mtvec_Mode(sail_ctx: &mut SailVirtCtx, v: Mtvec, x: BitVector<2>) -> Mtvec {
    BitField {
        bits: update_subrange_bits(v.bits, 1, 0, x),
    }
}

fn legalize_tvec(sail_ctx: &mut SailVirtCtx, o: Mtvec, v: BitVector<64>) -> Mtvec {
    let v = Mk_Mtvec(sail_ctx, v);
    match {
        let var_73 = _get_Mtvec_Mode(sail_ctx, v);
        trapVectorMode_of_bits(sail_ctx, var_73)
    } {
        TrapVectorMode::TV_Direct => v,
        TrapVectorMode::TV_Vector => v,
        _ => {
            let var_71 = v;
            let var_72 = _get_Mtvec_Mode(sail_ctx, o);
            _update_Mtvec_Mode(sail_ctx, var_71, var_72)
        }
    }
}

fn _get_Mcause_Cause(sail_ctx: &mut SailVirtCtx, v: Mcause) -> BitVector<63> {
    v.subrange::<0, 63, 63>()
}

fn _get_Mcause_IsInterrupt(sail_ctx: &mut SailVirtCtx, v: Mcause) -> BitVector<1> {
    v.subrange::<63, 64, 1>()
}

fn tvec_addr(sail_ctx: &mut SailVirtCtx, m: Mtvec, c: Mcause) -> Option<BitVector<64>> {
    let base: xlenbits =
        bitvector_concat::<62, 2>(_get_Mtvec_Base(sail_ctx, m), BitVector::<2>::new(0b00));
    match {
        let var_75 = _get_Mtvec_Mode(sail_ctx, m);
        trapVectorMode_of_bits(sail_ctx, var_75)
    } {
        TrapVectorMode::TV_Direct => Some(base),
        TrapVectorMode::TV_Vector => {
            if { (_get_Mcause_IsInterrupt(sail_ctx, c) == BitVector::<1>::new(0b1)) } {
                Some({
                    let var_74 = (zero_extend_64(_get_Mcause_Cause(sail_ctx, c)) << 2);
                    base.wrapped_add(var_74)
                })
            } else {
                Some(base)
            }
        }
        TrapVectorMode::TV_Reserved => None,
    }
}

fn legalize_xepc(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> BitVector<64> {
    if {
        ((sys_enable_writable_misa(()) && sys_enable_rvc(()))
            || (_get_Misa_C(sail_ctx, sail_ctx.misa) == BitVector::<1>::new(0b1)))
    } {
        bitvector_update(v, 0, false)
    } else {
        (v & sign_extend(64, BitVector::<3>::new(0b100)))
    }
}

fn pc_alignment_mask(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> BitVector<64> {
    !(zero_extend_64(
        if { (_get_Misa_C(sail_ctx, sail_ctx.misa) == BitVector::<1>::new(0b1)) } {
            BitVector::<2>::new(0b00)
        } else {
            BitVector::<2>::new(0b10)
        },
    ))
}

fn _get_Counteren_CY(sail_ctx: &mut SailVirtCtx, v: Counteren) -> BitVector<1> {
    v.subrange::<0, 1, 1>()
}

fn _update_Counteren_CY(sail_ctx: &mut SailVirtCtx, v: Counteren, x: BitVector<1>) -> Counteren {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn _get_Counteren_IR(sail_ctx: &mut SailVirtCtx, v: Counteren) -> BitVector<1> {
    v.subrange::<2, 3, 1>()
}

fn _update_Counteren_IR(sail_ctx: &mut SailVirtCtx, v: Counteren, x: BitVector<1>) -> Counteren {
    BitField {
        bits: update_subrange_bits(v.bits, 2, 2, x),
    }
}

fn _get_Counteren_TM(sail_ctx: &mut SailVirtCtx, v: Counteren) -> BitVector<1> {
    v.subrange::<1, 2, 1>()
}

fn _update_Counteren_TM(sail_ctx: &mut SailVirtCtx, v: Counteren, x: BitVector<1>) -> Counteren {
    BitField {
        bits: update_subrange_bits(v.bits, 1, 1, x),
    }
}

fn legalize_mcounteren(sail_ctx: &mut SailVirtCtx, c: Counteren, v: BitVector<64>) -> Counteren {
    {
        let var_76 = {
            let var_80 = _update_Counteren_IR(sail_ctx, c, {
                let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
                {
                    let var_84 = 0;
                    let var_85 = bitvector_access(v, 2);
                    __generated_vector.set_vector_entry(var_84, var_85)
                };
                __generated_vector
            });
            let var_81 = {
                let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
                {
                    let var_82 = 0;
                    let var_83 = bitvector_access(v, 1);
                    __generated_vector.set_vector_entry(var_82, var_83)
                };
                __generated_vector
            };
            _update_Counteren_TM(sail_ctx, var_80, var_81)
        };
        let var_77 = {
            let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
            {
                let var_78 = 0;
                let var_79 = bitvector_access(v, 0);
                __generated_vector.set_vector_entry(var_78, var_79)
            };
            __generated_vector
        };
        _update_Counteren_CY(sail_ctx, var_76, var_77)
    }
}

fn legalize_scounteren(sail_ctx: &mut SailVirtCtx, c: Counteren, v: BitVector<64>) -> Counteren {
    {
        let var_86 = {
            let var_90 = _update_Counteren_IR(sail_ctx, c, {
                let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
                {
                    let var_94 = 0;
                    let var_95 = bitvector_access(v, 2);
                    __generated_vector.set_vector_entry(var_94, var_95)
                };
                __generated_vector
            });
            let var_91 = {
                let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
                {
                    let var_92 = 0;
                    let var_93 = bitvector_access(v, 1);
                    __generated_vector.set_vector_entry(var_92, var_93)
                };
                __generated_vector
            };
            _update_Counteren_TM(sail_ctx, var_90, var_91)
        };
        let var_87 = {
            let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
            {
                let var_88 = 0;
                let var_89 = bitvector_access(v, 0);
                __generated_vector.set_vector_entry(var_88, var_89)
            };
            __generated_vector
        };
        _update_Counteren_CY(sail_ctx, var_86, var_87)
    }
}

fn _update_Counterin_CY(sail_ctx: &mut SailVirtCtx, v: Counterin, x: BitVector<1>) -> Counterin {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn _update_Counterin_IR(sail_ctx: &mut SailVirtCtx, v: Counterin, x: BitVector<1>) -> Counterin {
    BitField {
        bits: update_subrange_bits(v.bits, 2, 2, x),
    }
}

fn legalize_mcountinhibit(sail_ctx: &mut SailVirtCtx, c: Counterin, v: BitVector<64>) -> Counterin {
    {
        let var_96 = _update_Counterin_IR(sail_ctx, c, {
            let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
            {
                let var_100 = 0;
                let var_101 = bitvector_access(v, 2);
                __generated_vector.set_vector_entry(var_100, var_101)
            };
            __generated_vector
        });
        let var_97 = {
            let mut __generated_vector: BitVector<1> = BitVector::<1>::new_empty();
            {
                let var_98 = 0;
                let var_99 = bitvector_access(v, 0);
                __generated_vector.set_vector_entry(var_98, var_99)
            };
            __generated_vector
        };
        _update_Counterin_CY(sail_ctx, var_96, var_97)
    }
}

fn Mk_Sstatus(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Sstatus {
    Sstatus { bits: v }
}

fn _get_Sstatus_FS(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<2> {
    v.subrange::<13, 15, 2>()
}

fn _update_Sstatus_FS(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<2>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 14, 13, x),
    }
}

fn _get_Sstatus_MXR(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<1> {
    v.subrange::<19, 20, 1>()
}

fn _update_Sstatus_MXR(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<1>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 19, 19, x),
    }
}

fn _update_Sstatus_SD(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<1>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 63, 63, x),
    }
}

fn _get_Sstatus_SIE(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<1> {
    v.subrange::<1, 2, 1>()
}

fn _update_Sstatus_SIE(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<1>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 1, 1, x),
    }
}

fn _get_Sstatus_SPIE(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<1> {
    v.subrange::<5, 6, 1>()
}

fn _update_Sstatus_SPIE(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<1>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 5, 5, x),
    }
}

fn _get_Sstatus_SPP(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<1> {
    v.subrange::<8, 9, 1>()
}

fn _update_Sstatus_SPP(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<1>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 8, 8, x),
    }
}

fn _get_Sstatus_SUM(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<1> {
    v.subrange::<18, 19, 1>()
}

fn _update_Sstatus_SUM(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<1>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 18, 18, x),
    }
}

fn _get_Sstatus_UIE(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<1> {
    v.subrange::<0, 1, 1>()
}

fn _update_Sstatus_UIE(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<1>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn _get_Sstatus_UPIE(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<1> {
    v.subrange::<4, 5, 1>()
}

fn _update_Sstatus_UPIE(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<1>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 4, 4, x),
    }
}

fn _get_Sstatus_VS(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<2> {
    v.subrange::<9, 11, 2>()
}

fn _update_Sstatus_VS(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<2>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 10, 9, x),
    }
}

fn _get_Sstatus_XS(sail_ctx: &mut SailVirtCtx, v: Sstatus) -> BitVector<2> {
    v.subrange::<15, 17, 2>()
}

fn _update_Sstatus_XS(sail_ctx: &mut SailVirtCtx, v: Sstatus, x: BitVector<2>) -> Sstatus {
    BitField {
        bits: update_subrange_bits(v.bits, 16, 15, x),
    }
}

fn set_sstatus_UXL(sail_ctx: &mut SailVirtCtx, s: Sstatus, a: BitVector<2>) -> Sstatus {
    let m = {
        let var_103 = s.bits;
        Mk_Mstatus(sail_ctx, var_103)
    };
    let m = set_mstatus_UXL(sail_ctx, m, a);
    {
        let var_102 = m.bits;
        Mk_Sstatus(sail_ctx, var_102)
    }
}

fn lower_mstatus(sail_ctx: &mut SailVirtCtx, m: Mstatus) -> Sstatus {
    let s = {
        let var_128 = zero_extend_64(BitVector::<1>::new(0b0));
        Mk_Sstatus(sail_ctx, var_128)
    };
    let s = {
        let var_126 = s;
        let var_127 = _get_Mstatus_SD(sail_ctx, m);
        _update_Sstatus_SD(sail_ctx, var_126, var_127)
    };
    let s = {
        let var_124 = s;
        let var_125 = get_mstatus_UXL(sail_ctx, m);
        set_sstatus_UXL(sail_ctx, var_124, var_125)
    };
    let s = {
        let var_122 = s;
        let var_123 = _get_Mstatus_MXR(sail_ctx, m);
        _update_Sstatus_MXR(sail_ctx, var_122, var_123)
    };
    let s = {
        let var_120 = s;
        let var_121 = _get_Mstatus_SUM(sail_ctx, m);
        _update_Sstatus_SUM(sail_ctx, var_120, var_121)
    };
    let s = {
        let var_118 = s;
        let var_119 = _get_Mstatus_XS(sail_ctx, m);
        _update_Sstatus_XS(sail_ctx, var_118, var_119)
    };
    let s = {
        let var_116 = s;
        let var_117 = _get_Mstatus_FS(sail_ctx, m);
        _update_Sstatus_FS(sail_ctx, var_116, var_117)
    };
    let s = {
        let var_114 = s;
        let var_115 = _get_Mstatus_VS(sail_ctx, m);
        _update_Sstatus_VS(sail_ctx, var_114, var_115)
    };
    let s = {
        let var_112 = s;
        let var_113 = _get_Mstatus_SPP(sail_ctx, m);
        _update_Sstatus_SPP(sail_ctx, var_112, var_113)
    };
    let s = {
        let var_110 = s;
        let var_111 = _get_Mstatus_SPIE(sail_ctx, m);
        _update_Sstatus_SPIE(sail_ctx, var_110, var_111)
    };
    let s = {
        let var_108 = s;
        let var_109 = _get_Mstatus_UPIE(sail_ctx, m);
        _update_Sstatus_UPIE(sail_ctx, var_108, var_109)
    };
    let s = {
        let var_106 = s;
        let var_107 = _get_Mstatus_SIE(sail_ctx, m);
        _update_Sstatus_SIE(sail_ctx, var_106, var_107)
    };
    let s = {
        let var_104 = s;
        let var_105 = _get_Mstatus_UIE(sail_ctx, m);
        _update_Sstatus_UIE(sail_ctx, var_104, var_105)
    };
    s
}

fn lift_sstatus(sail_ctx: &mut SailVirtCtx, m: Mstatus, s: Sstatus) -> Mstatus {
    let m = {
        let var_152 = m;
        let var_153 = _get_Sstatus_MXR(sail_ctx, s);
        _update_Mstatus_MXR(sail_ctx, var_152, var_153)
    };
    let m = {
        let var_150 = m;
        let var_151 = _get_Sstatus_SUM(sail_ctx, s);
        _update_Mstatus_SUM(sail_ctx, var_150, var_151)
    };
    let m = {
        let var_148 = m;
        let var_149 = _get_Sstatus_XS(sail_ctx, s);
        _update_Mstatus_XS(sail_ctx, var_148, var_149)
    };
    let m = {
        let var_146 = m;
        let var_147 = _get_Sstatus_FS(sail_ctx, s);
        _update_Mstatus_FS(sail_ctx, var_146, var_147)
    };
    let m = {
        let var_144 = m;
        let var_145 = _get_Sstatus_VS(sail_ctx, s);
        _update_Mstatus_VS(sail_ctx, var_144, var_145)
    };
    let dirty = (({
        let var_143 = _get_Mstatus_FS(sail_ctx, m);
        extStatus_of_bits(sail_ctx, var_143)
    } == ExtStatus::Dirty)
        || (({
            let var_142 = _get_Mstatus_XS(sail_ctx, m);
            extStatus_of_bits(sail_ctx, var_142)
        } == ExtStatus::Dirty)
            || ({
                let var_141 = _get_Mstatus_VS(sail_ctx, m);
                extStatus_of_bits(sail_ctx, var_141)
            } == ExtStatus::Dirty)));
    let m = {
        let var_139 = m;
        let var_140 = bool_to_bits(sail_ctx, dirty);
        _update_Mstatus_SD(sail_ctx, var_139, var_140)
    };
    let m = {
        let var_137 = m;
        let var_138 = _get_Sstatus_SPP(sail_ctx, s);
        _update_Mstatus_SPP(sail_ctx, var_137, var_138)
    };
    let m = {
        let var_135 = m;
        let var_136 = _get_Sstatus_SPIE(sail_ctx, s);
        _update_Mstatus_SPIE(sail_ctx, var_135, var_136)
    };
    let m = {
        let var_133 = m;
        let var_134 = _get_Sstatus_UPIE(sail_ctx, s);
        _update_Mstatus_UPIE(sail_ctx, var_133, var_134)
    };
    let m = {
        let var_131 = m;
        let var_132 = _get_Sstatus_SIE(sail_ctx, s);
        _update_Mstatus_SIE(sail_ctx, var_131, var_132)
    };
    let m = {
        let var_129 = m;
        let var_130 = _get_Sstatus_UIE(sail_ctx, s);
        _update_Mstatus_UIE(sail_ctx, var_129, var_130)
    };
    m
}

fn legalize_sstatus(sail_ctx: &mut SailVirtCtx, m: Mstatus, v: BitVector<64>) -> Mstatus {
    {
        let var_154 = m;
        let var_155 = {
            let var_156 = m;
            let var_157 = Mk_Sstatus(sail_ctx, v);
            lift_sstatus(sail_ctx, var_156, var_157)
        }
        .bits;
        legalize_mstatus(sail_ctx, var_154, var_155)
    }
}

fn Mk_Sedeleg(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Sedeleg {
    Sedeleg { bits: v }
}

fn legalize_sedeleg(sail_ctx: &mut SailVirtCtx, s: Sedeleg, v: BitVector<64>) -> Sedeleg {
    {
        let var_158 = zero_extend_64(v.subrange::<0, 9, 9>());
        Mk_Sedeleg(sail_ctx, var_158)
    }
}

fn Mk_Sinterrupts(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Sinterrupts {
    Sinterrupts { bits: v }
}

fn _get_Sinterrupts_SEI(sail_ctx: &mut SailVirtCtx, v: Sinterrupts) -> BitVector<1> {
    v.subrange::<9, 10, 1>()
}

fn _update_Sinterrupts_SEI(
    sail_ctx: &mut SailVirtCtx,
    v: Sinterrupts,
    x: BitVector<1>,
) -> Sinterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 9, 9, x),
    }
}

fn _get_Sinterrupts_SSI(sail_ctx: &mut SailVirtCtx, v: Sinterrupts) -> BitVector<1> {
    v.subrange::<1, 2, 1>()
}

fn _update_Sinterrupts_SSI(
    sail_ctx: &mut SailVirtCtx,
    v: Sinterrupts,
    x: BitVector<1>,
) -> Sinterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 1, 1, x),
    }
}

fn _get_Sinterrupts_STI(sail_ctx: &mut SailVirtCtx, v: Sinterrupts) -> BitVector<1> {
    v.subrange::<5, 6, 1>()
}

fn _update_Sinterrupts_STI(
    sail_ctx: &mut SailVirtCtx,
    v: Sinterrupts,
    x: BitVector<1>,
) -> Sinterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 5, 5, x),
    }
}

fn _get_Sinterrupts_UEI(sail_ctx: &mut SailVirtCtx, v: Sinterrupts) -> BitVector<1> {
    v.subrange::<8, 9, 1>()
}

fn _update_Sinterrupts_UEI(
    sail_ctx: &mut SailVirtCtx,
    v: Sinterrupts,
    x: BitVector<1>,
) -> Sinterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 8, 8, x),
    }
}

fn _get_Sinterrupts_USI(sail_ctx: &mut SailVirtCtx, v: Sinterrupts) -> BitVector<1> {
    v.subrange::<0, 1, 1>()
}

fn _update_Sinterrupts_USI(
    sail_ctx: &mut SailVirtCtx,
    v: Sinterrupts,
    x: BitVector<1>,
) -> Sinterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn _get_Sinterrupts_UTI(sail_ctx: &mut SailVirtCtx, v: Sinterrupts) -> BitVector<1> {
    v.subrange::<4, 5, 1>()
}

fn _update_Sinterrupts_UTI(
    sail_ctx: &mut SailVirtCtx,
    v: Sinterrupts,
    x: BitVector<1>,
) -> Sinterrupts {
    BitField {
        bits: update_subrange_bits(v.bits, 4, 4, x),
    }
}

fn lower_mip(sail_ctx: &mut SailVirtCtx, m: Minterrupts, d: Minterrupts) -> Sinterrupts {
    let s: Sinterrupts = {
        let var_171 = zero_extend_64(BitVector::<1>::new(0b0));
        Mk_Sinterrupts(sail_ctx, var_171)
    };
    let s = {
        let var_169 = s;
        let var_170 = (_get_Minterrupts_SEI(sail_ctx, m) & _get_Minterrupts_SEI(sail_ctx, d));
        _update_Sinterrupts_SEI(sail_ctx, var_169, var_170)
    };
    let s = {
        let var_167 = s;
        let var_168 = (_get_Minterrupts_STI(sail_ctx, m) & _get_Minterrupts_STI(sail_ctx, d));
        _update_Sinterrupts_STI(sail_ctx, var_167, var_168)
    };
    let s = {
        let var_165 = s;
        let var_166 = (_get_Minterrupts_SSI(sail_ctx, m) & _get_Minterrupts_SSI(sail_ctx, d));
        _update_Sinterrupts_SSI(sail_ctx, var_165, var_166)
    };
    let s = {
        let var_163 = s;
        let var_164 = (_get_Minterrupts_UEI(sail_ctx, m) & _get_Minterrupts_UEI(sail_ctx, d));
        _update_Sinterrupts_UEI(sail_ctx, var_163, var_164)
    };
    let s = {
        let var_161 = s;
        let var_162 = (_get_Minterrupts_UTI(sail_ctx, m) & _get_Minterrupts_UTI(sail_ctx, d));
        _update_Sinterrupts_UTI(sail_ctx, var_161, var_162)
    };
    let s = {
        let var_159 = s;
        let var_160 = (_get_Minterrupts_USI(sail_ctx, m) & _get_Minterrupts_USI(sail_ctx, d));
        _update_Sinterrupts_USI(sail_ctx, var_159, var_160)
    };
    s
}

fn lower_mie(sail_ctx: &mut SailVirtCtx, m: Minterrupts, d: Minterrupts) -> Sinterrupts {
    let s: Sinterrupts = {
        let var_184 = zero_extend_64(BitVector::<1>::new(0b0));
        Mk_Sinterrupts(sail_ctx, var_184)
    };
    let s = {
        let var_182 = s;
        let var_183 = (_get_Minterrupts_SEI(sail_ctx, m) & _get_Minterrupts_SEI(sail_ctx, d));
        _update_Sinterrupts_SEI(sail_ctx, var_182, var_183)
    };
    let s = {
        let var_180 = s;
        let var_181 = (_get_Minterrupts_STI(sail_ctx, m) & _get_Minterrupts_STI(sail_ctx, d));
        _update_Sinterrupts_STI(sail_ctx, var_180, var_181)
    };
    let s = {
        let var_178 = s;
        let var_179 = (_get_Minterrupts_SSI(sail_ctx, m) & _get_Minterrupts_SSI(sail_ctx, d));
        _update_Sinterrupts_SSI(sail_ctx, var_178, var_179)
    };
    let s = {
        let var_176 = s;
        let var_177 = (_get_Minterrupts_UEI(sail_ctx, m) & _get_Minterrupts_UEI(sail_ctx, d));
        _update_Sinterrupts_UEI(sail_ctx, var_176, var_177)
    };
    let s = {
        let var_174 = s;
        let var_175 = (_get_Minterrupts_UTI(sail_ctx, m) & _get_Minterrupts_UTI(sail_ctx, d));
        _update_Sinterrupts_UTI(sail_ctx, var_174, var_175)
    };
    let s = {
        let var_172 = s;
        let var_173 = (_get_Minterrupts_USI(sail_ctx, m) & _get_Minterrupts_USI(sail_ctx, d));
        _update_Sinterrupts_USI(sail_ctx, var_172, var_173)
    };
    s
}

fn lift_sip(
    sail_ctx: &mut SailVirtCtx,
    mip: Minterrupts,
    mideleg: Minterrupts,
    value: Sinterrupts,
) -> Minterrupts {
    let m: Minterrupts = mip;
    let m = if { (_get_Minterrupts_SSI(sail_ctx, mideleg) == BitVector::<1>::new(0b1)) } {
        {
            let var_189 = m;
            let var_190 = _get_Sinterrupts_SSI(sail_ctx, value);
            _update_Minterrupts_SSI(sail_ctx, var_189, var_190)
        }
    } else {
        m
    };
    if { haveNExt(sail_ctx, ()) } {
        let m = if { (_get_Minterrupts_UEI(sail_ctx, mideleg) == BitVector::<1>::new(0b1)) } {
            {
                let var_187 = m;
                let var_188 = _get_Sinterrupts_UEI(sail_ctx, value);
                _update_Minterrupts_UEI(sail_ctx, var_187, var_188)
            }
        } else {
            m
        };
        let m = if { (_get_Minterrupts_USI(sail_ctx, mideleg) == BitVector::<1>::new(0b1)) } {
            {
                let var_185 = m;
                let var_186 = _get_Sinterrupts_USI(sail_ctx, value);
                _update_Minterrupts_USI(sail_ctx, var_185, var_186)
            }
        } else {
            m
        };
        m
    } else {
        m
    }
}

fn legalize_sip(
    sail_ctx: &mut SailVirtCtx,
    m: Minterrupts,
    d: Minterrupts,
    v: BitVector<64>,
) -> Minterrupts {
    {
        let var_191 = m;
        let var_192 = d;
        let var_193 = Mk_Sinterrupts(sail_ctx, v);
        lift_sip(sail_ctx, var_191, var_192, var_193)
    }
}

fn lift_sie(
    sail_ctx: &mut SailVirtCtx,
    o: Minterrupts,
    d: Minterrupts,
    s: Sinterrupts,
) -> Minterrupts {
    let m: Minterrupts = o;
    let m = if { (_get_Minterrupts_SEI(sail_ctx, d) == BitVector::<1>::new(0b1)) } {
        {
            let var_204 = m;
            let var_205 = _get_Sinterrupts_SEI(sail_ctx, s);
            _update_Minterrupts_SEI(sail_ctx, var_204, var_205)
        }
    } else {
        m
    };
    let m = if { (_get_Minterrupts_STI(sail_ctx, d) == BitVector::<1>::new(0b1)) } {
        {
            let var_202 = m;
            let var_203 = _get_Sinterrupts_STI(sail_ctx, s);
            _update_Minterrupts_STI(sail_ctx, var_202, var_203)
        }
    } else {
        m
    };
    let m = if { (_get_Minterrupts_SSI(sail_ctx, d) == BitVector::<1>::new(0b1)) } {
        {
            let var_200 = m;
            let var_201 = _get_Sinterrupts_SSI(sail_ctx, s);
            _update_Minterrupts_SSI(sail_ctx, var_200, var_201)
        }
    } else {
        m
    };
    if { haveNExt(sail_ctx, ()) } {
        let m = if { (_get_Minterrupts_UEI(sail_ctx, d) == BitVector::<1>::new(0b1)) } {
            {
                let var_198 = m;
                let var_199 = _get_Sinterrupts_UEI(sail_ctx, s);
                _update_Minterrupts_UEI(sail_ctx, var_198, var_199)
            }
        } else {
            m
        };
        let m = if { (_get_Minterrupts_UTI(sail_ctx, d) == BitVector::<1>::new(0b1)) } {
            {
                let var_196 = m;
                let var_197 = _get_Sinterrupts_UTI(sail_ctx, s);
                _update_Minterrupts_UTI(sail_ctx, var_196, var_197)
            }
        } else {
            m
        };
        let m = if { (_get_Minterrupts_USI(sail_ctx, d) == BitVector::<1>::new(0b1)) } {
            {
                let var_194 = m;
                let var_195 = _get_Sinterrupts_USI(sail_ctx, s);
                _update_Minterrupts_USI(sail_ctx, var_194, var_195)
            }
        } else {
            m
        };
        m
    } else {
        m
    }
}

fn legalize_sie(
    sail_ctx: &mut SailVirtCtx,
    m: Minterrupts,
    d: Minterrupts,
    v: BitVector<64>,
) -> Minterrupts {
    {
        let var_206 = m;
        let var_207 = d;
        let var_208 = Mk_Sinterrupts(sail_ctx, v);
        lift_sie(sail_ctx, var_206, var_207, var_208)
    }
}

fn Mk_Satp64(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> Satp64 {
    Satp64 { bits: v }
}

fn _get_Satp64_Mode(sail_ctx: &mut SailVirtCtx, v: Satp64) -> BitVector<4> {
    v.subrange::<60, 64, 4>()
}

fn legalize_satp64(
    sail_ctx: &mut SailVirtCtx,
    a: Architecture,
    o: BitVector<64>,
    v: BitVector<64>,
) -> BitVector<64> {
    let s = Mk_Satp64(sail_ctx, v);
    match {
        let var_209 = a;
        let var_210 = _get_Satp64_Mode(sail_ctx, s);
        satp64Mode_of_bits(sail_ctx, var_209, var_210)
    } {
        None => o,
        Some(SATPMode::Sv32) => o,
        Some(_) => s.bits,
    }
}

fn legalize_satp32(
    sail_ctx: &mut SailVirtCtx,
    a: Architecture,
    o: BitVector<32>,
    v: BitVector<32>,
) -> BitVector<32> {
    v
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum seed_opst {
    BIST,
    ES16,
    WAIT,
    DEAD,
}

fn opst_code_forwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: seed_opst) -> BitVector<2> {
    match arg_hashtag_ {
        seed_opst::BIST => BitVector::<2>::new(0b00),
        seed_opst::WAIT => BitVector::<2>::new(0b01),
        seed_opst::ES16 => BitVector::<2>::new(0b10),
        seed_opst::DEAD => BitVector::<2>::new(0b11),
    }
}

fn read_seed_csr(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> BitVector<64> {
    let reserved_bits: BitVector<6> = BitVector::<6>::new(0b000000);
    let custom_bits: BitVector<8> = BitVector::<8>::new(0b00000000);
    let seed: BitVector<16> = get_16_random_bits(());
    zero_extend_64(bitvector_concat(
        opst_code_forwards(sail_ctx, seed_opst::ES16),
        bitvector_concat(reserved_bits, bitvector_concat(custom_bits, seed)),
    ))
}

fn write_seed_csr(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> Option<BitVector<64>> {
    None
}

fn Mk_MEnvcfg(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> MEnvcfg {
    MEnvcfg { bits: v }
}

fn _get_MEnvcfg_FIOM(sail_ctx: &mut SailVirtCtx, v: MEnvcfg) -> BitVector<1> {
    v.subrange::<0, 1, 1>()
}

fn _update_MEnvcfg_FIOM(sail_ctx: &mut SailVirtCtx, v: MEnvcfg, x: BitVector<1>) -> MEnvcfg {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn Mk_SEnvcfg(sail_ctx: &mut SailVirtCtx, v: BitVector<64>) -> SEnvcfg {
    SEnvcfg { bits: v }
}

fn _get_SEnvcfg_FIOM(sail_ctx: &mut SailVirtCtx, v: SEnvcfg) -> BitVector<1> {
    v.subrange::<0, 1, 1>()
}

fn _update_SEnvcfg_FIOM(sail_ctx: &mut SailVirtCtx, v: SEnvcfg, x: BitVector<1>) -> SEnvcfg {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn legalize_menvcfg(sail_ctx: &mut SailVirtCtx, o: MEnvcfg, v: BitVector<64>) -> MEnvcfg {
    let v = Mk_MEnvcfg(sail_ctx, v);
    let o = {
        let var_211 = o;
        let var_212 = if { sys_enable_writable_fiom(()) } {
            _get_MEnvcfg_FIOM(sail_ctx, v)
        } else {
            BitVector::<1>::new(0b0)
        };
        _update_MEnvcfg_FIOM(sail_ctx, var_211, var_212)
    };
    o
}

fn legalize_senvcfg(sail_ctx: &mut SailVirtCtx, o: SEnvcfg, v: BitVector<64>) -> SEnvcfg {
    let v = Mk_SEnvcfg(sail_ctx, v);
    let o = {
        let var_213 = o;
        let var_214 = if { sys_enable_writable_fiom(()) } {
            _get_SEnvcfg_FIOM(sail_ctx, v)
        } else {
            BitVector::<1>::new(0b0)
        };
        _update_SEnvcfg_FIOM(sail_ctx, var_213, var_214)
    };
    o
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum agtype {
    UNDISTURBED,
    AGNOSTIC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum PmpAddrMatchType {
    OFF,
    TOR,
    NA4,
    NAPOT,
}

fn pmpAddrMatchType_of_bits(sail_ctx: &mut SailVirtCtx, bs: BitVector<2>) -> PmpAddrMatchType {
    match bs {
        b__0 if { (b__0 == BitVector::<2>::new(0b00)) } => PmpAddrMatchType::OFF,
        b__1 if { (b__1 == BitVector::<2>::new(0b01)) } => PmpAddrMatchType::TOR,
        b__2 if { (b__2 == BitVector::<2>::new(0b10)) } => PmpAddrMatchType::NA4,
        _ => PmpAddrMatchType::NAPOT,
    }
}

fn pmpAddrMatchType_to_bits(sail_ctx: &mut SailVirtCtx, bs: PmpAddrMatchType) -> BitVector<2> {
    match bs {
        PmpAddrMatchType::OFF => BitVector::<2>::new(0b00),
        PmpAddrMatchType::TOR => BitVector::<2>::new(0b01),
        PmpAddrMatchType::NA4 => BitVector::<2>::new(0b10),
        PmpAddrMatchType::NAPOT => BitVector::<2>::new(0b11),
    }
}

fn Mk_Pmpcfg_ent(sail_ctx: &mut SailVirtCtx, v: BitVector<8>) -> Pmpcfg_ent {
    Pmpcfg_ent { bits: v }
}

fn _get_Pmpcfg_ent_A(sail_ctx: &mut SailVirtCtx, v: Pmpcfg_ent) -> BitVector<2> {
    v.subrange::<3, 5, 2>()
}

fn _update_Pmpcfg_ent_A(sail_ctx: &mut SailVirtCtx, v: Pmpcfg_ent, x: BitVector<2>) -> Pmpcfg_ent {
    BitField {
        bits: update_subrange_bits(v.bits, 4, 3, x),
    }
}

fn _get_Pmpcfg_ent_L(sail_ctx: &mut SailVirtCtx, v: Pmpcfg_ent) -> BitVector<1> {
    v.subrange::<7, 8, 1>()
}

fn _get_Pmpcfg_ent_R(sail_ctx: &mut SailVirtCtx, v: Pmpcfg_ent) -> BitVector<1> {
    v.subrange::<0, 1, 1>()
}

fn _update_Pmpcfg_ent_R(sail_ctx: &mut SailVirtCtx, v: Pmpcfg_ent, x: BitVector<1>) -> Pmpcfg_ent {
    BitField {
        bits: update_subrange_bits(v.bits, 0, 0, x),
    }
}

fn _get_Pmpcfg_ent_W(sail_ctx: &mut SailVirtCtx, v: Pmpcfg_ent) -> BitVector<1> {
    v.subrange::<1, 2, 1>()
}

fn _update_Pmpcfg_ent_W(sail_ctx: &mut SailVirtCtx, v: Pmpcfg_ent, x: BitVector<1>) -> Pmpcfg_ent {
    BitField {
        bits: update_subrange_bits(v.bits, 1, 1, x),
    }
}

fn _update_Pmpcfg_ent_X(sail_ctx: &mut SailVirtCtx, v: Pmpcfg_ent, x: BitVector<1>) -> Pmpcfg_ent {
    BitField {
        bits: update_subrange_bits(v.bits, 2, 2, x),
    }
}

fn pmpReadCfgReg(sail_ctx: &mut SailVirtCtx, n: usize) -> BitVector<64> {
    if { (64 == 32) } {
        panic!("unreachable code")
    } else {
        // todo: fix this in the next version of the tranpiled code
        // assert!(false, "todo_process_message");
        bitvector_concat::<8, 56>(
            sail_ctx.pmpcfg_n[((n * 4) + 7)].bits,
            bitvector_concat::<8, 48>(
                sail_ctx.pmpcfg_n[((n * 4) + 6)].bits,
                bitvector_concat::<8, 40>(
                    sail_ctx.pmpcfg_n[((n * 4) + 5)].bits,
                    bitvector_concat::<8, 32>(
                        sail_ctx.pmpcfg_n[((n * 4) + 4)].bits,
                        bitvector_concat::<8, 24>(
                            sail_ctx.pmpcfg_n[((n * 4) + 3)].bits,
                            bitvector_concat::<8, 16>(
                                sail_ctx.pmpcfg_n[((n * 4) + 2)].bits,
                                bitvector_concat::<8, 8>(
                                    sail_ctx.pmpcfg_n[((n * 4) + 1)].bits,
                                    sail_ctx.pmpcfg_n[((n * 4) + 0)].bits,
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    }
}

fn pmpReadAddrReg(sail_ctx: &mut SailVirtCtx, n: usize) -> BitVector<64> {
    let G = sys_pmp_grain(());
    let match_type = _get_Pmpcfg_ent_A(sail_ctx, sail_ctx.pmpcfg_n[n]);
    let addr = sail_ctx.pmpaddr_n[n];
    match bitvector_access(match_type, 1) {
        true if { (G >= 2) } => {
            let mask: xlenbits = zero_extend_64({
                let var_215 = min_int((G - 1), 64);
                ones::<64>(sail_ctx, var_215)
            });
            (addr | mask)
        }
        false if { (G >= 1) } => {
            let mask: xlenbits = zero_extend_64({
                let var_216 = min_int(G, 64);
                ones::<64>(sail_ctx, var_216)
            });
            (addr & !(mask))
        }
        _ => addr,
    }
}

fn pmpLocked(sail_ctx: &mut SailVirtCtx, cfg: Pmpcfg_ent) -> bool {
    (_get_Pmpcfg_ent_L(sail_ctx, cfg) == BitVector::<1>::new(0b1))
}

fn pmpTORLocked(sail_ctx: &mut SailVirtCtx, cfg: Pmpcfg_ent) -> bool {
    ((_get_Pmpcfg_ent_L(sail_ctx, cfg) == BitVector::<1>::new(0b1))
        && ({
            let var_217 = _get_Pmpcfg_ent_A(sail_ctx, cfg);
            pmpAddrMatchType_of_bits(sail_ctx, var_217)
        } == PmpAddrMatchType::TOR))
}

fn pmpWriteCfg(
    sail_ctx: &mut SailVirtCtx,
    n: usize,
    cfg: Pmpcfg_ent,
    v: BitVector<8>,
) -> Pmpcfg_ent {
    if { pmpLocked(sail_ctx, cfg) } {
        cfg
    } else {
        let cfg = {
            let var_225 = (v & BitVector::<8>::new(0b10011111));
            Mk_Pmpcfg_ent(sail_ctx, var_225)
        };
        let cfg = if {
            ((_get_Pmpcfg_ent_W(sail_ctx, cfg) == BitVector::<1>::new(0b1))
                && (_get_Pmpcfg_ent_R(sail_ctx, cfg) == BitVector::<1>::new(0b0)))
        } {
            {
                let var_221 = {
                    let var_223 = _update_Pmpcfg_ent_X(sail_ctx, cfg, BitVector::<1>::new(0b0));
                    let var_224 = BitVector::<1>::new(0b0);
                    _update_Pmpcfg_ent_W(sail_ctx, var_223, var_224)
                };
                let var_222 = BitVector::<1>::new(0b0);
                _update_Pmpcfg_ent_R(sail_ctx, var_221, var_222)
            }
        } else {
            cfg
        };
        /*let cfg = if {
            ((sys_pmp_grain(()) >= 1)
                && ({
                    let var_220 = _get_Pmpcfg_ent_A(sail_ctx, cfg);
                    pmpAddrMatchType_of_bits(sail_ctx, var_220)
                } == PmpAddrMatchType::NA4))
        } {
            {
                let var_218 = cfg;
                let var_219 = pmpAddrMatchType_to_bits(sail_ctx, PmpAddrMatchType::OFF);
                _update_Pmpcfg_ent_A(sail_ctx, var_218, var_219)
            }
        } else {
            cfg
        };*/
        cfg
    }
}

fn pmpWriteCfgReg(sail_ctx: &mut SailVirtCtx, n: usize, v: BitVector<64>) {
    if { (64 == 32) } {
        panic!("unreachable code")
    } else {
        // Todo: fix this in the next transpiled code
        // assert!(false, "todo_process_message");
        for i in 0..8 {
            // TODO: Fix the bug in virtsail
            let idx = ((n * 4) + i);
            sail_ctx.pmpcfg_n[idx] = {
                let var_226 = idx;
                let var_227 = sail_ctx.pmpcfg_n[idx];
                let var_228 = subrange_bits_8(v, ((8 * i) + 7), (8 * i));
                pmpWriteCfg(sail_ctx, var_226, var_227, var_228)
            }
        }
    }
}

fn pmpWriteAddr(
    sail_ctx: &mut SailVirtCtx,
    locked: bool,
    tor_locked: bool,
    reg: BitVector<64>,
    v: BitVector<64>,
) -> BitVector<64> {
    if { (64 == 32) } {
        panic!("unreachable code")
    } else {
        if { (locked || tor_locked) } {
            reg
        } else {
            zero_extend_64(v.subrange::<0, 54, 54>())
        }
    }
}

fn pmpWriteAddrReg(sail_ctx: &mut SailVirtCtx, n: usize, v: BitVector<64>) {
    sail_ctx.pmpaddr_n[n] = {
        let var_232 = pmpLocked(sail_ctx, sail_ctx.pmpcfg_n[n]);
        let var_233 = if { ((n + 1) < 64) } {
            pmpTORLocked(sail_ctx, sail_ctx.pmpcfg_n[(n + 1)])
        } else {
            false
        };
        let var_234 = sail_ctx.pmpaddr_n[n];
        let var_235 = v;
        pmpWriteAddr(sail_ctx, var_232, var_233, var_234, var_235)
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum pmpAddrMatch {
    PMP_NoMatch,
    PMP_PartialMatch,
    PMP_Match,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum pmpMatch {
    PMP_Success,
    PMP_Continue,
    PMP_Fail,
}

fn ext_check_CSR(
    sail_ctx: &mut SailVirtCtx,
    csrno: BitVector<12>,
    p: Privilege,
    isWrite: bool,
) -> bool {
    true
}

fn ext_check_CSR_fail(sail_ctx: &mut SailVirtCtx, unit_arg: ()) {
    ()
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Ext_FetchAddr_Check {
    Ext_FetchAddr_OK(xlenbits),
    Ext_FetchAddr_Error(_tick_a),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Ext_ControlAddr_Check {
    Ext_ControlAddr_OK(xlenbits),
    Ext_ControlAddr_Error(_tick_a),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Ext_DataAddr_Check {
    Ext_DataAddr_OK(xlenbits),
    Ext_DataAddr_Error(_tick_a),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Ext_PhysAddr_Check {
    Ext_PhysAddr_OK(()),
    Ext_PhysAddr_Error(ExceptionType),
}

fn ext_veto_disable_C(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> bool {
    false
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vvfunct6 {
    VV_VADD,
    VV_VSUB,
    VV_VMINU,
    VV_VMIN,
    VV_VMAXU,
    VV_VMAX,
    VV_VAND,
    VV_VOR,
    VV_VXOR,
    VV_VRGATHER,
    VV_VRGATHEREI16,
    VV_VSADDU,
    VV_VSADD,
    VV_VSSUBU,
    VV_VSSUB,
    VV_VSLL,
    VV_VSMUL,
    VV_VSRL,
    VV_VSRA,
    VV_VSSRL,
    VV_VSSRA,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vvcmpfunct6 {
    VVCMP_VMSEQ,
    VVCMP_VMSNE,
    VVCMP_VMSLTU,
    VVCMP_VMSLT,
    VVCMP_VMSLEU,
    VVCMP_VMSLE,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vvmfunct6 {
    VVM_VMADC,
    VVM_VMSBC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vvmcfunct6 {
    VVMC_VMADC,
    VVMC_VMSBC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vvmsfunct6 {
    VVMS_VADC,
    VVMS_VSBC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vxmfunct6 {
    VXM_VMADC,
    VXM_VMSBC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vxmcfunct6 {
    VXMC_VMADC,
    VXMC_VMSBC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vxmsfunct6 {
    VXMS_VADC,
    VXMS_VSBC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vimfunct6 {
    VIM_VMADC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vimcfunct6 {
    VIMC_VMADC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vimsfunct6 {
    VIMS_VADC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vxcmpfunct6 {
    VXCMP_VMSEQ,
    VXCMP_VMSNE,
    VXCMP_VMSLTU,
    VXCMP_VMSLT,
    VXCMP_VMSLEU,
    VXCMP_VMSLE,
    VXCMP_VMSGTU,
    VXCMP_VMSGT,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vicmpfunct6 {
    VICMP_VMSEQ,
    VICMP_VMSNE,
    VICMP_VMSLEU,
    VICMP_VMSLE,
    VICMP_VMSGTU,
    VICMP_VMSGT,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum nvfunct6 {
    NV_VNCLIPU,
    NV_VNCLIP,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum nvsfunct6 {
    NVS_VNSRL,
    NVS_VNSRA,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum nxfunct6 {
    NX_VNCLIPU,
    NX_VNCLIP,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum nxsfunct6 {
    NXS_VNSRL,
    NXS_VNSRA,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum mmfunct6 {
    MM_VMAND,
    MM_VMNAND,
    MM_VMANDN,
    MM_VMXOR,
    MM_VMOR,
    MM_VMNOR,
    MM_VMORN,
    MM_VMXNOR,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum nifunct6 {
    NI_VNCLIPU,
    NI_VNCLIP,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum nisfunct6 {
    NIS_VNSRL,
    NIS_VNSRA,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum wvvfunct6 {
    WVV_VADD,
    WVV_VSUB,
    WVV_VADDU,
    WVV_VSUBU,
    WVV_VWMUL,
    WVV_VWMULU,
    WVV_VWMULSU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum wvfunct6 {
    WV_VADD,
    WV_VSUB,
    WV_VADDU,
    WV_VSUBU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum wvxfunct6 {
    WVX_VADD,
    WVX_VSUB,
    WVX_VADDU,
    WVX_VSUBU,
    WVX_VWMUL,
    WVX_VWMULU,
    WVX_VWMULSU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum wxfunct6 {
    WX_VADD,
    WX_VSUB,
    WX_VADDU,
    WX_VSUBU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vext2funct6 {
    VEXT2_ZVF2,
    VEXT2_SVF2,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vext4funct6 {
    VEXT4_ZVF4,
    VEXT4_SVF4,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vext8funct6 {
    VEXT8_ZVF8,
    VEXT8_SVF8,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vxfunct6 {
    VX_VADD,
    VX_VSUB,
    VX_VRSUB,
    VX_VMINU,
    VX_VMIN,
    VX_VMAXU,
    VX_VMAX,
    VX_VAND,
    VX_VOR,
    VX_VXOR,
    VX_VSADDU,
    VX_VSADD,
    VX_VSSUBU,
    VX_VSSUB,
    VX_VSLL,
    VX_VSMUL,
    VX_VSRL,
    VX_VSRA,
    VX_VSSRL,
    VX_VSSRA,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vifunct6 {
    VI_VADD,
    VI_VRSUB,
    VI_VAND,
    VI_VOR,
    VI_VXOR,
    VI_VSADDU,
    VI_VSADD,
    VI_VSLL,
    VI_VSRL,
    VI_VSRA,
    VI_VSSRL,
    VI_VSSRA,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vxsgfunct6 {
    VX_VSLIDEUP,
    VX_VSLIDEDOWN,
    VX_VRGATHER,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum visgfunct6 {
    VI_VSLIDEUP,
    VI_VSLIDEDOWN,
    VI_VRGATHER,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum mvvfunct6 {
    MVV_VAADDU,
    MVV_VAADD,
    MVV_VASUBU,
    MVV_VASUB,
    MVV_VMUL,
    MVV_VMULH,
    MVV_VMULHU,
    MVV_VMULHSU,
    MVV_VDIVU,
    MVV_VDIV,
    MVV_VREMU,
    MVV_VREM,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum mvvmafunct6 {
    MVV_VMACC,
    MVV_VNMSAC,
    MVV_VMADD,
    MVV_VNMSUB,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum rmvvfunct6 {
    MVV_VREDSUM,
    MVV_VREDAND,
    MVV_VREDOR,
    MVV_VREDXOR,
    MVV_VREDMINU,
    MVV_VREDMIN,
    MVV_VREDMAXU,
    MVV_VREDMAX,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum rivvfunct6 {
    IVV_VWREDSUMU,
    IVV_VWREDSUM,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum rfvvfunct6 {
    FVV_VFREDOSUM,
    FVV_VFREDUSUM,
    FVV_VFREDMAX,
    FVV_VFREDMIN,
    FVV_VFWREDOSUM,
    FVV_VFWREDUSUM,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum wmvvfunct6 {
    WMVV_VWMACCU,
    WMVV_VWMACC,
    WMVV_VWMACCSU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum mvxfunct6 {
    MVX_VAADDU,
    MVX_VAADD,
    MVX_VASUBU,
    MVX_VASUB,
    MVX_VSLIDE1UP,
    MVX_VSLIDE1DOWN,
    MVX_VMUL,
    MVX_VMULH,
    MVX_VMULHU,
    MVX_VMULHSU,
    MVX_VDIVU,
    MVX_VDIV,
    MVX_VREMU,
    MVX_VREM,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum mvxmafunct6 {
    MVX_VMACC,
    MVX_VNMSAC,
    MVX_VMADD,
    MVX_VNMSUB,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum wmvxfunct6 {
    WMVX_VWMACCU,
    WMVX_VWMACC,
    WMVX_VWMACCUS,
    WMVX_VWMACCSU,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum maskfunct3 {
    VV_VMERGE,
    VI_VMERGE,
    VX_VMERGE,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vlewidth {
    VLE8,
    VLE16,
    VLE32,
    VLE64,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fvvfunct6 {
    FVV_VADD,
    FVV_VSUB,
    FVV_VMIN,
    FVV_VMAX,
    FVV_VSGNJ,
    FVV_VSGNJN,
    FVV_VSGNJX,
    FVV_VDIV,
    FVV_VMUL,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fvvmafunct6 {
    FVV_VMADD,
    FVV_VNMADD,
    FVV_VMSUB,
    FVV_VNMSUB,
    FVV_VMACC,
    FVV_VNMACC,
    FVV_VMSAC,
    FVV_VNMSAC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fwvvfunct6 {
    FWVV_VADD,
    FWVV_VSUB,
    FWVV_VMUL,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fwvvmafunct6 {
    FWVV_VMACC,
    FWVV_VNMACC,
    FWVV_VMSAC,
    FWVV_VNMSAC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fwvfunct6 {
    FWV_VADD,
    FWV_VSUB,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fvvmfunct6 {
    FVVM_VMFEQ,
    FVVM_VMFLE,
    FVVM_VMFLT,
    FVVM_VMFNE,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vfunary0 {
    FV_CVT_XU_F,
    FV_CVT_X_F,
    FV_CVT_F_XU,
    FV_CVT_F_X,
    FV_CVT_RTZ_XU_F,
    FV_CVT_RTZ_X_F,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vfwunary0 {
    FWV_CVT_XU_F,
    FWV_CVT_X_F,
    FWV_CVT_F_XU,
    FWV_CVT_F_X,
    FWV_CVT_F_F,
    FWV_CVT_RTZ_XU_F,
    FWV_CVT_RTZ_X_F,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vfnunary0 {
    FNV_CVT_XU_F,
    FNV_CVT_X_F,
    FNV_CVT_F_XU,
    FNV_CVT_F_X,
    FNV_CVT_F_F,
    FNV_CVT_ROD_F_F,
    FNV_CVT_RTZ_XU_F,
    FNV_CVT_RTZ_X_F,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vfunary1 {
    FVV_VSQRT,
    FVV_VRSQRT7,
    FVV_VREC7,
    FVV_VCLASS,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fvffunct6 {
    VF_VADD,
    VF_VSUB,
    VF_VMIN,
    VF_VMAX,
    VF_VSGNJ,
    VF_VSGNJN,
    VF_VSGNJX,
    VF_VDIV,
    VF_VRDIV,
    VF_VMUL,
    VF_VRSUB,
    VF_VSLIDE1UP,
    VF_VSLIDE1DOWN,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fvfmafunct6 {
    VF_VMADD,
    VF_VNMADD,
    VF_VMSUB,
    VF_VNMSUB,
    VF_VMACC,
    VF_VNMACC,
    VF_VMSAC,
    VF_VNMSAC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fwvffunct6 {
    FWVF_VADD,
    FWVF_VSUB,
    FWVF_VMUL,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fwvfmafunct6 {
    FWVF_VMACC,
    FWVF_VNMACC,
    FWVF_VMSAC,
    FWVF_VNMSAC,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fwffunct6 {
    FWF_VADD,
    FWF_VSUB,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum fvfmfunct6 {
    VFM_VMFEQ,
    VFM_VMFLE,
    VFM_VMFLT,
    VFM_VMFNE,
    VFM_VMFGT,
    VFM_VMFGE,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum vmlsop {
    VLM,
    VSM,
}

fn ext_check_xret_priv(sail_ctx: &mut SailVirtCtx, p: Privilege) -> bool {
    true
}

fn ext_fail_xret_priv(sail_ctx: &mut SailVirtCtx, unit_arg: ()) {
    ()
}

fn handle_trap_extension(
    sail_ctx: &mut SailVirtCtx,
    p: Privilege,
    pc: BitVector<64>,
    u: Option<()>,
) {
    ()
}

fn prepare_trap_vector(sail_ctx: &mut SailVirtCtx, p: Privilege, cause: Mcause) -> BitVector<64> {
    let tvec: Mtvec = match p {
        Privilege::Machine => sail_ctx.mtvec,
        Privilege::Supervisor => sail_ctx.stvec,
        Privilege::User => sail_ctx.utvec,
    };
    match tvec_addr(sail_ctx, tvec, cause) {
        Some(epc) => epc,
        None => internal_error(
            String::from("../sail-riscv/model/riscv_sys_exceptions.sail"),
            29,
            String::from("Invalid tvec mode"),
        ),
    }
}

fn get_xret_target(sail_ctx: &mut SailVirtCtx, p: Privilege) -> BitVector<64> {
    match p {
        Privilege::Machine => sail_ctx.mepc,
        Privilege::Supervisor => sail_ctx.sepc,
        Privilege::User => sail_ctx.uepc,
    }
}

fn set_xret_target(
    sail_ctx: &mut SailVirtCtx,
    p: Privilege,
    value: BitVector<64>,
) -> BitVector<64> {
    let target = legalize_xepc(sail_ctx, value);
    match p {
        Privilege::Machine => sail_ctx.mepc = target,
        Privilege::Supervisor => sail_ctx.sepc = target,
        Privilege::User => sail_ctx.uepc = target,
    };
    target
}

fn prepare_xret_target(sail_ctx: &mut SailVirtCtx, p: Privilege) -> BitVector<64> {
    get_xret_target(sail_ctx, p)
}

fn get_mtvec(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> BitVector<64> {
    sail_ctx.mtvec.bits
}

fn get_stvec(sail_ctx: &mut SailVirtCtx, unit_arg: ()) -> BitVector<64> {
    sail_ctx.stvec.bits
}

fn set_mtvec(sail_ctx: &mut SailVirtCtx, value: BitVector<64>) -> BitVector<64> {
    sail_ctx.mtvec = legalize_tvec(sail_ctx, sail_ctx.mtvec, value);
    sail_ctx.mtvec.bits
}

fn set_stvec(sail_ctx: &mut SailVirtCtx, value: BitVector<64>) -> BitVector<64> {
    sail_ctx.stvec = legalize_tvec(sail_ctx, sail_ctx.stvec, value);
    sail_ctx.stvec.bits
}

fn csr_name_map_forwards(sail_ctx: &mut SailVirtCtx, arg_hashtag_: BitVector<12>) -> String {
    match arg_hashtag_ {
        b__0 if { (b__0 == BitVector::<12>::new(0b000000000000)) } => String::from("ustatus"),
        b__1 if { (b__1 == BitVector::<12>::new(0b000000000100)) } => String::from("uie"),
        b__2 if { (b__2 == BitVector::<12>::new(0b000000000101)) } => String::from("utvec"),
        b__3 if { (b__3 == BitVector::<12>::new(0b000001000000)) } => String::from("uscratch"),
        b__4 if { (b__4 == BitVector::<12>::new(0b000001000001)) } => String::from("uepc"),
        b__5 if { (b__5 == BitVector::<12>::new(0b000001000010)) } => String::from("ucause"),
        b__6 if { (b__6 == BitVector::<12>::new(0b000001000011)) } => String::from("utval"),
        b__7 if { (b__7 == BitVector::<12>::new(0b000001000100)) } => String::from("uip"),
        b__8 if { (b__8 == BitVector::<12>::new(0b000000000001)) } => String::from("fflags"),
        b__9 if { (b__9 == BitVector::<12>::new(0b000000000010)) } => String::from("frm"),
        b__10 if { (b__10 == BitVector::<12>::new(0b000000000011)) } => String::from("fcsr"),
        b__11 if { (b__11 == BitVector::<12>::new(0b000000010101)) } => String::from("seed"),
        b__12 if { (b__12 == BitVector::<12>::new(0b110000000010)) } => String::from("cycle"),
        b__13 if { (b__13 == BitVector::<12>::new(0b110000000001)) } => String::from("time"),
        b__14 if { (b__14 == BitVector::<12>::new(0b110000000010)) } => String::from("instret"),
        b__15 if { (b__15 == BitVector::<12>::new(0b110010000000)) } => String::from("cycleh"),
        b__16 if { (b__16 == BitVector::<12>::new(0b110010000001)) } => String::from("timeh"),
        b__17 if { (b__17 == BitVector::<12>::new(0b110010000010)) } => String::from("instreth"),
        b__18 if { (b__18 == BitVector::<12>::new(0b000100000000)) } => String::from("sstatus"),
        b__19 if { (b__19 == BitVector::<12>::new(0b000100000010)) } => String::from("sedeleg"),
        b__20 if { (b__20 == BitVector::<12>::new(0b000100000011)) } => String::from("sideleg"),
        b__21 if { (b__21 == BitVector::<12>::new(0b000100000100)) } => String::from("sie"),
        b__22 if { (b__22 == BitVector::<12>::new(0b000100000101)) } => String::from("stvec"),
        b__23 if { (b__23 == BitVector::<12>::new(0b000100000110)) } => String::from("scounteren"),
        b__24 if { (b__24 == BitVector::<12>::new(0b000101000000)) } => String::from("sscratch"),
        b__25 if { (b__25 == BitVector::<12>::new(0b000101000001)) } => String::from("sepc"),
        b__26 if { (b__26 == BitVector::<12>::new(0b000101000010)) } => String::from("scause"),
        b__27 if { (b__27 == BitVector::<12>::new(0b000101000011)) } => String::from("stval"),
        b__28 if { (b__28 == BitVector::<12>::new(0b000101000100)) } => String::from("sip"),
        b__29 if { (b__29 == BitVector::<12>::new(0b000110000000)) } => String::from("satp"),
        b__30 if { (b__30 == BitVector::<12>::new(0b000100001010)) } => String::from("senvcfg"),
        b__31 if { (b__31 == BitVector::<12>::new(0b111100010001)) } => String::from("mvendorid"),
        b__32 if { (b__32 == BitVector::<12>::new(0b111100010010)) } => String::from("marchid"),
        b__33 if { (b__33 == BitVector::<12>::new(0b111100010011)) } => String::from("mimpid"),
        b__34 if { (b__34 == BitVector::<12>::new(0b111100010100)) } => String::from("mhartid"),
        b__35 if { (b__35 == BitVector::<12>::new(0b111100010101)) } => String::from("mconfigptr"),
        b__36 if { (b__36 == BitVector::<12>::new(0b001100000000)) } => String::from("mstatus"),
        b__37 if { (b__37 == BitVector::<12>::new(0b001100000001)) } => String::from("misa"),
        b__38 if { (b__38 == BitVector::<12>::new(0b001100000010)) } => String::from("medeleg"),
        b__39 if { (b__39 == BitVector::<12>::new(0b001100000011)) } => String::from("mideleg"),
        b__40 if { (b__40 == BitVector::<12>::new(0b001100000100)) } => String::from("mie"),
        b__41 if { (b__41 == BitVector::<12>::new(0b001100000101)) } => String::from("mtvec"),
        b__42 if { (b__42 == BitVector::<12>::new(0b001100000110)) } => String::from("mcounteren"),
        b__43 if { (b__43 == BitVector::<12>::new(0b001100100000)) } => {
            String::from("mcountinhibit")
        }
        b__44 if { (b__44 == BitVector::<12>::new(0b001100001010)) } => String::from("menvcfg"),
        b__45 if { (b__45 == BitVector::<12>::new(0b001101000000)) } => String::from("mscratch"),
        b__46 if { (b__46 == BitVector::<12>::new(0b001101000001)) } => String::from("mepc"),
        b__47 if { (b__47 == BitVector::<12>::new(0b001101000010)) } => String::from("mcause"),
        b__48 if { (b__48 == BitVector::<12>::new(0b001101000011)) } => String::from("mtval"),
        b__49 if { (b__49 == BitVector::<12>::new(0b001101000100)) } => String::from("mip"),
        b__50 if { (b__50 == BitVector::<12>::new(0b001110100000)) } => String::from("pmpcfg0"),
        b__51 if { (b__51 == BitVector::<12>::new(0b001110100001)) } => String::from("pmpcfg1"),
        b__52 if { (b__52 == BitVector::<12>::new(0b001110100010)) } => String::from("pmpcfg2"),
        b__53 if { (b__53 == BitVector::<12>::new(0b001110100011)) } => String::from("pmpcfg3"),
        b__54 if { (b__54 == BitVector::<12>::new(0b001110100100)) } => String::from("pmpcfg4"),
        b__55 if { (b__55 == BitVector::<12>::new(0b001110100101)) } => String::from("pmpcfg5"),
        b__56 if { (b__56 == BitVector::<12>::new(0b001110100110)) } => String::from("pmpcfg6"),
        b__57 if { (b__57 == BitVector::<12>::new(0b001110100111)) } => String::from("pmpcfg7"),
        b__58 if { (b__58 == BitVector::<12>::new(0b001110101000)) } => String::from("pmpcfg8"),
        b__59 if { (b__59 == BitVector::<12>::new(0b001110101001)) } => String::from("pmpcfg9"),
        b__60 if { (b__60 == BitVector::<12>::new(0b001110101010)) } => String::from("pmpcfg10"),
        b__61 if { (b__61 == BitVector::<12>::new(0b001110101011)) } => String::from("pmpcfg11"),
        b__62 if { (b__62 == BitVector::<12>::new(0b001110101100)) } => String::from("pmpcfg12"),
        b__63 if { (b__63 == BitVector::<12>::new(0b001110101101)) } => String::from("pmpcfg13"),
        b__64 if { (b__64 == BitVector::<12>::new(0b001110101110)) } => String::from("pmpcfg14"),
        b__65 if { (b__65 == BitVector::<12>::new(0b001110101111)) } => String::from("pmpcfg15"),
        b__66 if { (b__66 == BitVector::<12>::new(0b001110110000)) } => String::from("pmpaddr0"),
        b__67 if { (b__67 == BitVector::<12>::new(0b001110110001)) } => String::from("pmpaddr1"),
        b__68 if { (b__68 == BitVector::<12>::new(0b001110110010)) } => String::from("pmpaddr2"),
        b__69 if { (b__69 == BitVector::<12>::new(0b001110110011)) } => String::from("pmpaddr3"),
        b__70 if { (b__70 == BitVector::<12>::new(0b001110110100)) } => String::from("pmpaddr4"),
        b__71 if { (b__71 == BitVector::<12>::new(0b001110110101)) } => String::from("pmpaddr5"),
        b__72 if { (b__72 == BitVector::<12>::new(0b001110110110)) } => String::from("pmpaddr6"),
        b__73 if { (b__73 == BitVector::<12>::new(0b001110110111)) } => String::from("pmpaddr7"),
        b__74 if { (b__74 == BitVector::<12>::new(0b001110111000)) } => String::from("pmpaddr8"),
        b__75 if { (b__75 == BitVector::<12>::new(0b001110111001)) } => String::from("pmpaddr9"),
        b__76 if { (b__76 == BitVector::<12>::new(0b001110111010)) } => String::from("pmpaddr10"),
        b__77 if { (b__77 == BitVector::<12>::new(0b001110111011)) } => String::from("pmpaddr11"),
        b__78 if { (b__78 == BitVector::<12>::new(0b001110111100)) } => String::from("pmpaddr12"),
        b__79 if { (b__79 == BitVector::<12>::new(0b001110111101)) } => String::from("pmpaddr13"),
        b__80 if { (b__80 == BitVector::<12>::new(0b001110111110)) } => String::from("pmpaddr14"),
        b__81 if { (b__81 == BitVector::<12>::new(0b001110111111)) } => String::from("pmpaddr15"),
        b__82 if { (b__82 == BitVector::<12>::new(0b001111000000)) } => String::from("pmpaddr16"),
        b__83 if { (b__83 == BitVector::<12>::new(0b001111000001)) } => String::from("pmpaddr17"),
        b__84 if { (b__84 == BitVector::<12>::new(0b001111000010)) } => String::from("pmpaddr18"),
        b__85 if { (b__85 == BitVector::<12>::new(0b001111000011)) } => String::from("pmpaddr19"),
        b__86 if { (b__86 == BitVector::<12>::new(0b001111000100)) } => String::from("pmpaddr20"),
        b__87 if { (b__87 == BitVector::<12>::new(0b001111000101)) } => String::from("pmpaddr21"),
        b__88 if { (b__88 == BitVector::<12>::new(0b001111000110)) } => String::from("pmpaddr22"),
        b__89 if { (b__89 == BitVector::<12>::new(0b001111000111)) } => String::from("pmpaddr23"),
        b__90 if { (b__90 == BitVector::<12>::new(0b001111001000)) } => String::from("pmpaddr24"),
        b__91 if { (b__91 == BitVector::<12>::new(0b001111001001)) } => String::from("pmpaddr25"),
        b__92 if { (b__92 == BitVector::<12>::new(0b001111001010)) } => String::from("pmpaddr26"),
        b__93 if { (b__93 == BitVector::<12>::new(0b001111001011)) } => String::from("pmpaddr27"),
        b__94 if { (b__94 == BitVector::<12>::new(0b001111001100)) } => String::from("pmpaddr28"),
        b__95 if { (b__95 == BitVector::<12>::new(0b001111001101)) } => String::from("pmpaddr29"),
        b__96 if { (b__96 == BitVector::<12>::new(0b001111001110)) } => String::from("pmpaddr30"),
        b__97 if { (b__97 == BitVector::<12>::new(0b001111001111)) } => String::from("pmpaddr31"),
        b__98 if { (b__98 == BitVector::<12>::new(0b001111010000)) } => String::from("pmpaddr32"),
        b__99 if { (b__99 == BitVector::<12>::new(0b001111010001)) } => String::from("pmpaddr33"),
        b__100 if { (b__100 == BitVector::<12>::new(0b001111010010)) } => String::from("pmpaddr34"),
        b__101 if { (b__101 == BitVector::<12>::new(0b001111010011)) } => String::from("pmpaddr35"),
        b__102 if { (b__102 == BitVector::<12>::new(0b001111010100)) } => String::from("pmpaddr36"),
        b__103 if { (b__103 == BitVector::<12>::new(0b001111010101)) } => String::from("pmpaddr37"),
        b__104 if { (b__104 == BitVector::<12>::new(0b001111010110)) } => String::from("pmpaddr38"),
        b__105 if { (b__105 == BitVector::<12>::new(0b001111010111)) } => String::from("pmpaddr39"),
        b__106 if { (b__106 == BitVector::<12>::new(0b001111011000)) } => String::from("pmpaddr40"),
        b__107 if { (b__107 == BitVector::<12>::new(0b001111011001)) } => String::from("pmpaddr41"),
        b__108 if { (b__108 == BitVector::<12>::new(0b001111011010)) } => String::from("pmpaddr42"),
        b__109 if { (b__109 == BitVector::<12>::new(0b001111011011)) } => String::from("pmpaddr43"),
        b__110 if { (b__110 == BitVector::<12>::new(0b001111011100)) } => String::from("pmpaddr44"),
        b__111 if { (b__111 == BitVector::<12>::new(0b001111011101)) } => String::from("pmpaddr45"),
        b__112 if { (b__112 == BitVector::<12>::new(0b001111011110)) } => String::from("pmpaddr46"),
        b__113 if { (b__113 == BitVector::<12>::new(0b001111011111)) } => String::from("pmpaddr47"),
        b__114 if { (b__114 == BitVector::<12>::new(0b001111100000)) } => String::from("pmpaddr48"),
        b__115 if { (b__115 == BitVector::<12>::new(0b001111100001)) } => String::from("pmpaddr49"),
        b__116 if { (b__116 == BitVector::<12>::new(0b001111100010)) } => String::from("pmpaddr50"),
        b__117 if { (b__117 == BitVector::<12>::new(0b001111100011)) } => String::from("pmpaddr51"),
        b__118 if { (b__118 == BitVector::<12>::new(0b001111100100)) } => String::from("pmpaddr52"),
        b__119 if { (b__119 == BitVector::<12>::new(0b001111100101)) } => String::from("pmpaddr53"),
        b__120 if { (b__120 == BitVector::<12>::new(0b001111100110)) } => String::from("pmpaddr54"),
        b__121 if { (b__121 == BitVector::<12>::new(0b001111100111)) } => String::from("pmpaddr55"),
        b__122 if { (b__122 == BitVector::<12>::new(0b001111101000)) } => String::from("pmpaddr56"),
        b__123 if { (b__123 == BitVector::<12>::new(0b001111101001)) } => String::from("pmpaddr57"),
        b__124 if { (b__124 == BitVector::<12>::new(0b001111101010)) } => String::from("pmpaddr58"),
        b__125 if { (b__125 == BitVector::<12>::new(0b001111101011)) } => String::from("pmpaddr59"),
        b__126 if { (b__126 == BitVector::<12>::new(0b001111101100)) } => String::from("pmpaddr60"),
        b__127 if { (b__127 == BitVector::<12>::new(0b001111101101)) } => String::from("pmpaddr61"),
        b__128 if { (b__128 == BitVector::<12>::new(0b001111101110)) } => String::from("pmpaddr62"),
        b__129 if { (b__129 == BitVector::<12>::new(0b001111101111)) } => String::from("pmpaddr63"),
        b__130 if { (b__130 == BitVector::<12>::new(0b101100000000)) } => String::from("mcycle"),
        b__131 if { (b__131 == BitVector::<12>::new(0b101100000010)) } => String::from("minstret"),
        b__132 if { (b__132 == BitVector::<12>::new(0b101110000000)) } => String::from("mcycleh"),
        b__133 if { (b__133 == BitVector::<12>::new(0b101110000010)) } => String::from("minstreth"),
        b__134 if { (b__134 == BitVector::<12>::new(0b011110100000)) } => String::from("tselect"),
        b__135 if { (b__135 == BitVector::<12>::new(0b011110100001)) } => String::from("tdata1"),
        b__136 if { (b__136 == BitVector::<12>::new(0b011110100010)) } => String::from("tdata2"),
        b__137 if { (b__137 == BitVector::<12>::new(0b011110100011)) } => String::from("tdata3"),
        b__138 if { (b__138 == BitVector::<12>::new(0b000000001000)) } => String::from("vstart"),
        b__139 if { (b__139 == BitVector::<12>::new(0b000000001001)) } => String::from("vxsat"),
        b__140 if { (b__140 == BitVector::<12>::new(0b000000001010)) } => String::from("vxrm"),
        b__141 if { (b__141 == BitVector::<12>::new(0b000000001111)) } => String::from("vcsr"),
        b__142 if { (b__142 == BitVector::<12>::new(0b110000100000)) } => String::from("vl"),
        b__143 if { (b__143 == BitVector::<12>::new(0b110000100001)) } => String::from("vtype"),
        b__144 if { (b__144 == BitVector::<12>::new(0b110000100010)) } => String::from("vlenb"),
        reg => hex_bits_12_forwards(reg),
    }
}

fn csr_name(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>) -> String {
    csr_name_map_forwards(sail_ctx, csr)
}

fn ext_is_CSR_defined(sail_ctx: &mut SailVirtCtx, _: BitVector<12>, _: Privilege) -> bool {
    false
}

fn ext_read_CSR(sail_ctx: &mut SailVirtCtx, _: BitVector<12>) -> Option<BitVector<64>> {
    None
}

fn ext_write_CSR(
    sail_ctx: &mut SailVirtCtx,
    _: BitVector<12>,
    _: BitVector<64>,
) -> Option<BitVector<64>> {
    None
}

fn csrAccess(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>) -> BitVector<2> {
    csr.subrange::<10, 12, 2>()
}

fn csrPriv(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>) -> BitVector<2> {
    csr.subrange::<8, 10, 2>()
}

fn is_CSR_defined(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>, p: Privilege) -> bool {
    match csr {
        b__0 if { (b__0 == BitVector::<12>::new(0b111100010001)) } => (p == Privilege::Machine),
        b__1 if { (b__1 == BitVector::<12>::new(0b111100010010)) } => (p == Privilege::Machine),
        b__2 if { (b__2 == BitVector::<12>::new(0b111100010011)) } => (p == Privilege::Machine),
        b__3 if { (b__3 == BitVector::<12>::new(0b111100010100)) } => (p == Privilege::Machine),
        b__4 if { (b__4 == BitVector::<12>::new(0b111100010101)) } => (p == Privilege::Machine),
        b__5 if { (b__5 == BitVector::<12>::new(0b001100000000)) } => (p == Privilege::Machine),
        b__6 if { (b__6 == BitVector::<12>::new(0b001100000001)) } => (p == Privilege::Machine),
        b__7 if { (b__7 == BitVector::<12>::new(0b001100000010)) } => {
            ((p == Privilege::Machine) && (haveSupMode(sail_ctx, ()) || haveNExt(sail_ctx, ())))
        }
        b__8 if { (b__8 == BitVector::<12>::new(0b001100000011)) } => {
            ((p == Privilege::Machine) && (haveSupMode(sail_ctx, ()) || haveNExt(sail_ctx, ())))
        }
        b__9 if { (b__9 == BitVector::<12>::new(0b001100000100)) } => (p == Privilege::Machine),
        b__10 if { (b__10 == BitVector::<12>::new(0b001100000101)) } => (p == Privilege::Machine),
        b__11 if { (b__11 == BitVector::<12>::new(0b001100000110)) } => {
            ((p == Privilege::Machine) && haveUsrMode(sail_ctx, ()))
        }
        b__12 if { (b__12 == BitVector::<12>::new(0b001100001010)) } => {
            ((p == Privilege::Machine) && haveUsrMode(sail_ctx, ()))
        }
        b__13 if { (b__13 == BitVector::<12>::new(0b001100010000)) } => {
            ((p == Privilege::Machine) && (64 == 32))
        }
        b__14 if { (b__14 == BitVector::<12>::new(0b001100011010)) } => {
            ((p == Privilege::Machine) && (haveUsrMode(sail_ctx, ()) && (64 == 32)))
        }
        b__15 if { (b__15 == BitVector::<12>::new(0b001100100000)) } => (p == Privilege::Machine),
        b__16 if { (b__16 == BitVector::<12>::new(0b001101000000)) } => (p == Privilege::Machine),
        b__17 if { (b__17 == BitVector::<12>::new(0b001101000001)) } => (p == Privilege::Machine),
        b__18 if { (b__18 == BitVector::<12>::new(0b001101000010)) } => (p == Privilege::Machine),
        b__19 if { (b__19 == BitVector::<12>::new(0b001101000011)) } => (p == Privilege::Machine),
        b__20 if { (b__20 == BitVector::<12>::new(0b001101000100)) } => (p == Privilege::Machine),
        v__0 if { (v__0.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111010)) } => {
            let idx: BitVector<4> = v__0.subrange::<0, 4, 4>();
            ((p == Privilege::Machine)
                && ((gt_int(sys_pmp_count(()), idx.as_usize())
                    && ((bitvector_access(idx, 0) == false) || (64 == 32)))
                    as bool))
        }
        v__2 if { (v__2.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111011)) } => {
            let idx: BitVector<4> = v__2.subrange::<0, 4, 4>();
            ((p == Privilege::Machine)
                && (gt_int(
                    sys_pmp_count(()),
                    bitvector_concat(BitVector::<2>::new(0b00), idx).as_usize(),
                ) as bool))
        }
        v__4 if { (v__4.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111100)) } => {
            let idx: BitVector<4> = v__4.subrange::<0, 4, 4>();
            ((p == Privilege::Machine)
                && (gt_int(
                    sys_pmp_count(()),
                    bitvector_concat(BitVector::<2>::new(0b01), idx).as_usize(),
                ) as bool))
        }
        v__6 if { (v__6.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111101)) } => {
            let idx: BitVector<4> = v__6.subrange::<0, 4, 4>();
            ((p == Privilege::Machine)
                && (gt_int(
                    sys_pmp_count(()),
                    bitvector_concat(BitVector::<2>::new(0b10), idx).as_usize(),
                ) as bool))
        }
        v__8 if { (v__8.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111110)) } => {
            let idx: BitVector<4> = v__8.subrange::<0, 4, 4>();
            ((p == Privilege::Machine)
                && (gt_int(
                    sys_pmp_count(()),
                    bitvector_concat(BitVector::<2>::new(0b11), idx).as_usize(),
                ) as bool))
        }
        b__21 if { (b__21 == BitVector::<12>::new(0b101100000000)) } => (p == Privilege::Machine),
        b__22 if { (b__22 == BitVector::<12>::new(0b101100000010)) } => (p == Privilege::Machine),
        b__23 if { (b__23 == BitVector::<12>::new(0b101110000000)) } => {
            ((p == Privilege::Machine) && (64 == 32))
        }
        b__24 if { (b__24 == BitVector::<12>::new(0b101110000010)) } => {
            ((p == Privilege::Machine) && (64 == 32))
        }
        b__25 if { (b__25 == BitVector::<12>::new(0b011110100000)) } => (p == Privilege::Machine),
        b__26 if { (b__26 == BitVector::<12>::new(0b000100000000)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__27 if { (b__27 == BitVector::<12>::new(0b000100000010)) } => {
            (haveSupMode(sail_ctx, ())
                && (haveNExt(sail_ctx, ())
                    && ((p == Privilege::Machine) || (p == Privilege::Supervisor))))
        }
        b__28 if { (b__28 == BitVector::<12>::new(0b000100000011)) } => {
            (haveSupMode(sail_ctx, ())
                && (haveNExt(sail_ctx, ())
                    && ((p == Privilege::Machine) || (p == Privilege::Supervisor))))
        }
        b__29 if { (b__29 == BitVector::<12>::new(0b000100000100)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__30 if { (b__30 == BitVector::<12>::new(0b000100000101)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__31 if { (b__31 == BitVector::<12>::new(0b000100000110)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__32 if { (b__32 == BitVector::<12>::new(0b000100001010)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__33 if { (b__33 == BitVector::<12>::new(0b000101000000)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__34 if { (b__34 == BitVector::<12>::new(0b000101000001)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__35 if { (b__35 == BitVector::<12>::new(0b000101000010)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__36 if { (b__36 == BitVector::<12>::new(0b000101000011)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__37 if { (b__37 == BitVector::<12>::new(0b000101000100)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__38 if { (b__38 == BitVector::<12>::new(0b000110000000)) } => {
            (haveSupMode(sail_ctx, ())
                && ((p == Privilege::Machine) || (p == Privilege::Supervisor)))
        }
        b__39 if { (b__39 == BitVector::<12>::new(0b110000000000)) } => haveUsrMode(sail_ctx, ()),
        b__40 if { (b__40 == BitVector::<12>::new(0b110000000001)) } => haveUsrMode(sail_ctx, ()),
        b__41 if { (b__41 == BitVector::<12>::new(0b110000000010)) } => haveUsrMode(sail_ctx, ()),
        b__42 if { (b__42 == BitVector::<12>::new(0b110010000000)) } => {
            (haveUsrMode(sail_ctx, ()) && (64 == 32))
        }
        b__43 if { (b__43 == BitVector::<12>::new(0b110010000001)) } => {
            (haveUsrMode(sail_ctx, ()) && (64 == 32))
        }
        b__44 if { (b__44 == BitVector::<12>::new(0b110010000010)) } => {
            (haveUsrMode(sail_ctx, ()) && (64 == 32))
        }
        b__45 if { (b__45 == BitVector::<12>::new(0b000000010101)) } => haveZkr(sail_ctx, ()),
        _ => ext_is_CSR_defined(sail_ctx, csr, p),
    }
}

fn check_CSR_access(
    sail_ctx: &mut SailVirtCtx,
    csrrw: BitVector<2>,
    csrpr: BitVector<2>,
    p: Privilege,
    isWrite: bool,
) -> bool {
    (!((isWrite == true) && (csrrw == BitVector::<2>::new(0b11))) && {
        let var_236 = privLevel_to_bits(sail_ctx, p);
        let var_237 = csrpr;
        _operator_biggerequal_u_(sail_ctx, var_236, var_237)
    })
}

fn check_TVM_SATP(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>, p: Privilege) -> bool {
    !((csr == BitVector::<12>::new(0b000110000000))
        && ((p == Privilege::Supervisor)
            && (_get_Mstatus_TVM(sail_ctx, sail_ctx.mstatus) == BitVector::<1>::new(0b1))))
}

fn check_Counteren(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>, p: Privilege) -> bool {
    match (csr, p) {
        (b__0, Privilege::Supervisor) if { (b__0 == BitVector::<12>::new(0b110000000000)) } => {
            (_get_Counteren_CY(sail_ctx, sail_ctx.mcounteren) == BitVector::<1>::new(0b1))
        }
        (b__1, Privilege::Supervisor) if { (b__1 == BitVector::<12>::new(0b110000000001)) } => {
            (_get_Counteren_TM(sail_ctx, sail_ctx.mcounteren) == BitVector::<1>::new(0b1))
        }
        (b__2, Privilege::Supervisor) if { (b__2 == BitVector::<12>::new(0b110000000010)) } => {
            (_get_Counteren_IR(sail_ctx, sail_ctx.mcounteren) == BitVector::<1>::new(0b1))
        }
        (b__3, Privilege::User) if { (b__3 == BitVector::<12>::new(0b110000000000)) } => {
            ((_get_Counteren_CY(sail_ctx, sail_ctx.mcounteren) == BitVector::<1>::new(0b1))
                && (!(haveSupMode(sail_ctx, ()))
                    || (_get_Counteren_CY(sail_ctx, sail_ctx.scounteren)
                        == BitVector::<1>::new(0b1))))
        }
        (b__4, Privilege::User) if { (b__4 == BitVector::<12>::new(0b110000000001)) } => {
            ((_get_Counteren_TM(sail_ctx, sail_ctx.mcounteren) == BitVector::<1>::new(0b1))
                && (!(haveSupMode(sail_ctx, ()))
                    || (_get_Counteren_TM(sail_ctx, sail_ctx.scounteren)
                        == BitVector::<1>::new(0b1))))
        }
        (b__5, Privilege::User) if { (b__5 == BitVector::<12>::new(0b110000000010)) } => {
            ((_get_Counteren_IR(sail_ctx, sail_ctx.mcounteren) == BitVector::<1>::new(0b1))
                && (!(haveSupMode(sail_ctx, ()))
                    || (_get_Counteren_IR(sail_ctx, sail_ctx.scounteren)
                        == BitVector::<1>::new(0b1))))
        }
        (_, _) => {
            if {
                (_operator_smallerequal_u_(sail_ctx, BitVector::<12>::new(0b110000000011), csr)
                    && _operator_smallerequal_u_(
                        sail_ctx,
                        csr,
                        BitVector::<12>::new(0b110000011111),
                    ))
            } {
                false
            } else {
                true
            }
        }
    }
}

fn check_seed_CSR(
    sail_ctx: &mut SailVirtCtx,
    csr: BitVector<12>,
    p: Privilege,
    isWrite: bool,
) -> bool {
    if { !(csr == BitVector::<12>::new(0b000000010101)) } {
        true
    } else if { !(isWrite) } {
        false
    } else {
        match p {
            Privilege::Machine => true,
            Privilege::Supervisor => false,
            Privilege::User => false,
        }
    }
}

fn check_CSR(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>, p: Privilege, isWrite: bool) -> bool {
    (is_CSR_defined(sail_ctx, csr, p)
        && ({
            let var_238 = csrAccess(sail_ctx, csr);
            let var_239 = csrPriv(sail_ctx, csr);
            let var_240 = p;
            let var_241 = isWrite;
            check_CSR_access(sail_ctx, var_238, var_239, var_240, var_241)
        } && (check_TVM_SATP(sail_ctx, csr, p)
            && (check_Counteren(sail_ctx, csr, p) && check_seed_CSR(sail_ctx, csr, p, isWrite)))))
}

fn exception_delegatee(sail_ctx: &mut SailVirtCtx, e: ExceptionType, p: Privilege) -> Privilege {
    let idx = num_of_ExceptionType(sail_ctx, e);
    let _super_ = {
        let var_245 = bitvector_access(sail_ctx.medeleg.bits, idx);
        bit_to_bool(sail_ctx, var_245)
    };
    let user = if { haveSupMode(sail_ctx, ()) } {
        (_super_
            && (haveNExt(sail_ctx, ()) && {
                let var_244 = bitvector_access(sail_ctx.sedeleg.bits, idx);
                bit_to_bool(sail_ctx, var_244)
            }))
    } else {
        (_super_ && haveNExt(sail_ctx, ()))
    };
    let deleg = if { (haveUsrMode(sail_ctx, ()) && user) } {
        Privilege::User
    } else if { (haveSupMode(sail_ctx, ()) && _super_) } {
        Privilege::Supervisor
    } else {
        Privilege::Machine
    };
    if {
        {
            let var_242 = privLevel_to_bits(sail_ctx, deleg);
            let var_243 = privLevel_to_bits(sail_ctx, p);
            _operator_smaller_u_(sail_ctx, var_242, var_243)
        }
    } {
        p
    } else {
        deleg
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum interrupt_set {
    Ints_Pending(xlenbits),
    Ints_Delegated(xlenbits),
    Ints_Empty(()),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum ctl_result {
    CTL_TRAP(sync_exception),
    CTL_SRET(()),
    CTL_MRET(()),
    CTL_URET(()),
}

fn tval(sail_ctx: &mut SailVirtCtx, excinfo: Option<BitVector<64>>) -> BitVector<64> {
    match excinfo {
        Some(e) => e,
        None => zero_extend_64(BitVector::<1>::new(0b0)),
    }
}

fn rvfi_trap(sail_ctx: &mut SailVirtCtx, unit_arg: ()) {
    ()
}

fn trap_handler(
    sail_ctx: &mut SailVirtCtx,
    del_priv: Privilege,
    intr: bool,
    c: BitVector<8>,
    pc: BitVector<64>,
    info: Option<BitVector<64>>,
    ext: Option<()>,
) -> BitVector<64> {
    rvfi_trap(sail_ctx, ());
    if { get_config_print_platform(sail_ctx, ()) } {
        print_platform(format!(
            "{}{}",
            String::from("handling "),
            format!(
                "{}{}",
                if { intr } {
                    String::from("int_hashtag_")
                } else {
                    String::from("exc_hashtag_")
                },
                format!(
                    "{}{}",
                    bits_str(c),
                    format!(
                        "{}{}",
                        String::from(" at priv "),
                        format!(
                            "{}{}",
                            privLevel_to_str(sail_ctx, del_priv),
                            format!(
                                "{}{}",
                                String::from(" with tval "),
                                bits_str(tval(sail_ctx, info))
                            )
                        )
                    )
                )
            )
        ))
    } else {
        ()
    };
    cancel_reservation(());
    match del_priv {
        Privilege::Machine => {
            sail_ctx.mcause = {
                let var_246 = bool_to_bits(sail_ctx, intr);
                sail_ctx.mcause.set_subrange::<63, 64, 1>(var_246)
            };
            sail_ctx.mcause = {
                let var_247 = zero_extend_63(c);
                sail_ctx.mcause.set_subrange::<0, 63, 63>(var_247)
            };
            sail_ctx.mstatus = {
                let var_248 = _get_Mstatus_MIE(sail_ctx, sail_ctx.mstatus);
                sail_ctx.mstatus.set_subrange::<7, 8, 1>(var_248)
            };
            sail_ctx.mstatus = sail_ctx
                .mstatus
                .set_subrange::<3, 4, 1>(BitVector::<1>::new(0b0));
            sail_ctx.mstatus = {
                let var_249 = privLevel_to_bits(sail_ctx, sail_ctx.cur_privilege);
                sail_ctx.mstatus.set_subrange::<11, 13, 2>(var_249)
            };
            sail_ctx.mtval = tval(sail_ctx, info);
            sail_ctx.mepc = pc;
            sail_ctx.cur_privilege = del_priv;
            handle_trap_extension(sail_ctx, del_priv, pc, ext);
            if { get_config_print_reg(sail_ctx, ()) } {
                print_reg(format!(
                    "{}{}",
                    String::from("CSR mstatus <- "),
                    bits_str(sail_ctx.mstatus.bits)
                ))
            } else {
                ()
            };
            prepare_trap_vector(sail_ctx, del_priv, sail_ctx.mcause)
        }
        Privilege::Supervisor => {
            assert!(false, "todo_process_message");
            sail_ctx.scause = {
                let var_250 = bool_to_bits(sail_ctx, intr);
                sail_ctx.scause.set_subrange::<63, 64, 1>(var_250)
            };
            sail_ctx.scause = {
                let var_251 = zero_extend_63(c);
                sail_ctx.scause.set_subrange::<0, 63, 63>(var_251)
            };
            sail_ctx.mstatus = {
                let var_252 = _get_Mstatus_SIE(sail_ctx, sail_ctx.mstatus);
                sail_ctx.mstatus.set_subrange::<5, 6, 1>(var_252)
            };
            sail_ctx.mstatus = sail_ctx
                .mstatus
                .set_subrange::<1, 2, 1>(BitVector::<1>::new(0b0));
            sail_ctx.mstatus =
                sail_ctx
                    .mstatus
                    .set_subrange::<8, 9, 1>(match sail_ctx.cur_privilege {
                        Privilege::User => BitVector::<1>::new(0b0),
                        Privilege::Supervisor => BitVector::<1>::new(0b1),
                        Privilege::Machine => internal_error(
                            String::from("../sail-riscv/model/riscv_sys_control.sail"),
                            359,
                            String::from("invalid privilege for s-mode trap"),
                        ),
                    });
            sail_ctx.stval = tval(sail_ctx, info);
            sail_ctx.sepc = pc;
            sail_ctx.cur_privilege = del_priv;
            handle_trap_extension(sail_ctx, del_priv, pc, ext);
            if { get_config_print_reg(sail_ctx, ()) } {
                print_reg(format!(
                    "{}{}",
                    String::from("CSR mstatus <- "),
                    bits_str(sail_ctx.mstatus.bits)
                ))
            } else {
                ()
            };
            prepare_trap_vector(sail_ctx, del_priv, sail_ctx.scause)
        }
        Privilege::User => {
            assert!(false, "todo_process_message");
            sail_ctx.ucause = {
                let var_253 = bool_to_bits(sail_ctx, intr);
                sail_ctx.ucause.set_subrange::<63, 64, 1>(var_253)
            };
            sail_ctx.ucause = {
                let var_254 = zero_extend_63(c);
                sail_ctx.ucause.set_subrange::<0, 63, 63>(var_254)
            };
            sail_ctx.mstatus = {
                let var_255 = _get_Mstatus_UIE(sail_ctx, sail_ctx.mstatus);
                sail_ctx.mstatus.set_subrange::<4, 5, 1>(var_255)
            };
            sail_ctx.mstatus = sail_ctx
                .mstatus
                .set_subrange::<0, 1, 1>(BitVector::<1>::new(0b0));
            sail_ctx.utval = tval(sail_ctx, info);
            sail_ctx.uepc = pc;
            sail_ctx.cur_privilege = del_priv;
            handle_trap_extension(sail_ctx, del_priv, pc, ext);
            if { get_config_print_reg(sail_ctx, ()) } {
                print_reg(format!(
                    "{}{}",
                    String::from("CSR mstatus <- "),
                    bits_str(sail_ctx.mstatus.bits)
                ))
            } else {
                ()
            };
            prepare_trap_vector(sail_ctx, del_priv, sail_ctx.ucause)
        }
    }
}

fn exception_handler(
    sail_ctx: &mut SailVirtCtx,
    cur_priv: Privilege,
    ctl: ctl_result,
    pc: BitVector<64>,
) -> BitVector<64> {
    match (cur_priv, ctl) {
        (_, ctl_result::CTL_TRAP(e)) => {
            let del_priv = {
                let var_264 = e.trap;
                let var_265 = cur_priv;
                exception_delegatee(sail_ctx, var_264, var_265)
            };
            if { get_config_print_platform(sail_ctx, ()) } {
                print_platform(format!(
                    "{}{}",
                    String::from("trapping from "),
                    format!(
                        "{}{}",
                        privLevel_to_str(sail_ctx, cur_priv),
                        format!(
                            "{}{}",
                            String::from(" to "),
                            format!(
                                "{}{}",
                                privLevel_to_str(sail_ctx, del_priv),
                                format!("{}{}", String::from(" to handle "), {
                                    let var_256 = e.trap;
                                    exceptionType_to_str(sail_ctx, var_256)
                                })
                            )
                        )
                    )
                ))
            } else {
                ()
            };
            {
                let var_257 = del_priv;
                let var_258 = false;
                let var_259 = {
                    let var_263 = e.trap;
                    exceptionType_to_bits(sail_ctx, var_263)
                };
                let var_260 = pc;
                let var_261 = e.excinfo;
                let var_262 = e.ext;
                trap_handler(
                    sail_ctx, var_257, var_258, var_259, var_260, var_261, var_262,
                )
            }
        }
        (_, ctl_result::CTL_MRET(())) => {
            let prev_priv = sail_ctx.cur_privilege;
            sail_ctx.mstatus = {
                let var_266 = _get_Mstatus_MPIE(sail_ctx, sail_ctx.mstatus);
                sail_ctx.mstatus.set_subrange::<3, 4, 1>(var_266)
            };
            sail_ctx.mstatus = sail_ctx
                .mstatus
                .set_subrange::<7, 8, 1>(BitVector::<1>::new(0b1));
            sail_ctx.cur_privilege = {
                let var_267 = _get_Mstatus_MPP(sail_ctx, sail_ctx.mstatus);
                privLevel_of_bits(sail_ctx, var_267)
            };
            sail_ctx.mstatus = {
                let var_268 = {
                    let var_269 = if { haveUsrMode(sail_ctx, ()) } {
                        Privilege::User
                    } else {
                        Privilege::Machine
                    };
                    privLevel_to_bits(sail_ctx, var_269)
                };
                sail_ctx.mstatus.set_subrange::<11, 13, 2>(var_268)
            };
            if { (sail_ctx.cur_privilege != Privilege::Machine) } {
                sail_ctx.mstatus = sail_ctx
                    .mstatus
                    .set_subrange::<17, 18, 1>(BitVector::<1>::new(0b0))
            } else {
                ()
            };
            if { get_config_print_reg(sail_ctx, ()) } {
                print_reg(format!(
                    "{}{}",
                    String::from("CSR mstatus <- "),
                    bits_str(sail_ctx.mstatus.bits)
                ))
            } else {
                ()
            };
            if { get_config_print_platform(sail_ctx, ()) } {
                print_platform(format!(
                    "{}{}",
                    String::from("ret-ing from "),
                    format!(
                        "{}{}",
                        privLevel_to_str(sail_ctx, prev_priv),
                        format!(
                            "{}{}",
                            String::from(" to "),
                            privLevel_to_str(sail_ctx, sail_ctx.cur_privilege)
                        )
                    )
                ))
            } else {
                ()
            };
            cancel_reservation(());
            (prepare_xret_target(sail_ctx, Privilege::Machine) & pc_alignment_mask(sail_ctx, ()))
        }
        (_, ctl_result::CTL_SRET(())) => {
            let prev_priv = sail_ctx.cur_privilege;
            sail_ctx.mstatus = {
                let var_270 = _get_Mstatus_SPIE(sail_ctx, sail_ctx.mstatus);
                sail_ctx.mstatus.set_subrange::<1, 2, 1>(var_270)
            };
            sail_ctx.mstatus = sail_ctx
                .mstatus
                .set_subrange::<5, 6, 1>(BitVector::<1>::new(0b1));
            sail_ctx.cur_privilege =
                if { (_get_Mstatus_SPP(sail_ctx, sail_ctx.mstatus) == BitVector::<1>::new(0b1)) } {
                    Privilege::Supervisor
                } else {
                    Privilege::User
                };
            sail_ctx.mstatus = sail_ctx
                .mstatus
                .set_subrange::<8, 9, 1>(BitVector::<1>::new(0b0));
            if { (sail_ctx.cur_privilege != Privilege::Machine) } {
                sail_ctx.mstatus = sail_ctx
                    .mstatus
                    .set_subrange::<17, 18, 1>(BitVector::<1>::new(0b0))
            } else {
                ()
            };
            if { get_config_print_reg(sail_ctx, ()) } {
                print_reg(format!(
                    "{}{}",
                    String::from("CSR mstatus <- "),
                    bits_str(sail_ctx.mstatus.bits)
                ))
            } else {
                ()
            };
            if { get_config_print_platform(sail_ctx, ()) } {
                print_platform(format!(
                    "{}{}",
                    String::from("ret-ing from "),
                    format!(
                        "{}{}",
                        privLevel_to_str(sail_ctx, prev_priv),
                        format!(
                            "{}{}",
                            String::from(" to "),
                            privLevel_to_str(sail_ctx, sail_ctx.cur_privilege)
                        )
                    )
                ))
            } else {
                ()
            };
            cancel_reservation(());
            (prepare_xret_target(sail_ctx, Privilege::Supervisor) & pc_alignment_mask(sail_ctx, ()))
        }
        (_, ctl_result::CTL_URET(())) => {
            let prev_priv = sail_ctx.cur_privilege;
            sail_ctx.mstatus = {
                let var_271 = _get_Mstatus_UPIE(sail_ctx, sail_ctx.mstatus);
                sail_ctx.mstatus.set_subrange::<0, 1, 1>(var_271)
            };
            sail_ctx.mstatus = sail_ctx
                .mstatus
                .set_subrange::<4, 5, 1>(BitVector::<1>::new(0b1));
            sail_ctx.cur_privilege = Privilege::User;
            if { get_config_print_reg(sail_ctx, ()) } {
                print_reg(format!(
                    "{}{}",
                    String::from("CSR mstatus <- "),
                    bits_str(sail_ctx.mstatus.bits)
                ))
            } else {
                ()
            };
            if { get_config_print_platform(sail_ctx, ()) } {
                print_platform(format!(
                    "{}{}",
                    String::from("ret-ing from "),
                    format!(
                        "{}{}",
                        privLevel_to_str(sail_ctx, prev_priv),
                        format!(
                            "{}{}",
                            String::from(" to "),
                            privLevel_to_str(sail_ctx, sail_ctx.cur_privilege)
                        )
                    )
                ))
            } else {
                ()
            };
            cancel_reservation(());
            (prepare_xret_target(sail_ctx, Privilege::User) & pc_alignment_mask(sail_ctx, ()))
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum MemoryOpResult {
    MemValue(_tick_a),
    MemException(ExceptionType),
}

fn handle_illegal(sail_ctx: &mut SailVirtCtx, unit_arg: ()) {
    let info = if { plat_mtval_has_illegal_inst_bits(()) } {
        Some(sail_ctx.instbits)
    } else {
        None
    };
    let t: sync_exception = sync_exception {
        trap: ExceptionType::E_Illegal_Instr(()),
        excinfo: info,
        ext: None,
    };
    {
        let var_272 = {
            let var_273 = sail_ctx.cur_privilege;
            let var_274 = ctl_result::CTL_TRAP(t);
            let var_275 = sail_ctx.PC;
            exception_handler(sail_ctx, var_273, var_274, var_275)
        };
        set_next_pc(sail_ctx, var_272)
    }
}

fn platform_wfi(sail_ctx: &mut SailVirtCtx, unit_arg: ()) {
    cancel_reservation(());
    if { _operator_smaller_u_(sail_ctx, sail_ctx.mtime, sail_ctx.mtimecmp) } {
        sail_ctx.mtime = sail_ctx.mtimecmp;
        sail_ctx.mcycle = sail_ctx.mtimecmp
    } else {
        ()
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum PTE_Check {
    PTE_Check_Success(ext_ptw),
    PTE_Check_Failure((ext_ptw, ext_ptw_fail)),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum PTW_Error {
    PTW_Invalid_Addr(()),
    PTW_Access(()),
    PTW_Invalid_PTE(()),
    PTW_No_Permission(()),
    PTW_Misaligned(()),
    PTW_PTE_Update(()),
    PTW_Ext_Error(ext_ptw_error),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum PTW_Result {
    PTW_Success(
        (
            BitVector<64>,
            BitVector<64>,
            BitVector<64>,
            nat,
            bool,
            ext_ptw,
        ),
    ),
    PTW_Failure((PTW_Error, ext_ptw)),
}

fn legalize_satp(
    sail_ctx: &mut SailVirtCtx,
    a: Architecture,
    o: BitVector<64>,
    v: BitVector<64>,
) -> BitVector<64> {
    if { (64 == 32) } {
        panic!("unreachable code")
    } else if { (64 == 64) } {
        let o64: BitVector<64> = zero_extend_64(o);
        let v64: BitVector<64> = zero_extend_64(v);
        let new_satp: BitVector<64> = legalize_satp64(sail_ctx, a, o64, v64);
        truncate(new_satp, 64)
    } else {
        internal_error(
            String::from("../sail-riscv/model/riscv_vmem.sail"),
            205,
            format!("{}{}", String::from("Unsupported xlen"), dec_str(64)),
        )
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum TR_Result {
    TR_Address((_tick_paddr, ext_ptw)),
    TR_Failure((_tick_failure, ext_ptw)),
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum ast {
    ILLEGAL(word),
    C_ILLEGAL(half),
    UTYPE((BitVector<20>, regidx, uop)),
    RISCV_JAL((BitVector<21>, regidx)),
    RISCV_JALR((BitVector<12>, regidx, regidx)),
    BTYPE((BitVector<13>, regidx, regidx, bop)),
    ITYPE((BitVector<12>, regidx, regidx, iop)),
    SHIFTIOP((BitVector<6>, regidx, regidx, sop)),
    RTYPE((regidx, regidx, regidx, rop)),
    LOAD((BitVector<12>, regidx, regidx, bool, word_width, bool, bool)),
    STORE((BitVector<12>, regidx, regidx, word_width, bool, bool)),
    ADDIW((BitVector<12>, regidx, regidx)),
    RTYPEW((regidx, regidx, regidx, ropw)),
    SHIFTIWOP((BitVector<5>, regidx, regidx, sopw)),
    FENCE((BitVector<4>, BitVector<4>)),
    FENCE_TSO((BitVector<4>, BitVector<4>)),
    FENCEI(()),
    ECALL(()),
    MRET(()),
    SRET(()),
    EBREAK(()),
    WFI(()),
    SFENCE_VMA((regidx, regidx)),
    LOADRES((bool, bool, regidx, word_width, regidx)),
    STORECON((bool, bool, regidx, regidx, word_width, regidx)),
    AMO((amoop, bool, bool, regidx, regidx, word_width, regidx)),
    C_NOP(()),
    C_ADDI4SPN((cregidx, BitVector<8>)),
    C_LW((BitVector<5>, cregidx, cregidx)),
    C_LD((BitVector<5>, cregidx, cregidx)),
    C_SW((BitVector<5>, cregidx, cregidx)),
    C_SD((BitVector<5>, cregidx, cregidx)),
    C_ADDI((BitVector<6>, regidx)),
    C_JAL(BitVector<11>),
    C_ADDIW((BitVector<6>, regidx)),
    C_LI((BitVector<6>, regidx)),
    C_ADDI16SP(BitVector<6>),
    C_LUI((BitVector<6>, regidx)),
    C_SRLI((BitVector<6>, cregidx)),
    C_SRAI((BitVector<6>, cregidx)),
    C_ANDI((BitVector<6>, cregidx)),
    C_SUB((cregidx, cregidx)),
    C_XOR((cregidx, cregidx)),
    C_OR((cregidx, cregidx)),
    C_AND((cregidx, cregidx)),
    C_SUBW((cregidx, cregidx)),
    C_ADDW((cregidx, cregidx)),
    C_J(BitVector<11>),
    C_BEQZ((BitVector<8>, cregidx)),
    C_BNEZ((BitVector<8>, cregidx)),
    C_SLLI((BitVector<6>, regidx)),
    C_LWSP((BitVector<6>, regidx)),
    C_LDSP((BitVector<6>, regidx)),
    C_SWSP((BitVector<6>, regidx)),
    C_SDSP((BitVector<6>, regidx)),
    C_JR(regidx),
    C_JALR(regidx),
    C_MV((regidx, regidx)),
    C_EBREAK(()),
    C_ADD((regidx, regidx)),
    MUL((regidx, regidx, regidx, mul_op)),
    DIV((regidx, regidx, regidx, bool)),
    REM((regidx, regidx, regidx, bool)),
    MULW((regidx, regidx, regidx)),
    DIVW((regidx, regidx, regidx, bool)),
    REMW((regidx, regidx, regidx, bool)),
    CSR((csreg, regidx, regidx, bool, csrop)),
    URET(()),
    C_NOP_HINT(BitVector<6>),
    C_ADDI_HINT(regidx),
    C_LI_HINT(BitVector<6>),
    C_LUI_HINT(BitVector<6>),
    C_MV_HINT(regidx),
    C_ADD_HINT(regidx),
    C_SLLI_HINT((BitVector<6>, regidx)),
    C_SRLI_HINT(cregidx),
    C_SRAI_HINT(cregidx),
    FENCE_RESERVED((BitVector<4>, BitVector<4>, BitVector<4>, regidx, regidx)),
    FENCEI_RESERVED((BitVector<12>, regidx, regidx)),
    LOAD_FP((BitVector<12>, regidx, regidx, word_width)),
    STORE_FP((BitVector<12>, regidx, regidx, word_width)),
    F_MADD_TYPE_S((regidx, regidx, regidx, rounding_mode, regidx, f_madd_op_S)),
    F_BIN_RM_TYPE_S((regidx, regidx, rounding_mode, regidx, f_bin_rm_op_S)),
    F_UN_RM_TYPE_S((regidx, rounding_mode, regidx, f_un_rm_op_S)),
    F_BIN_TYPE_S((regidx, regidx, regidx, f_bin_op_S)),
    F_UN_TYPE_S((regidx, regidx, f_un_op_S)),
    C_FLWSP((BitVector<6>, regidx)),
    C_FSWSP((BitVector<6>, regidx)),
    C_FLW((BitVector<5>, cregidx, cregidx)),
    C_FSW((BitVector<5>, cregidx, cregidx)),
    F_MADD_TYPE_D((regidx, regidx, regidx, rounding_mode, regidx, f_madd_op_D)),
    F_BIN_RM_TYPE_D((regidx, regidx, rounding_mode, regidx, f_bin_rm_op_D)),
    F_UN_RM_TYPE_D((regidx, rounding_mode, regidx, f_un_rm_op_D)),
    F_BIN_TYPE_D((regidx, regidx, regidx, f_bin_op_D)),
    F_UN_TYPE_D((regidx, regidx, f_un_op_D)),
    C_FLDSP((BitVector<6>, regidx)),
    C_FSDSP((BitVector<6>, regidx)),
    C_FLD((BitVector<5>, cregidx, cregidx)),
    C_FSD((BitVector<5>, cregidx, cregidx)),
    SINVAL_VMA((regidx, regidx)),
    SFENCE_W_INVAL(()),
    SFENCE_INVAL_IR(()),
    RISCV_SLLIUW((BitVector<6>, regidx, regidx)),
    ZBA_RTYPEUW((regidx, regidx, regidx, bropw_zba)),
    ZBA_RTYPE((regidx, regidx, regidx, brop_zba)),
    RISCV_RORIW((BitVector<5>, regidx, regidx)),
    RISCV_RORI((BitVector<6>, regidx, regidx)),
    ZBB_RTYPEW((regidx, regidx, regidx, bropw_zbb)),
    ZBB_RTYPE((regidx, regidx, regidx, brop_zbb)),
    ZBB_EXTOP((regidx, regidx, extop_zbb)),
    RISCV_REV8((regidx, regidx)),
    RISCV_ORCB((regidx, regidx)),
    RISCV_CPOP((regidx, regidx)),
    RISCV_CPOPW((regidx, regidx)),
    RISCV_CLZ((regidx, regidx)),
    RISCV_CLZW((regidx, regidx)),
    RISCV_CTZ((regidx, regidx)),
    RISCV_CTZW((regidx, regidx)),
    RISCV_CLMUL((regidx, regidx, regidx)),
    RISCV_CLMULH((regidx, regidx, regidx)),
    RISCV_CLMULR((regidx, regidx, regidx)),
    ZBS_IOP((BitVector<6>, regidx, regidx, biop_zbs)),
    ZBS_RTYPE((regidx, regidx, regidx, brop_zbs)),
    C_LBU((BitVector<2>, cregidx, cregidx)),
    C_LHU((BitVector<2>, cregidx, cregidx)),
    C_LH((BitVector<2>, cregidx, cregidx)),
    C_SB((BitVector<2>, cregidx, cregidx)),
    C_SH((BitVector<2>, cregidx, cregidx)),
    C_ZEXT_B(cregidx),
    C_SEXT_B(cregidx),
    C_ZEXT_H(cregidx),
    C_SEXT_H(cregidx),
    C_ZEXT_W(cregidx),
    C_NOT(cregidx),
    C_MUL((cregidx, cregidx)),
    F_BIN_RM_TYPE_H((regidx, regidx, rounding_mode, regidx, f_bin_rm_op_H)),
    F_MADD_TYPE_H((regidx, regidx, regidx, rounding_mode, regidx, f_madd_op_H)),
    F_BIN_TYPE_H((regidx, regidx, regidx, f_bin_op_H)),
    F_UN_RM_TYPE_H((regidx, rounding_mode, regidx, f_un_rm_op_H)),
    F_UN_TYPE_H((regidx, regidx, f_un_op_H)),
    RISCV_FLI_H((BitVector<5>, regidx)),
    RISCV_FLI_S((BitVector<5>, regidx)),
    RISCV_FLI_D((BitVector<5>, regidx)),
    RISCV_FMINM_H((regidx, regidx, regidx)),
    RISCV_FMAXM_H((regidx, regidx, regidx)),
    RISCV_FMINM_S((regidx, regidx, regidx)),
    RISCV_FMAXM_S((regidx, regidx, regidx)),
    RISCV_FMINM_D((regidx, regidx, regidx)),
    RISCV_FMAXM_D((regidx, regidx, regidx)),
    RISCV_FROUND_H((regidx, rounding_mode, regidx)),
    RISCV_FROUNDNX_H((regidx, rounding_mode, regidx)),
    RISCV_FROUND_S((regidx, rounding_mode, regidx)),
    RISCV_FROUNDNX_S((regidx, rounding_mode, regidx)),
    RISCV_FROUND_D((regidx, rounding_mode, regidx)),
    RISCV_FROUNDNX_D((regidx, rounding_mode, regidx)),
    RISCV_FMVH_X_D((regidx, regidx)),
    RISCV_FMVP_D_X((regidx, regidx, regidx)),
    RISCV_FLEQ_H((regidx, regidx, regidx)),
    RISCV_FLTQ_H((regidx, regidx, regidx)),
    RISCV_FLEQ_S((regidx, regidx, regidx)),
    RISCV_FLTQ_S((regidx, regidx, regidx)),
    RISCV_FLEQ_D((regidx, regidx, regidx)),
    RISCV_FLTQ_D((regidx, regidx, regidx)),
    RISCV_FCVTMOD_W_D((regidx, regidx)),
    SHA256SIG0((regidx, regidx)),
    SHA256SIG1((regidx, regidx)),
    SHA256SUM0((regidx, regidx)),
    SHA256SUM1((regidx, regidx)),
    AES32ESMI((BitVector<2>, regidx, regidx, regidx)),
    AES32ESI((BitVector<2>, regidx, regidx, regidx)),
    AES32DSMI((BitVector<2>, regidx, regidx, regidx)),
    AES32DSI((BitVector<2>, regidx, regidx, regidx)),
    SHA512SIG0L((regidx, regidx, regidx)),
    SHA512SIG0H((regidx, regidx, regidx)),
    SHA512SIG1L((regidx, regidx, regidx)),
    SHA512SIG1H((regidx, regidx, regidx)),
    SHA512SUM0R((regidx, regidx, regidx)),
    SHA512SUM1R((regidx, regidx, regidx)),
    AES64KS1I((BitVector<4>, regidx, regidx)),
    AES64KS2((regidx, regidx, regidx)),
    AES64IM((regidx, regidx)),
    AES64ESM((regidx, regidx, regidx)),
    AES64ES((regidx, regidx, regidx)),
    AES64DSM((regidx, regidx, regidx)),
    AES64DS((regidx, regidx, regidx)),
    SHA512SIG0((regidx, regidx)),
    SHA512SIG1((regidx, regidx)),
    SHA512SUM0((regidx, regidx)),
    SHA512SUM1((regidx, regidx)),
    SM3P0((regidx, regidx)),
    SM3P1((regidx, regidx)),
    SM4ED((BitVector<2>, regidx, regidx, regidx)),
    SM4KS((BitVector<2>, regidx, regidx, regidx)),
    ZBKB_RTYPE((regidx, regidx, regidx, brop_zbkb)),
    ZBKB_PACKW((regidx, regidx, regidx)),
    RISCV_ZIP((regidx, regidx)),
    RISCV_UNZIP((regidx, regidx)),
    RISCV_BREV8((regidx, regidx)),
    RISCV_XPERM8((regidx, regidx, regidx)),
    RISCV_XPERM4((regidx, regidx, regidx)),
    ZICOND_RTYPE((regidx, regidx, regidx, zicondop)),
    VSETVLI(
        (
            BitVector<1>,
            BitVector<1>,
            BitVector<3>,
            BitVector<3>,
            regidx,
            regidx,
        ),
    ),
    VSETVL((regidx, regidx, regidx)),
    VSETIVLI(
        (
            BitVector<1>,
            BitVector<1>,
            BitVector<3>,
            BitVector<3>,
            regidx,
            regidx,
        ),
    ),
    VVTYPE((vvfunct6, BitVector<1>, regidx, regidx, regidx)),
    NVSTYPE((nvsfunct6, BitVector<1>, regidx, regidx, regidx)),
    NVTYPE((nvfunct6, BitVector<1>, regidx, regidx, regidx)),
    MASKTYPEV((regidx, regidx, regidx)),
    MOVETYPEV((regidx, regidx)),
    VXTYPE((vxfunct6, BitVector<1>, regidx, regidx, regidx)),
    NXSTYPE((nxsfunct6, BitVector<1>, regidx, regidx, regidx)),
    NXTYPE((nxfunct6, BitVector<1>, regidx, regidx, regidx)),
    VXSG((vxsgfunct6, BitVector<1>, regidx, regidx, regidx)),
    MASKTYPEX((regidx, regidx, regidx)),
    MOVETYPEX((regidx, regidx)),
    VITYPE((vifunct6, BitVector<1>, regidx, BitVector<5>, regidx)),
    NISTYPE((nisfunct6, BitVector<1>, regidx, regidx, regidx)),
    NITYPE((nifunct6, BitVector<1>, regidx, regidx, regidx)),
    VISG((visgfunct6, BitVector<1>, regidx, BitVector<5>, regidx)),
    MASKTYPEI((regidx, BitVector<5>, regidx)),
    MOVETYPEI((regidx, BitVector<5>)),
    VMVRTYPE((regidx, BitVector<5>, regidx)),
    MVVTYPE((mvvfunct6, BitVector<1>, regidx, regidx, regidx)),
    MVVMATYPE((mvvmafunct6, BitVector<1>, regidx, regidx, regidx)),
    WVVTYPE((wvvfunct6, BitVector<1>, regidx, regidx, regidx)),
    WVTYPE((wvfunct6, BitVector<1>, regidx, regidx, regidx)),
    WMVVTYPE((wmvvfunct6, BitVector<1>, regidx, regidx, regidx)),
    VEXT2TYPE((vext2funct6, BitVector<1>, regidx, regidx)),
    VEXT4TYPE((vext4funct6, BitVector<1>, regidx, regidx)),
    VEXT8TYPE((vext8funct6, BitVector<1>, regidx, regidx)),
    VMVXS((regidx, regidx)),
    MVVCOMPRESS((regidx, regidx, regidx)),
    MVXTYPE((mvxfunct6, BitVector<1>, regidx, regidx, regidx)),
    MVXMATYPE((mvxmafunct6, BitVector<1>, regidx, regidx, regidx)),
    WVXTYPE((wvxfunct6, BitVector<1>, regidx, regidx, regidx)),
    WXTYPE((wxfunct6, BitVector<1>, regidx, regidx, regidx)),
    WMVXTYPE((wmvxfunct6, BitVector<1>, regidx, regidx, regidx)),
    VMVSX((regidx, regidx)),
    FVVTYPE((fvvfunct6, BitVector<1>, regidx, regidx, regidx)),
    FVVMATYPE((fvvmafunct6, BitVector<1>, regidx, regidx, regidx)),
    FWVVTYPE((fwvvfunct6, BitVector<1>, regidx, regidx, regidx)),
    FWVVMATYPE((fwvvmafunct6, BitVector<1>, regidx, regidx, regidx)),
    FWVTYPE((fwvfunct6, BitVector<1>, regidx, regidx, regidx)),
    VFUNARY0((BitVector<1>, regidx, vfunary0, regidx)),
    VFWUNARY0((BitVector<1>, regidx, vfwunary0, regidx)),
    VFNUNARY0((BitVector<1>, regidx, vfnunary0, regidx)),
    VFUNARY1((BitVector<1>, regidx, vfunary1, regidx)),
    VFMVFS((regidx, regidx)),
    FVFTYPE((fvffunct6, BitVector<1>, regidx, regidx, regidx)),
    FVFMATYPE((fvfmafunct6, BitVector<1>, regidx, regidx, regidx)),
    FWVFTYPE((fwvffunct6, BitVector<1>, regidx, regidx, regidx)),
    FWVFMATYPE((fwvfmafunct6, BitVector<1>, regidx, regidx, regidx)),
    FWFTYPE((fwffunct6, BitVector<1>, regidx, regidx, regidx)),
    VFMERGE((regidx, regidx, regidx)),
    VFMV((regidx, regidx)),
    VFMVSF((regidx, regidx)),
    VLSEGTYPE((BitVector<3>, BitVector<1>, regidx, vlewidth, regidx)),
    VLSEGFFTYPE((BitVector<3>, BitVector<1>, regidx, vlewidth, regidx)),
    VSSEGTYPE((BitVector<3>, BitVector<1>, regidx, vlewidth, regidx)),
    VLSSEGTYPE((BitVector<3>, BitVector<1>, regidx, regidx, vlewidth, regidx)),
    VSSSEGTYPE((BitVector<3>, BitVector<1>, regidx, regidx, vlewidth, regidx)),
    VLUXSEGTYPE((BitVector<3>, BitVector<1>, regidx, regidx, vlewidth, regidx)),
    VLOXSEGTYPE((BitVector<3>, BitVector<1>, regidx, regidx, vlewidth, regidx)),
    VSUXSEGTYPE((BitVector<3>, BitVector<1>, regidx, regidx, vlewidth, regidx)),
    VSOXSEGTYPE((BitVector<3>, BitVector<1>, regidx, regidx, vlewidth, regidx)),
    VLRETYPE((BitVector<3>, regidx, vlewidth, regidx)),
    VSRETYPE((BitVector<3>, regidx, regidx)),
    VMTYPE((regidx, regidx, vmlsop)),
    MMTYPE((mmfunct6, regidx, regidx, regidx)),
    VCPOP_M((BitVector<1>, regidx, regidx)),
    VFIRST_M((BitVector<1>, regidx, regidx)),
    VMSBF_M((BitVector<1>, regidx, regidx)),
    VMSIF_M((BitVector<1>, regidx, regidx)),
    VMSOF_M((BitVector<1>, regidx, regidx)),
    VIOTA_M((BitVector<1>, regidx, regidx)),
    VID_V((BitVector<1>, regidx)),
    VVMTYPE((vvmfunct6, regidx, regidx, regidx)),
    VVMCTYPE((vvmcfunct6, regidx, regidx, regidx)),
    VVMSTYPE((vvmsfunct6, regidx, regidx, regidx)),
    VVCMPTYPE((vvcmpfunct6, BitVector<1>, regidx, regidx, regidx)),
    VXMTYPE((vxmfunct6, regidx, regidx, regidx)),
    VXMCTYPE((vxmcfunct6, regidx, regidx, regidx)),
    VXMSTYPE((vxmsfunct6, regidx, regidx, regidx)),
    VXCMPTYPE((vxcmpfunct6, BitVector<1>, regidx, regidx, regidx)),
    VIMTYPE((vimfunct6, regidx, regidx, regidx)),
    VIMCTYPE((vimcfunct6, regidx, regidx, regidx)),
    VIMSTYPE((vimsfunct6, regidx, regidx, regidx)),
    VICMPTYPE((vicmpfunct6, BitVector<1>, regidx, regidx, regidx)),
    FVVMTYPE((fvvmfunct6, BitVector<1>, regidx, regidx, regidx)),
    FVFMTYPE((fvfmfunct6, BitVector<1>, regidx, regidx, regidx)),
    RIVVTYPE((rivvfunct6, BitVector<1>, regidx, regidx, regidx)),
    RMVVTYPE((rmvvfunct6, BitVector<1>, regidx, regidx, regidx)),
    RFVVTYPE((rfvvfunct6, BitVector<1>, regidx, regidx, regidx)),
}

pub fn readCSR(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>) -> BitVector<64> {
    let res: xlenbits = match (csr, 64) {
        (b__0, _) if { (b__0 == BitVector::<12>::new(0b111100010001)) } => {
            zero_extend_64(sail_ctx.mvendorid)
        }
        (b__1, _) if { (b__1 == BitVector::<12>::new(0b111100010010)) } => sail_ctx.marchid,
        (b__2, _) if { (b__2 == BitVector::<12>::new(0b111100010011)) } => sail_ctx.mimpid,
        (b__3, _) if { (b__3 == BitVector::<12>::new(0b111100010100)) } => sail_ctx.mhartid,
        (b__4, _) if { (b__4 == BitVector::<12>::new(0b111100010101)) } => sail_ctx.mconfigptr,
        (b__5, _) if { (b__5 == BitVector::<12>::new(0b001100000000)) } => sail_ctx.mstatus.bits,
        (b__6, _) if { (b__6 == BitVector::<12>::new(0b001100000001)) } => sail_ctx.misa.bits,
        (b__7, _) if { (b__7 == BitVector::<12>::new(0b001100000010)) } => sail_ctx.medeleg.bits,
        (b__8, _) if { (b__8 == BitVector::<12>::new(0b001100000011)) } => sail_ctx.mideleg.bits,
        (b__9, _) if { (b__9 == BitVector::<12>::new(0b001100000100)) } => sail_ctx.mie.bits,
        (b__10, _) if { (b__10 == BitVector::<12>::new(0b001100000101)) } => {
            get_mtvec(sail_ctx, ())
        }
        (b__11, _) if { (b__11 == BitVector::<12>::new(0b001100000110)) } => {
            zero_extend_64(sail_ctx.mcounteren.bits)
        }
        (b__12, _) if { (b__12 == BitVector::<12>::new(0b001100001010)) } => {
            subrange_bits(sail_ctx.menvcfg.bits, (64 - 1), 0)
        }
        (b__15, _) if { (b__15 == BitVector::<12>::new(0b001100100000)) } => {
            zero_extend_64(sail_ctx.mcountinhibit.bits)
        }
        (b__16, _) if { (b__16 == BitVector::<12>::new(0b001101000000)) } => sail_ctx.mscratch,
        (b__17, _) if { (b__17 == BitVector::<12>::new(0b001101000001)) } => {
            (get_xret_target(sail_ctx, Privilege::Machine) & pc_alignment_mask(sail_ctx, ()))
        }
        (b__18, _) if { (b__18 == BitVector::<12>::new(0b001101000010)) } => sail_ctx.mcause.bits,
        (b__19, _) if { (b__19 == BitVector::<12>::new(0b001101000011)) } => sail_ctx.mtval,
        (b__20, _) if { (b__20 == BitVector::<12>::new(0b001101000100)) } => sail_ctx.mip.bits,
        (v__12, _)
            if {
                {
                    let idx: BitVector<4> = v__12.subrange::<0, 4, 4>();
                    ((bitvector_access(idx, 0) == false) || (64 == 32))
                        && (v__12.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111010))
                }
            } =>
        {
            let idx: BitVector<4> = v__12.subrange::<0, 4, 4>();
            let var_276 = idx.as_usize();
            pmpReadCfgReg(sail_ctx, var_276)
        }
        (v__14, _) if { (v__14.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111011)) } => {
            let idx: BitVector<4> = v__14.subrange::<0, 4, 4>();
            let var_277 = bitvector_concat(BitVector::<2>::new(0b00), idx).as_usize();
            pmpReadAddrReg(sail_ctx, var_277)
        }
        (v__16, _) if { (v__16.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111100)) } => {
            let idx: BitVector<4> = v__16.subrange::<0, 4, 4>();
            let var_278 = bitvector_concat(BitVector::<2>::new(0b01), idx).as_usize();
            pmpReadAddrReg(sail_ctx, var_278)
        }
        (v__18, _) if { (v__18.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111101)) } => {
            let idx: BitVector<4> = v__18.subrange::<0, 4, 4>();
            let var_279 = bitvector_concat(BitVector::<2>::new(0b10), idx).as_usize();
            pmpReadAddrReg(sail_ctx, var_279)
        }
        (v__20, _) if { (v__20.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111110)) } => {
            let idx: BitVector<4> = v__20.subrange::<0, 4, 4>();
            let var_280 = bitvector_concat(BitVector::<2>::new(0b11), idx).as_usize();
            pmpReadAddrReg(sail_ctx, var_280)
        }
        (b__21, _) if { (b__21 == BitVector::<12>::new(0b101100000000)) } => {
            subrange_bits(sail_ctx.mcycle, (64 - 1), 0)
        }
        (b__22, _) if { (b__22 == BitVector::<12>::new(0b101100000010)) } => {
            subrange_bits(sail_ctx.minstret, (64 - 1), 0)
        }
        (b__25, _) if { (b__25 == BitVector::<12>::new(0b000000001000)) } => {
            zero_extend_64(sail_ctx.vstart)
        }
        (b__26, _) if { (b__26 == BitVector::<12>::new(0b000000001001)) } => {
            zero_extend_64(sail_ctx.vxsat)
        }
        (b__27, _) if { (b__27 == BitVector::<12>::new(0b000000001010)) } => {
            zero_extend_64(sail_ctx.vxrm)
        }
        (b__28, _) if { (b__28 == BitVector::<12>::new(0b000000001111)) } => {
            zero_extend_64(sail_ctx.vcsr.bits)
        }
        (b__29, _) if { (b__29 == BitVector::<12>::new(0b110000100000)) } => sail_ctx.vl,
        (b__30, _) if { (b__30 == BitVector::<12>::new(0b110000100001)) } => sail_ctx.vtype.bits,
        (b__31, _) if { (b__31 == BitVector::<12>::new(0b110000100010)) } => sail_ctx.vlenb,
        (b__32, _) if { (b__32 == BitVector::<12>::new(0b011110100000)) } => !(sail_ctx.tselect),
        (b__33, _) if { (b__33 == BitVector::<12>::new(0b000100000000)) } => {
            lower_mstatus(sail_ctx, sail_ctx.mstatus).bits
        }
        (b__34, _) if { (b__34 == BitVector::<12>::new(0b000100000010)) } => sail_ctx.sedeleg.bits,
        (b__35, _) if { (b__35 == BitVector::<12>::new(0b000100000011)) } => sail_ctx.sideleg.bits,
        (b__36, _) if { (b__36 == BitVector::<12>::new(0b000100000100)) } => {
            lower_mie(sail_ctx, sail_ctx.mie, sail_ctx.mideleg).bits
        }
        (b__37, _) if { (b__37 == BitVector::<12>::new(0b000100000101)) } => {
            get_stvec(sail_ctx, ())
        }
        (b__38, _) if { (b__38 == BitVector::<12>::new(0b000100000110)) } => {
            zero_extend_64(sail_ctx.scounteren.bits)
        }
        (b__39, _) if { (b__39 == BitVector::<12>::new(0b000100001010)) } => {
            subrange_bits(sail_ctx.senvcfg.bits, (64 - 1), 0)
        }
        (b__40, _) if { (b__40 == BitVector::<12>::new(0b000101000000)) } => sail_ctx.sscratch,
        (b__41, _) if { (b__41 == BitVector::<12>::new(0b000101000001)) } => {
            (get_xret_target(sail_ctx, Privilege::Supervisor) & pc_alignment_mask(sail_ctx, ()))
        }
        (b__42, _) if { (b__42 == BitVector::<12>::new(0b000101000010)) } => sail_ctx.scause.bits,
        (b__43, _) if { (b__43 == BitVector::<12>::new(0b000101000011)) } => sail_ctx.stval,
        (b__44, _) if { (b__44 == BitVector::<12>::new(0b000101000100)) } => {
            lower_mip(sail_ctx, sail_ctx.mip, sail_ctx.mideleg).bits
        }
        (b__45, _) if { (b__45 == BitVector::<12>::new(0b000110000000)) } => sail_ctx.satp,
        (b__46, _) if { (b__46 == BitVector::<12>::new(0b110000000000)) } => {
            subrange_bits(sail_ctx.mcycle, (64 - 1), 0)
        }
        (b__47, _) if { (b__47 == BitVector::<12>::new(0b110000000001)) } => {
            subrange_bits(sail_ctx.mtime, (64 - 1), 0)
        }
        (b__48, _) if { (b__48 == BitVector::<12>::new(0b110000000010)) } => {
            subrange_bits(sail_ctx.minstret, (64 - 1), 0)
        }
        (b__52, _) if { (b__52 == BitVector::<12>::new(0b000000010101)) } => {
            read_seed_csr(sail_ctx, ())
        }
        _ => match ext_read_CSR(sail_ctx, csr) {
            Some(res) => res,
            None => {
                print_output(String::from("unhandled read to CSR "), csr);
                zero_extend_64(BitVector::<4>::new(0b0000))
            }
        },
    };
    if { get_config_print_reg(sail_ctx, ()) } {
        print_reg(format!(
            "{}{}",
            String::from("CSR "),
            format!(
                "{}{}",
                csr_name(sail_ctx, csr),
                format!("{}{}", String::from(" -> "), bits_str(res))
            )
        ))
    } else {
        ()
    };
    res
}

pub fn writeCSR(sail_ctx: &mut SailVirtCtx, csr: BitVector<12>, value: BitVector<64>) {
    let res: Option<xlenbits> = match (csr, 64) {
        (b__0, _) if { (b__0 == BitVector::<12>::new(0b001100000000)) } => {
            sail_ctx.mstatus = legalize_mstatus(sail_ctx, sail_ctx.mstatus, value);
            Some(sail_ctx.mstatus.bits)
        }
        (b__1, _) if { (b__1 == BitVector::<12>::new(0b001100000001)) } => {
            sail_ctx.misa = legalize_misa(sail_ctx, sail_ctx.misa, value);
            Some(sail_ctx.misa.bits)
        }
        (b__2, _) if { (b__2 == BitVector::<12>::new(0b001100000010)) } => {
            sail_ctx.medeleg = legalize_medeleg(sail_ctx, sail_ctx.medeleg, value);
            Some(sail_ctx.medeleg.bits)
        }
        (b__3, _) if { (b__3 == BitVector::<12>::new(0b001100000011)) } => {
            sail_ctx.mideleg = legalize_mideleg(sail_ctx, sail_ctx.mideleg, value);
            Some(sail_ctx.mideleg.bits)
        }
        (b__4, _) if { (b__4 == BitVector::<12>::new(0b001100000100)) } => {
            sail_ctx.mie = legalize_mie(sail_ctx, sail_ctx.mie, value);
            Some(sail_ctx.mie.bits)
        }
        (b__5, _) if { (b__5 == BitVector::<12>::new(0b001100000101)) } => {
            Some(set_mtvec(sail_ctx, value))
        }
        (b__6, _) if { (b__6 == BitVector::<12>::new(0b001100000110)) } => {
            sail_ctx.mcounteren = legalize_mcounteren(sail_ctx, sail_ctx.mcounteren, value);
            Some(zero_extend_64(sail_ctx.mcounteren.bits))
        }
        (b__8, 64) if { (b__8 == BitVector::<12>::new(0b001100001010)) } => {
            sail_ctx.menvcfg = legalize_menvcfg(sail_ctx, sail_ctx.menvcfg, value);
            Some(sail_ctx.menvcfg.bits)
        }
        (b__11, _) if { (b__11 == BitVector::<12>::new(0b001100100000)) } => {
            sail_ctx.mcountinhibit =
                legalize_mcountinhibit(sail_ctx, sail_ctx.mcountinhibit, value);
            Some(zero_extend_64(sail_ctx.mcountinhibit.bits))
        }
        (b__12, _) if { (b__12 == BitVector::<12>::new(0b001101000000)) } => {
            sail_ctx.mscratch = value;
            Some(sail_ctx.mscratch)
        }
        (b__13, _) if { (b__13 == BitVector::<12>::new(0b001101000001)) } => {
            Some(set_xret_target(sail_ctx, Privilege::Machine, value))
        }
        (b__14, _) if { (b__14 == BitVector::<12>::new(0b001101000010)) } => {
            sail_ctx.mcause.bits = value;
            Some(sail_ctx.mcause.bits)
        }
        (b__15, _) if { (b__15 == BitVector::<12>::new(0b001101000011)) } => {
            sail_ctx.mtval = value;
            Some(sail_ctx.mtval)
        }
        (b__16, _) if { (b__16 == BitVector::<12>::new(0b001101000100)) } => {
            sail_ctx.mip = legalize_mip(sail_ctx, sail_ctx.mip, value);
            Some(sail_ctx.mip.bits)
        }
        (v__22, _)
            if {
                {
                    let idx: BitVector<4> = v__22.subrange::<0, 4, 4>();
                    ((bitvector_access(idx, 0) == false) || (64 == 32))
                        && (v__22.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111010))
                }
            } =>
        {
            let idx: BitVector<4> = v__22.subrange::<0, 4, 4>();
            let idx = idx.as_usize();
            pmpWriteCfgReg(sail_ctx, idx, value);
            Some(pmpReadCfgReg(sail_ctx, idx))
        }
        (v__24, _) if { (v__24.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111011)) } => {
            let idx: BitVector<4> = v__24.subrange::<0, 4, 4>();
            let idx = bitvector_concat(BitVector::<2>::new(0b00), idx).as_usize();
            pmpWriteAddrReg(sail_ctx, idx, value);
            Some(pmpReadAddrReg(sail_ctx, idx))
        }
        (v__26, _) if { (v__26.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111100)) } => {
            let idx: BitVector<4> = v__26.subrange::<0, 4, 4>();
            let idx = bitvector_concat(BitVector::<2>::new(0b01), idx).as_usize();
            pmpWriteAddrReg(sail_ctx, idx, value);
            Some(pmpReadAddrReg(sail_ctx, idx))
        }
        (v__28, _) if { (v__28.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111101)) } => {
            let idx: BitVector<4> = v__28.subrange::<0, 4, 4>();
            let idx = bitvector_concat(BitVector::<2>::new(0b10), idx).as_usize();
            pmpWriteAddrReg(sail_ctx, idx, value);
            Some(pmpReadAddrReg(sail_ctx, idx))
        }
        (v__30, _) if { (v__30.subrange::<4, 12, 8>() == BitVector::<8>::new(0b00111110)) } => {
            let idx: BitVector<4> = v__30.subrange::<0, 4, 4>();
            let idx = bitvector_concat(BitVector::<2>::new(0b11), idx).as_usize();
            pmpWriteAddrReg(sail_ctx, idx, value);
            Some(pmpReadAddrReg(sail_ctx, idx))
        }
        (b__17, _) if { (b__17 == BitVector::<12>::new(0b101100000000)) } => {
            sail_ctx.mcycle = value;
            Some(value)
        }
        (b__18, _) if { (b__18 == BitVector::<12>::new(0b101100000010)) } => {
            sail_ctx.minstret = value;
            sail_ctx.minstret_increment = false;
            Some(value)
        }
        (b__21, _) if { (b__21 == BitVector::<12>::new(0b011110100000)) } => {
            sail_ctx.tselect = value;
            Some(sail_ctx.tselect)
        }
        (b__22, _) if { (b__22 == BitVector::<12>::new(0b000100000000)) } => {
            sail_ctx.mstatus = legalize_sstatus(sail_ctx, sail_ctx.mstatus, value);
            Some(sail_ctx.mstatus.bits)
        }
        (b__23, _) if { (b__23 == BitVector::<12>::new(0b000100000010)) } => {
            sail_ctx.sedeleg = legalize_sedeleg(sail_ctx, sail_ctx.sedeleg, value);
            Some(sail_ctx.sedeleg.bits)
        }
        (b__24, _) if { (b__24 == BitVector::<12>::new(0b000100000011)) } => {
            sail_ctx.sideleg.bits = value;
            Some(sail_ctx.sideleg.bits)
        }
        (b__25, _) if { (b__25 == BitVector::<12>::new(0b000100000100)) } => {
            sail_ctx.mie = legalize_sie(sail_ctx, sail_ctx.mie, sail_ctx.mideleg, value);
            Some(sail_ctx.mie.bits)
        }
        (b__26, _) if { (b__26 == BitVector::<12>::new(0b000100000101)) } => {
            Some(set_stvec(sail_ctx, value))
        }
        (b__27, _) if { (b__27 == BitVector::<12>::new(0b000100000110)) } => {
            sail_ctx.scounteren = legalize_scounteren(sail_ctx, sail_ctx.scounteren, value);
            Some(zero_extend_64(sail_ctx.scounteren.bits))
        }
        (b__28, _) if { (b__28 == BitVector::<12>::new(0b000100001010)) } => {
            sail_ctx.senvcfg = {
                let var_285 = sail_ctx.senvcfg;
                let var_286 = zero_extend_64(value);
                legalize_senvcfg(sail_ctx, var_285, var_286)
            };
            Some(subrange_bits(sail_ctx.senvcfg.bits, (64 - 1), 0))
        }
        (b__29, _) if { (b__29 == BitVector::<12>::new(0b000101000000)) } => {
            sail_ctx.sscratch = value;
            Some(sail_ctx.sscratch)
        }
        (b__30, _) if { (b__30 == BitVector::<12>::new(0b000101000001)) } => {
            Some(set_xret_target(sail_ctx, Privilege::Supervisor, value))
        }
        (b__31, _) if { (b__31 == BitVector::<12>::new(0b000101000010)) } => {
            sail_ctx.scause.bits = value;
            Some(sail_ctx.scause.bits)
        }
        (b__32, _) if { (b__32 == BitVector::<12>::new(0b000101000011)) } => {
            sail_ctx.stval = value;
            Some(sail_ctx.stval)
        }
        (b__33, _) if { (b__33 == BitVector::<12>::new(0b000101000100)) } => {
            sail_ctx.mip = legalize_sip(sail_ctx, sail_ctx.mip, sail_ctx.mideleg, value);
            Some(sail_ctx.mip.bits)
        }
        (b__34, _) if { (b__34 == BitVector::<12>::new(0b000110000000)) } => {
            sail_ctx.satp = {
                let var_287 = cur_Architecture(sail_ctx, ());
                let var_288 = sail_ctx.satp;
                let var_289 = value;
                legalize_satp(sail_ctx, var_287, var_288, var_289)
            };
            Some(sail_ctx.satp)
        }
        (b__35, _) if { (b__35 == BitVector::<12>::new(0b000000010101)) } => {
            write_seed_csr(sail_ctx, ())
        }
        (b__36, _) if { (b__36 == BitVector::<12>::new(0b000000001000)) } => {
            let vstart_length = get_vlen_pow(sail_ctx, ());
            sail_ctx.vstart = zero_extend_16(subrange_bits_8(value, 7, 0));
            Some(zero_extend_64(sail_ctx.vstart))
        }
        (b__37, _) if { (b__37 == BitVector::<12>::new(0b000000001001)) } => {
            sail_ctx.vxsat = value.subrange::<0, 1, 1>();
            Some(zero_extend_64(sail_ctx.vxsat))
        }
        (b__38, _) if { (b__38 == BitVector::<12>::new(0b000000001010)) } => {
            sail_ctx.vxrm = value.subrange::<0, 2, 2>();
            Some(zero_extend_64(sail_ctx.vxrm))
        }
        (b__39, _) if { (b__39 == BitVector::<12>::new(0b000000001111)) } => {
            sail_ctx.vcsr.bits = value.subrange::<0, 3, 3>();
            Some(zero_extend_64(sail_ctx.vcsr.bits))
        }
        (b__40, _) if { (b__40 == BitVector::<12>::new(0b110000100000)) } => {
            sail_ctx.vl = value;
            Some(sail_ctx.vl)
        }
        (b__41, _) if { (b__41 == BitVector::<12>::new(0b110000100001)) } => {
            sail_ctx.vtype.bits = value;
            Some(sail_ctx.vtype.bits)
        }
        (b__42, _) if { (b__42 == BitVector::<12>::new(0b110000100010)) } => {
            sail_ctx.vlenb = value;
            Some(sail_ctx.vlenb)
        }
        _ => ext_write_CSR(sail_ctx, csr, value),
    };
    match res {
        Some(v) => {
            if { get_config_print_reg(sail_ctx, ()) } {
                print_reg(format!(
                    "{}{}",
                    String::from("CSR "),
                    format!(
                        "{}{}",
                        csr_name(sail_ctx, csr),
                        format!(
                            "{}{}",
                            String::from(" <- "),
                            format!(
                                "{}{}",
                                bits_str(v),
                                format!(
                                    "{}{}",
                                    String::from(" (input: "),
                                    format!("{}{}", bits_str(value), String::from(")"))
                                )
                            )
                        )
                    )
                ))
            } else {
                ()
            }
        }
        None => print_output(String::from("unhandled write to CSR "), csr),
    }
}

fn execute_ITYPE(
    sail_ctx: &mut SailVirtCtx,
    imm: BitVector<12>,
    rs1: regidx,
    rd: regidx,
    op: iop,
) -> Retired {
    let rs1_val = rX_bits(sail_ctx, rs1);
    let immext: xlenbits = sign_extend(64, imm);
    let result: xlenbits = match op {
        iop::RISCV_ADDI => rs1_val.wrapped_add(immext),
        iop::RISCV_SLTI => zero_extend_64({
            let var_290 = _operator_smaller_s_(sail_ctx, rs1_val, immext);
            bool_to_bits(sail_ctx, var_290)
        }),
        iop::RISCV_SLTIU => zero_extend_64({
            let var_291 = _operator_smaller_u_(sail_ctx, rs1_val, immext);
            bool_to_bits(sail_ctx, var_291)
        }),
        iop::RISCV_ANDI => (rs1_val & immext),
        iop::RISCV_ORI => (rs1_val | immext),
        iop::RISCV_XORI => (rs1_val ^ immext),
    };
    wX_bits(sail_ctx, rd, result);
    Retired::RETIRE_SUCCESS
}

pub(crate) fn execute_MRET(sail_ctx: &mut SailVirtCtx) -> Retired {
    if { (sail_ctx.cur_privilege != Privilege::Machine) } {
        handle_illegal(sail_ctx, ());
        Retired::RETIRE_FAIL
    } else if { !(ext_check_xret_priv(sail_ctx, Privilege::Machine)) } {
        ext_fail_xret_priv(sail_ctx, ());
        Retired::RETIRE_FAIL
    } else {
        {
            let var_292 = {
                let var_293 = sail_ctx.cur_privilege;
                let var_294 = ctl_result::CTL_MRET(());
                let var_295 = sail_ctx.PC;
                exception_handler(sail_ctx, var_293, var_294, var_295)
            };
            set_next_pc(sail_ctx, var_292)
        };
        Retired::RETIRE_SUCCESS
    }
}

fn execute_WFI(sail_ctx: &mut SailVirtCtx) -> Retired {
    match sail_ctx.cur_privilege {
        Privilege::Machine => {
            platform_wfi(sail_ctx, ());
            Retired::RETIRE_SUCCESS
        }
        Privilege::Supervisor => {
            if { (_get_Mstatus_TW(sail_ctx, sail_ctx.mstatus) == BitVector::<1>::new(0b1)) } {
                handle_illegal(sail_ctx, ());
                Retired::RETIRE_FAIL
            } else {
                platform_wfi(sail_ctx, ());
                Retired::RETIRE_SUCCESS
            }
        }
        Privilege::User => {
            handle_illegal(sail_ctx, ());
            Retired::RETIRE_FAIL
        }
    }
}

fn execute_CSR(
    sail_ctx: &mut SailVirtCtx,
    csr: csreg,
    rs1: regidx,
    rd: regidx,
    is_imm: bool,
    op: csrop,
) -> Retired {
    let rs1_val: xlenbits = if { is_imm } {
        zero_extend_64(rs1)
    } else {
        rX_bits(sail_ctx, rs1)
    };
    let isWrite: bool = match op {
        csrop::CSRRW => true,
        _ => {
            if { is_imm } {
                (rs1_val.as_usize() != 0)
            } else {
                (rs1.as_usize() != 0)
            }
        }
    };
    if { !(check_CSR(sail_ctx, csr, sail_ctx.cur_privilege, isWrite)) } {
        handle_illegal(sail_ctx, ());
        Retired::RETIRE_FAIL
    } else if { !(ext_check_CSR(sail_ctx, csr, sail_ctx.cur_privilege, isWrite)) } {
        ext_check_CSR_fail(sail_ctx, ());
        Retired::RETIRE_FAIL
    } else {
        let csr_val = readCSR(sail_ctx, csr);
        if { isWrite } {
            let new_val: xlenbits = match op {
                csrop::CSRRW => rs1_val,
                csrop::CSRRS => (csr_val | rs1_val),
                csrop::CSRRC => (csr_val & !(rs1_val)),
            };
            writeCSR(sail_ctx, csr, new_val)
        } else {
            ()
        };
        wX_bits(sail_ctx, rd, csr_val);
        Retired::RETIRE_SUCCESS
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum FetchResult {
    F_Ext_Error(ext_fetch_addr_error),
    F_Base(word),
    F_RVC(half),
    F_Error((ExceptionType, xlenbits)),
}
