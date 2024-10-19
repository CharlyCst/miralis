// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0

// TODO: these constants should be generated automatically from the RISC-V formal spec
pub const PAGE_TABLE_ENTRY_VALID_BIT: usize = 0;
pub const PAGE_TABLE_ENTRY_VALID_MASK: usize = 1 << PAGE_TABLE_ENTRY_VALID_BIT;
pub const PAGE_TABLE_ENTRY_READ_BIT: usize = 1;
pub const PAGE_TABLE_ENTRY_READ_MASK: usize = 1 << PAGE_TABLE_ENTRY_READ_BIT;
pub const PAGE_TABLE_ENTRY_WRITE_BIT: usize = 2;
pub const PAGE_TABLE_ENTRY_WRITE_MASK: usize = 1 << PAGE_TABLE_ENTRY_WRITE_BIT;
pub const PAGE_TABLE_ENTRY_EXECUTE_BIT: usize = 3;
pub const PAGE_TABLE_ENTRY_EXECUTE_MASK: usize = 1 << PAGE_TABLE_ENTRY_EXECUTE_BIT;
pub const PAGE_TABLE_ENTRY_USER_BIT: usize = 4;
pub const PAGE_TABLE_ENTRY_USER_MASK: usize = 1 << PAGE_TABLE_ENTRY_USER_BIT;
pub const PAGE_TABLE_ENTRY_GLOBAL_BIT: usize = 5;
pub const PAGE_TABLE_ENTRY_GLOBAL_MASK: usize = 1 << PAGE_TABLE_ENTRY_GLOBAL_BIT;
pub const PAGE_TABLE_ENTRY_ACCESSED_BIT: usize = 6;
pub const PAGE_TABLE_ENTRY_ACCESSED_MASK: usize = 1 << PAGE_TABLE_ENTRY_ACCESSED_BIT;
pub const PAGE_TABLE_ENTRY_DIRTY_BIT: usize = 7;
pub const PAGE_TABLE_ENTRY_DIRTY_MASK: usize = 1 << PAGE_TABLE_ENTRY_DIRTY_BIT;

pub const PAGE_TABLE_ENTRY_EMPTY_CONF: usize = 0;
pub const PAGE_TABLE_ENTRY_UAD_CONF_MASK: usize =
    PAGE_TABLE_ENTRY_USER_MASK | PAGE_TABLE_ENTRY_ACCESSED_MASK | PAGE_TABLE_ENTRY_DIRTY_MASK;

pub const PAGE_TABLE_ENTRY_NO_PERMISSIONS: usize = 0;
pub const PAGE_TABLE_ENTRY_RW_PERMISSIONS: usize =
    PAGE_TABLE_ENTRY_READ_MASK | PAGE_TABLE_ENTRY_WRITE_MASK;
pub const PAGE_TABLE_ENTRY_RWX_PERMISSIONS: usize =
    PAGE_TABLE_ENTRY_READ_MASK | PAGE_TABLE_ENTRY_WRITE_MASK | PAGE_TABLE_ENTRY_EXECUTE_MASK;

pub const PAGE_TABLE_ENTRY_TYPE_MASK: usize = PAGE_TABLE_ENTRY_VALID_MASK
    | PAGE_TABLE_ENTRY_READ_MASK
    | PAGE_TABLE_ENTRY_WRITE_MASK
    | PAGE_TABLE_ENTRY_EXECUTE_MASK;
pub const PAGE_TABLE_ENTRY_NOT_MAPPED: usize = 0;
pub const PAGE_TABLE_ENTRY_POINTER: usize = PAGE_TABLE_ENTRY_VALID_MASK;

pub const CONFIGURATION_BIT_MASK: usize = 0x3ff; // first 10 bits
pub const ADDRESS_SHIFT: usize = 2;

pub const HGATP64_MODE_SHIFT: usize = 60;
pub const HGATP64_VMID_SHIFT: usize = 44;
pub const HGATP_PAGE_SHIFT: usize = 12;
pub const HGATP_PPN_MASK: usize = 0x0000FFFFFFFFFFF;

pub const HGATP_MODE_SV57X4: usize = 10;
