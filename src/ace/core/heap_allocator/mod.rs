// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use allocator::HeapAllocator;

use crate::ace::core::memory_layout::ConfidentialMemoryAddress;
mod allocator;

/// global allocator allocates memory on the security monitor's heap.
#[global_allocator]
static mut HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::empty();

pub(super) fn init_heap(start_address: ConfidentialMemoryAddress, heap_size: usize) {
    log::info!(
        "Heap {:x}-{:x}",
        start_address.as_usize(),
        start_address.as_usize() + heap_size
    );
    unsafe {
        #[allow(static_mut_refs)]
        HEAP_ALLOCATOR
            .lock()
            .add_free_memory_region(start_address.into_mut_ptr(), heap_size);
    }
}
