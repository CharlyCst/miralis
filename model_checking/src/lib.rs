use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::arch::{Arch, Architecture};
use miralis::virt::traits::RegisterContextGetter;
use sail_model::{
    execute_MRET, execute_WFI, pmpCheck, readCSR, step_interrupts_only, AccessType, ExceptionType,
    Privilege,
};
use sail_prelude::{BitField, BitVector};

use crate::adapters::{pmpaddr_sail_to_miralis, pmpcfg_sail_to_miralis, sail_to_miralis};

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

fn generate_csr_register() -> u64 {
    // We want only 12 bits
    let mut csr: u64 = any!(u64) & 0xFFF;

    // Ignore sedeleg and sideleg
    if csr == 0b000100000010 || csr == 0b000100000011 {
        csr = 0x0;
    }

    // Odd pmpcfg indices configs are not allowed
    if 0x3A0 <= csr && csr <= 0x3AF {
        csr &= !0b1;
    }

    return csr;
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn read_csr() {
    let (mut ctx, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Infinite number of pmps for the formal verification
    ctx.nb_pmp = usize::MAX;

    let mut csr_register = generate_csr_register();

    let is_mstatus = csr_register == 0b001100000000;
    let is_sstatus = csr_register == 0b000100000000;
    let is_mepc = csr_register == 0b001101000001;
    let is_sepc = csr_register == 0b000101000001;
    let is_sie = csr_register == 0b000100000100;
    let is_seed = csr_register == 0b000000010101;

    // TODO: Adapt the last 6 registers for the symbolic verification
    if is_mstatus || is_sstatus || is_mepc || is_sepc || is_sie || is_seed {
        csr_register = 0;
    }

    // Read value from Miralis
    let decoded_csr = mctx.decode_csr(csr_register as usize);
    let miralis_value = ctx.get(decoded_csr);

    // Read value from Sail
    let sail_value = readCSR(&mut sail_ctx, BitVector::<12>::new(csr_register)).bits as usize;

    // Verify value is the same
    assert_eq!(miralis_value, sail_value, "Read equivalence");
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn interrupt_virtualization() {
    let (mut ctx, _, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // We don't delegate any interrupts in the formal verification
    sail_ctx.mideleg = BitField::new(0);
    ctx.csr.mideleg = 0;

    // Check the virtualization
    step_interrupts_only(&mut sail_ctx, 0);
    ctx.check_and_inject_interrupts();

    // Verify the results
    assert_eq!(
        ctx,
        sail_to_miralis(sail_ctx),
        "Interrupt virtualisation doesn't work properly"
    )
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
