use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::arch::{Arch, Architecture};
use sail_model::{
    dispatchInterrupt, execute_MRET, execute_WFI, pmpCheck, trap_handler, AccessType,
    ExceptionType, Privilege,
};
use sail_prelude::{BitField, BitVector};

use crate::adapters::{
    pmpaddr_sail_to_miralis, pmpcfg_sail_to_miralis, sail_interrupts_code_to_miralis,
};

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

// The first function symbolically verifies whether an interrupt needs to be injected, formally checking the "if".
// The second function ensures that the interrupt is correctly injected into the system, verifying the "how".
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn requires_interrupt_injection() {
    let (mut ctx, _, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // We don't want this field to interfere we the result of the computation in this scenario
    ctx.is_wfi = false;

    // We don't delegate any interrupts in the formal verification
    sail_ctx.mideleg = BitField::new(0);
    ctx.csr.mideleg = 0;

    // Finally, we can check the equivalence
    assert_eq!(
        ctx.has_pending_interrupt(),
        sail_interrupts_code_to_miralis(dispatchInterrupt(&mut sail_ctx, Privilege::Machine)),
        "Interrupt detection is not correct"
    )
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

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn pmp_equivalence() {
    let (_, _, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generation of the entire address space we want to check
    let address_to_check = BitVector::new(any!(u64) >> 4);

    // The virtual firmware is always running in userspace
    let virtual_firmware_privilege = Privilege::User;

    let access_type: AccessType = match any!(u8) % 4 {
        0 => AccessType::Read(()),
        1 => AccessType::Write(()),
        2 => AccessType::ReadWrite(((), ())),
        _ => AccessType::Execute(()),
    };

    let physical_check: Option<ExceptionType> = pmpCheck::<64>(
        &mut sail_ctx,
        address_to_check,
        8,
        access_type,
        virtual_firmware_privilege,
    );

    let virtual_check: Option<ExceptionType> = {
        // Creation of the PMP group
        let mut pmp_group = PmpGroup::new(sail_prelude::sys_pmp_count(()));
        pmp_group.virt_pmp_offset = VIRTUAL_PMP_OFFSET;

        // Physical write of the pmp registers
        pmp_group.load_with_offset(
            &pmpaddr_sail_to_miralis(sail_ctx.pmpaddr_n),
            &pmpcfg_sail_to_miralis(sail_ctx.pmpcfg_n),
            0,
            sail_prelude::sys_pmp_count(()),
        );
        unsafe {
            Arch::write_pmp(&pmp_group).flush();
        }

        // Retrieve hardware context
        let mut generated_sail_ctx = adapters::miralis_to_sail(&return_userspace_ctx());

        // Execution of the pmp check function
        pmpCheck::<64>(
            &mut generated_sail_ctx,
            address_to_check,
            8,
            access_type,
            virtual_firmware_privilege,
        )
    };

    // Check pmp equivalence
    assert_eq!(
        physical_check, virtual_check,
        "pmp are not installed correctly in Miralis"
    );
}
