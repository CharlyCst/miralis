use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::arch::{mie, write_pmp};
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use sail_model::{
    execute_MRET, execute_WFI, pmpCheck, readCSR, step_interrupts_only, writeCSR, AccessType,
    ExceptionType, Privilege,
};
use sail_prelude::{sys_pmp_count, BitField, BitVector};

use crate::adapters::{pmpaddr_sail_to_miralis, pmpcfg_sail_to_miralis, sail_to_miralis};

#[macro_use]
mod symbolic;
mod adapters;


#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn pmp_equivalence_with_miralis() {
    let (_, _, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generation of the entire address space we want to check
    let address_to_check = BitVector::new((any!(u64) >> 4) & !0xfffffff);

    // The virtual firmware is always running in userspace
    let virtual_firmware_privilege = Privilege::User;

    let access_type: AccessType = match any!(u8) % 4 {
        0 => AccessType::Read(()),
        1 => AccessType::Write(()),
        2 => AccessType::ReadWrite(((), ())),
        _ => AccessType::Execute(()),
    };

    let virtual_offset = VIRTUAL_PMP_OFFSET;
    let number_virtual_pmps = sys_pmp_count(()) - virtual_offset;

    // Deactivate the last pmp entries in the physical context
    for idx in number_virtual_pmps..sys_pmp_count(()) {
        sail_ctx.pmpcfg_n[idx] = BitField::new(0);
        sail_ctx.pmpaddr_n[idx] = BitVector::new(0);
    }

    let physical_check: Option<ExceptionType> = pmpCheck::<64>(
        &mut sail_ctx,
        address_to_check,
        8,
        access_type,
        virtual_firmware_privilege,
    );

    const START_MIRALIS_RANGE: usize = 0x80000000;
    const MIRALIS_SIZE: usize = 0x200000;

    let virtual_check: Option<ExceptionType> = {
        // Creation of the PMP group
        let mut pmp_group =
            PmpGroup::init_pmp_group(sys_pmp_count(()), START_MIRALIS_RANGE, MIRALIS_SIZE);
        pmp_group.virt_pmp_offset = virtual_offset;

        // Physical write of the pmp registers
        pmp_group.load_with_offset(
            &pmpaddr_sail_to_miralis(sail_ctx.pmpaddr_n),
            &pmpcfg_sail_to_miralis(sail_ctx.pmpcfg_n),
            virtual_offset,
            number_virtual_pmps,
        );
        unsafe {
            write_pmp(&pmp_group).flush();
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

    if START_MIRALIS_RANGE <= address_to_check.bits() as usize && (address_to_check.bits() as usize) < START_MIRALIS_RANGE + MIRALIS_SIZE {
        // Miralis must be protected
        assert_ne!(virtual_check, None);
    } else {
        // Check pmp equivalence
        assert_eq!(
            physical_check, virtual_check,
            "pmp are not installed correctly in Miralis"
        );
    }
}
