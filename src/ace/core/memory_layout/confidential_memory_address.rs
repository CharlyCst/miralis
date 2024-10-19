// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use pointers_utility::{ptr_byte_add_mut, ptr_byte_offset};

use crate::ace::error::Error;

/// The wrapper over a raw pointer that is guaranteed to be an address located in the confidential memory region.
#[repr(transparent)]
#[derive(Debug, PartialEq)]
pub struct ConfidentialMemoryAddress(*mut usize);

impl ConfidentialMemoryAddress {
    /// Precondition: The global memory layout is initialized.
    /// Precondition: The address is in the confidential region of the global memory layout.
    pub(super) fn new(address: *mut usize) -> Self {
        Self(address)
    }

    // TODO: check if needed. If yes, make sure the raw pointer is not used incorrectly
    // Currently we only use it during creation of the heap allocator structure. It
    // would be good to get rid of this because it requires extra safety guarantees for
    // parallel execution of the security monitor
    pub unsafe fn into_mut_ptr(self) -> *mut usize {
        self.0
    }

    pub unsafe fn to_ptr(&self) -> *const u8 {
        self.0 as *const u8
    }

    pub fn as_usize(&self) -> usize {
        // TODO: check if we need to expose the pointer.
        // If not, use addr() instead.
        // self.0.addr()
        self.0 as usize
    }

    /// Postcondition: Verifies that the pointer is aligned to the given alignment.
    pub fn is_aligned_to(&self, align: usize) -> bool {
        self.0.is_aligned_to(align)
    }

    /// Postcondition: Compute the offset.
    pub fn offset_from(&self, pointer: *const usize) -> isize {
        ptr_byte_offset(pointer, self.0)
    }

    /// Creates a new confidential memory address at given offset. Error is returned if the resulting address exceeds
    /// the upper boundary.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the address at given offset is still within the confidential memory region.
    // TODO: can we require the offset to be a multiple of usize?
    /// Precondition: The offset address is in the given range.
    /// Precondition: The global memory layout is initialized.
    /// Precondition: The maximum (and thus the offset address) is in the confidential memory range.
    /// Postcondition: The offset pointer is in the confidential memory range.
    pub unsafe fn add(
        &self,
        offset_in_bytes: usize,
        upper_bound: *const usize,
    ) -> Result<ConfidentialMemoryAddress, Error> {
        let pointer = ptr_byte_add_mut(self.0, offset_in_bytes, upper_bound)
            .map_err(|_| Error::AddressNotInConfidentialMemory())?;
        Ok(ConfidentialMemoryAddress(pointer))
    }

    /// Reads usize-sized sequence of bytes from the confidential memory region.
    /// # Safety
    ///
    /// Caller must ensure that the pointer is not used by two threads simultaneously and that it is correctly aligned for usize.
    /// See `ptr::read_volatile` for safety concerns
    // TODO: currently only_spec because shim registration for read_volatile doesn't work
    // TODO require that lifetime [lft_el] is actually alive
    pub unsafe fn read_volatile<'a>(&'a self) -> usize {
        self.0.read_volatile()
    }

    /// Writes usize-sized sequence of bytes to the confidential memory region.
    /// # Safety
    ///
    /// Caller must ensure that the pointer is not used by two threads simultaneously and that it is correctly aligned for usize.
    /// See `ptr::write_volatile` for safety concerns
    // TODO: currently only_spec because shim registration for write_volatile doesn't work
    pub unsafe fn write_volatile(&self, value: usize) {
        self.0.write_volatile(value);
    }
}
