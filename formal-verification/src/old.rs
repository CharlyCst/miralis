#![allow(unused, non_snake_case)]

use crate::SailVirtContext;
use sail_prelude::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Privilege {
    User,
    Supervisor,
    Machine,
}

pub enum Execute {
    RetireFail,
    RetireSuccess,
}

fn haveUsrMode() -> bool {
    true
}

fn privLevel_to_bits(p: Privilege) -> BitVector<2> {
    match p {
        Privilege::User => BitVector::new(0b00),
        Privilege::Supervisor => BitVector::new(0b01),
        Privilege::Machine => BitVector::new(0b11),
    }
}

fn privLevel_of_bits(p: BitVector<2>) -> Privilege {
    match p.bits() {
        0b00 => Privilege::User,
        0b01 => Privilege::Supervisor,
        0b11 => Privilege::Machine,
        _ => panic!("Invalid privilege level"),
    }
}

fn pc_alignment_mask() -> BitVector<64> {
    !(BitVector::new(0b10))
}

fn _get_Mstatus_MPIE(ctx: &mut SailVirtContext) -> BitVector<1> {
    ctx.mstatus.subrange::<7, 8, 1>()
}

fn _get_Mstatus_MPP(ctx: &mut SailVirtContext) -> BitVector<2> {
    ctx.mstatus.subrange::<11, 13, 2>()
}

fn set_next_pc(ctx: &mut SailVirtContext, pc: BitVector<64>) {
    ctx.next_pc = pc
}

fn handle_illegal(TodoArgs: ()) {
    ()
}

fn get_xret_target(p: Privilege, ctx: &mut SailVirtContext) -> BitVector<64> {
    match p {
        Privilege::Machine => ctx.mepc,
        Privilege::Supervisor => ctx.sepc,
        Privilege::User => ctx.sepc,
    }
}

fn prepare_xret_target(p: Privilege, ctx: &mut SailVirtContext) -> BitVector<64> {
    get_xret_target(p, ctx)
}

fn exception_handler(
    cur_priv: Privilege,
    pc: BitVector<64>,
    ctx: &mut SailVirtContext,
) -> BitVector<64> {
    let prev_priv = ctx.cur_privilege;
    ctx.mstatus = ctx.mstatus.set_subrange::<3, 4, 1>(_get_Mstatus_MPIE(ctx));
    ctx.mstatus = ctx.mstatus.set_subrange::<7, 8, 1>(BitVector::new(0b1));
    ctx.cur_privilege = privLevel_of_bits(_get_Mstatus_MPP(ctx));
    ctx.mstatus = ctx
        .mstatus
        .set_subrange::<11, 13, 2>(privLevel_to_bits(if haveUsrMode() {
            Privilege::User
        } else {
            Privilege::Machine
        }));
    if ctx.cur_privilege != Privilege::Machine {
        ctx.mstatus = ctx.mstatus.set_subrange::<17, 18, 1>(BitVector::new(0));
    } else {
        ()
    };
    prepare_xret_target(Privilege::Machine, ctx) & pc_alignment_mask()
}

fn ext_check_xret_priv(TodoArgs: Privilege) -> bool {
    true
}

fn ext_fail_xret_priv(TodoArgs: ()) {
    ()
}

pub fn execute_MRET(ctx: &mut SailVirtContext) -> Execute{
    if ctx.cur_privilege != Privilege::Machine {
        {
            handle_illegal(());
            Execute::RetireFail
        }
    } else if !ext_check_xret_priv(Privilege::Machine) {
        {
            ext_fail_xret_priv(());
            Execute::RetireFail
        }
    } else {
        {
            let tmp = exception_handler(ctx.cur_privilege, ctx.pc, ctx);
            set_next_pc(ctx, tmp);
            Execute::RetireSuccess
        }
    }
}

