// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use pointers_utility::ptr_byte_add_mut;

use crate::ace::core::memory_layout::MemoryLayout;
use crate::ace::error::Error;

/// The wrapper over a raw pointer that is guaranteed to be an address located in the non-confidential memory region.
#[repr(transparent)]
#[derive(Debug)]
pub struct NonConfidentialMemoryAddress(*mut usize);

/// Model: The memory address is represented by the location in memory.
/// We require the ghost state for the global memory layout to be available.
/// Invariant: The global memory layout has been initialized.
/// Invariant: The address is in non-confidential memory.
impl NonConfidentialMemoryAddress {
    /// Constructs an address in a non-confidential memory. Returns error if the address is outside non-confidential
    /// memory.
    /// Precondition: The global memory layout has been initialized.
    /// Precondition: The location is in non-confidential memory.
    /// Postcondition: The non-confidential memory address is correctly initialized.
    pub fn new(address: *mut usize) -> Result<Self, Error> {
        match MemoryLayout::read().is_in_non_confidential_range(address) {
            false => Err(Error::AddressNotInNonConfidentialMemory()),
            true => Ok(Self(address)),
        }
    }

    /// Creates a new non-confidential memory address at given offset. Returns error if the resulting address exceeds
    /// the upper boundary.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the address advanced by the offset is still within the non-confidential memory
    /// region.
    // TODO: can we require the offset to be a multiple of usize?
    /// Precondition: The offset address is in the given range.
    /// Precondition: The global memory layout is initialized.
    /// Precondition: The maximum (and thus the offset address) is in the non-confidential memory range.
    /// Postcondition: The offset pointer is in the non-confidential memory range.
    pub unsafe fn add(
        &self,
        offset_in_bytes: usize,
        upper_bound: *const usize,
    ) -> Result<NonConfidentialMemoryAddress, Error> {
        let pointer = ptr_byte_add_mut(self.0, offset_in_bytes, upper_bound)
            .map_err(|_| Error::AddressNotInNonConfidentialMemory())?;
        Ok(NonConfidentialMemoryAddress(pointer))
    }

    /// Reads usize-sized sequence of bytes from the non-confidential memory region.
    ///
    /// # Safety
    ///
    /// We need to ensure the pointer is not used by two threads simultaneously. See `ptr::read_volatile` for safety
    /// concerns.
    pub unsafe fn read(&self) -> usize {
        self.0.read_volatile()
    }

    /// Writes usize-sized sequence of bytes to the non-confidential memory region.
    ///
    /// # Safety
    ///
    /// We need to ensure the pointer is not used by two threads simultaneously. See `ptr::write_volatile` for safety
    /// concerns.
    pub unsafe fn write(&self, value: usize) {
        self.0.write_volatile(value);
    }

    pub fn as_ptr(&self) -> *const usize {
        self.0
    }

    pub fn usize(&self) -> usize {
        // TODO: check if we need to expose the pointer.
        // If not, use addr() instead.
        // self.0.addr()
        self.0 as usize
    }
}
