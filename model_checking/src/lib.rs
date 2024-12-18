use sail_model::{execute_MRET, execute_WFI, trap_handler};
use sail_prelude::BitVector;

#[macro_use]
mod symbolic;
mod adapters;

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn mret() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    ctx.emulate_mret(&mut mctx);

    execute_MRET(&mut sail_ctx);

    assert_eq!(
        ctx,
        adapters::sail_to_miralis(sail_ctx),
        "mret instruction emulation is not correct"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn wfi() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    ctx.emulate_wfi(&mut mctx);

    execute_WFI(&mut sail_ctx);

    // This field is used only in Miralis. We set it to false otherwise the assertions fails.
    ctx.is_wfi = false;

    assert_eq!(
        ctx,
        adapters::sail_to_miralis(sail_ctx),
        "wfi instruction emulation is not correct"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn interrupt_virtualization() {
    let (mut ctx, _, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generation of an interrupt
    let current_interrupt = any!(usize) % 64;

    ctx.inject_interrupt(current_interrupt);

    // Intr field is always true because we formally check the interrupt virtualization and therefore traps are out of scope
    {
        // Makes the borrow checker happy
        let cur_privilege = sail_ctx.cur_privilege;
        let pc = sail_ctx.PC;
        let ret_pc = trap_handler(
            &mut sail_ctx,
            cur_privilege,
            true,
            BitVector::new(current_interrupt as u64),
            pc,
            Some(BitVector::new(0)),
            None,
        );

        // Now we can set the return pc
        sail_ctx.nextPC = ret_pc;
    }

    // Finally, we can check that both virtual contexts are equivalent
    assert_eq!(
        ctx,
        adapters::sail_to_miralis(sail_ctx),
        "Interrupt injection is not correct"
    );
}
