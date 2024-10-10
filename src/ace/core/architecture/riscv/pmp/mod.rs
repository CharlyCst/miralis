// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use crate::ace::core::architecture::riscv::specification::{
    PMP_ADDRESS_SHIFT, PMP_CONFIG_SHIFT, PMP_OFF_MASK, PMP_PERMISSION_RWX_MASK, PMP_TOR_MASK,
};
use crate::ace::core::architecture::CSR;
use crate::ace::debug::__print_pmp_configuration;
use crate::ace::error::Error;
use crate::{debug, ensure};
use crate::host::MiralisContext;
use crate::virt::VirtContext;

// OpenSBI set already PMPs to isolate OpenSBI firmware from the rest of the
// system PMP0 protects OpenSBI memory region while PMP1 defines the system
// range We will use PMP0 and PMP1 to protect the confidential memory region,
// PMP2 to protect the OpenSBI, and PMP3 to define the system range.
pub fn split_memory_into_confidential_and_non_confidential(
    mctx: &mut MiralisContext,
    confidential_memory_start: usize,
    confidential_memory_end: usize,
) -> Result<(), Error> {
    // TODO: read how many PMPs are supported
    const MINIMUM_NUMBER_OF_PMP_REQUIRED: usize = 4;
    let number_of_pmps = 16;
    log::info!("Number of PMPs={}", number_of_pmps);
    ensure!(
        number_of_pmps >= MINIMUM_NUMBER_OF_PMP_REQUIRED,
        Error::NotEnoughPmps()
    )?;

    // TODO: simplify use of PMP by using a single PMP entry to isolate the confidential memory.
    // We assume here that the first two PMPs are not used by anyone else, e.g., OpenSBI firmware
    // MODIFIED CODE FOR MIRALIS
    mctx.pmp.set_pmpaddr(4, confidential_memory_start);
    mctx.pmp.set_pmpaddr( 5, confidential_memory_end);

    // CSR.pmpaddr4.write(confidential_memory_start >> PMP_ADDRESS_SHIFT);
    // CSR.pmpaddr5.write(confidential_memory_end >> PMP_ADDRESS_SHIFT);
    // END MODIFIED CODE
    close_access_to_confidential_memory();
    crate::ace::debug::__print_pmp_configuration();
    Ok(())
}

// 0x180000000 0x280000000
pub fn open_access_to_confidential_memory() {
    // MODIFIED CODE FOR MIRALIS
    let mask = (PMP_PERMISSION_RWX_MASK << 32) | ((PMP_TOR_MASK | PMP_PERMISSION_RWX_MASK) << 40);
    CSR.pmpcfg0.read_and_set_bits(mask);
    clear_caches();
    // END MODIFIED CODE
}

pub fn close_access_to_confidential_memory() {
    // MODIFIED CODE FOR MIRALIS
    let mask = (PMP_PERMISSION_RWX_MASK << 32) | ((PMP_PERMISSION_RWX_MASK) << 40);
    CSR.pmpcfg0.read_and_clear_bits(mask);
    clear_caches();
    // END MODIFIED CODE
}

fn clear_caches() {
    // See Section 3.7.2 of RISC-V privileged specification v1.12.
    // PMP translations can be cached and address translation can be done speculatively. Thus, it is adviced to flush caching structures.
    super::fence::sfence_vma();
    super::fence::hfence_gvma();
}
