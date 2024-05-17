#![no_std]
//RISC-V PMP Configuration.

pub use log;

use crate::arch::pmp_csrs::{
    pmpaddr_csr_read, pmpaddr_csr_write, pmpcfg_csr_read, pmpcfg_csr_write,
};

//The following three constants assume 16 PMP entries.
pub const PMP_ENTRIES: usize = 16;
pub const PMP_CFG_ENTRIES: usize = 2;
//The number of PMP entries used to protect for instance memory mapped CSRs related to interrupts,
//in this case, 1 entry for SiFive CLINT (the highest priority entry)
//pub const FROZEN_PMP_ENTRIES: usize = 1;
pub const FROZEN_PMP_ENTRIES: usize = 0;

const PMP_CFG: usize = 0;
const PMP_ADDR: usize = 1;

const XWR_MASK: usize = 7;
const RV64_PAGESIZE_MASK: usize = 0xfffffffffffff000;

pub struct PMPWriteResponse {
    pub write_failed: bool,
    pub failure_code: PMPErrorCode,
    pub addressing_mode: PMPAddressingMode,
    pub addr1: usize,
    pub addr2: usize,
    pub cfg1: usize,
    pub cfg2: usize,
}

const EMPTY_PMP_WRITE_RESPONSE: PMPWriteResponse = PMPWriteResponse {
    write_failed: true,
    failure_code: PMPErrorCode::Uninitialized,
    addressing_mode: PMPAddressingMode::OFF,
    addr1: 0,
    addr2: 0,
    cfg1: 0,
    cfg2: 0,
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(usize)]
pub enum PMPAddressingMode {
    OFF = 0,
    TOR = 1,
    NA4 = 2,
    NAPOT = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(usize)]
pub enum PMPErrorCode {
    Uninitialized = 0,
    Success = 1, //TODO: Is it needed?
    InvalidCSRID = 2,
    NotPageAligned = 3,
    InvalidPermissions = 4,
    InvalidIndex = 5,
    InvalidCfg = 6,
}

fn compute_log2(num: usize) -> usize {
    let mut result: usize = 0;
    let mut value: usize = num;

    while value > 1 {
        value = value >> 1;
        result = result + 1;
    }

    result
}

//The csr_id is explicitly providing support to only read either pmpcfg or pmpaddr. Todo: Is this useful?
pub fn pmp_read(csr_id: usize, csr_index: usize) -> Result<usize, PMPErrorCode> {
    //This will ensure that the index is in expected range
    if csr_index >= PMP_ENTRIES {
        return Err(PMPErrorCode::InvalidIndex);
    }

    match csr_id {
        PMP_CFG => Ok(pmpcfg_read(csr_index)),
        PMP_ADDR => Ok(pmpaddr_read(csr_index)),
        _ => {
            return Err(PMPErrorCode::InvalidCSRID);
        }
    }
}

//Only computes the values to be written to the PMP, doesn't actually modify the PMPs.
//Returns the computed values in PMPWriteResponse.
pub fn pmp_write_compute(
    csr_index: usize,
    region_addr: usize,
    region_size: usize,
    region_perm: usize,
) -> PMPWriteResponse {
    log::trace!(
        "Computing PMPs with args: {}, {:x}, {:x}, {:x}",
        csr_index,
        region_addr,
        region_addr + region_size,
        region_perm
    );
    let mut pmp_write_response: PMPWriteResponse = EMPTY_PMP_WRITE_RESPONSE;
    //This will ensure that the index is in expected range
    if csr_index >= PMP_ENTRIES {
        pmp_write_response.failure_code = PMPErrorCode::InvalidIndex;
        return pmp_write_response;
    }

    //To enforce that the region_addr is the start of a page and the region_size is a multiple of
    //page_size.
    if ((region_addr & (RV64_PAGESIZE_MASK)) != region_addr)
        && ((region_size & (RV64_PAGESIZE_MASK)) != region_size)
    {
        log::debug!("PMP addr or size not page aligned!");
        pmp_write_response.failure_code = PMPErrorCode::NotPageAligned;
        return pmp_write_response;
    }

    if region_perm & XWR_MASK != XWR_MASK {
        pmp_write_response.failure_code = PMPErrorCode::InvalidPermissions;
        return pmp_write_response;
    }

    let mut pmpaddr: usize;
    let mut pmpcfg: usize;
    let mut log_2_region_size: usize = 0;

    /*if (region_size & (region_size - 1)) == 0 {
        log_2_region_size = compute_log2(region_size);
        if (log_2_region_size > 0)
            && ((region_addr >> 2) & ((1 << (log_2_region_size - 2)) - 1) == 0)
        {
            pmp_write_response.addressing_mode = PMPAddressingMode::NAPOT;
        }
    }

    //Determine addressing mode:
    //NAPOT addressing mode conditions: The region_addr must contain enough trailing zeroes to encode the region_size in the
    //pmpaddr register together with the address and the region_size is a power of two.
    if pmp_write_response.addressing_mode == PMPAddressingMode::NAPOT {
        log::trace!("NAPOT Addressing Mode csr_index {} region_size: {:x} log_2_region_size: {:x} addr: {:x}", csr_index, region_size, log_2_region_size, region_addr);
        let addrmask: usize = (1 << (log_2_region_size - 2)) - 1; //NAPOT encoding
        pmpaddr = (region_addr >> 2) & !addrmask;
        pmpaddr = pmpaddr | (addrmask >> 1); //To add the 0 before the 1s.

        pmpcfg = region_perm | ((pmp_write_response.addressing_mode as usize) << 3);

        match pmpcfg_compute(csr_index, pmpcfg) {
            Ok(val) => {
                pmp_write_response.cfg1 = val;
            }
            Err(code) => {
                pmp_write_response.failure_code = code;
                return pmp_write_response;
            }
        }
        pmp_write_response.addr1 = pmpaddr;
    } *///else {
    //TOR addressing mode    //TODO: NA4 addressing mode!
    log::trace!("TOR Addressing Mode csr_index: {}", csr_index);
    if csr_index == (PMP_ENTRIES - 1) {
        //Last PMP entry - Don't have enough PMP entries for protecting this region with TOR addressing mode.
        pmp_write_response.failure_code = PMPErrorCode::InvalidIndex;
        return pmp_write_response;
    }
    let csr_index_2: usize = csr_index + 1;
    //Initialize two PMP entries
    //First PMP entry (index i) contains the top address and pmpcfg value
    pmpaddr = region_addr + region_size;
    //>> 2; //TODO: Does this need to be generic or can we assume a fixed
    //PMP granularity?
    pmp_write_response.addressing_mode = PMPAddressingMode::TOR;
    pmpcfg = region_perm | ((pmp_write_response.addressing_mode as usize) << 3);
    log::trace!(
        "PMPADDR value {:x} for index {:x} PMPCFG value {:x}",
        pmpaddr,
        csr_index_2,
        pmpcfg
    );
    match pmpcfg_compute(csr_index_2, pmpcfg) {
        Ok(val) => {
            pmp_write_response.cfg2 = val;
        }
        Err(code) => {
            pmp_write_response.failure_code = code;
            return pmp_write_response;
        }
    }
    pmp_write_response.addr2 = pmpaddr >> 2;

    //Second PMP entry (index i-1) contains the bottom address and pmpcfg = 0
    pmpcfg = 0;
    pmpaddr = region_addr;
    log::trace!(
        "PMPADDR value {:x} for index {} PMPCFG value {:x}",
        pmpaddr,
        csr_index,
        pmpcfg
    );
    match pmpcfg_compute(csr_index, pmpcfg) {
        Ok(val) => {
            pmp_write_response.cfg1 = val;
        }
        Err(code) => {
            pmp_write_response.failure_code = code;
            return pmp_write_response;
        }
    }
    pmp_write_response.addr1 = pmpaddr >> 2;
    // }

    pmp_write_response.write_failed = false;
    return pmp_write_response;
}

//Returns read value
fn pmpaddr_read(index: usize) -> usize {
    let pmpaddr: usize;
    pmpaddr = pmpaddr_csr_read(index);
    return pmpaddr;
}

//Returns read value from pmpcfg[n]
pub fn pmpcfg_read(index: usize) -> usize {
    let mut pmpcfg: usize;

    //Need to extract the pmpcfg value based on the index. Assumes 8 bit pmpcfg as specified in the
    //RV Specification Vol 2 - Privileged Arch.

    pmpcfg = pmpcfg_csr_read(index);

    let index_pos: usize = index % 8;
    let pmpcfg_mask: usize = 0xff << (index_pos * 8);

    pmpcfg = (pmpcfg & pmpcfg_mask) >> (index_pos * 8);

    return pmpcfg;
}

fn pmpcfg_compute(index: usize, value: usize) -> Result<usize, PMPErrorCode> {
    let pmpcfg: usize;
    let index_pos: usize = index % 8;

    if (value & !(0xff)) != 0 {
        log::debug!("Invalid pmpcfg value!");
        return Err(PMPErrorCode::InvalidCfg);
    }

    pmpcfg = value << (index_pos * 8);

    log::trace!(
        "Computed for index: {:x} pmpcfg: {:x} value: {:x}",
        index,
        pmpcfg,
        value
    );

    return Ok(pmpcfg);
}

#[allow(dead_code)]
pub fn pmpcfg_write(index: usize, value: usize) -> Result<usize, PMPErrorCode> {
    let mut pmpcfg: usize;
    let index_pos: usize = index % 8;
    let pmpcfg_mask: usize = 0xff << (index_pos * 8);

    if (value & !(0xff)) != 0 {
        log::debug!("Invalid pmpcfg value!");
        return Err(PMPErrorCode::InvalidCfg);
    }

    pmpcfg = pmpcfg_csr_read(index);

    pmpcfg = pmpcfg & !(pmpcfg_mask);

    pmpcfg = pmpcfg | (value << (index_pos * 8));

    log::trace!(
        "Computed for index: {:x} pmpcfg: {:x} value: {:x}",
        index,
        pmpcfg,
        value
    );
    //pmpcfg_csr_write(index, pmpcfg);

    return Ok(pmpcfg);

    //Sfence after writing the PMP.
    //unsafe {
    //    asm!("sfence.vma");
    //}

    //return PMPErrorCode::Success;
}

pub fn clear_pmp() {
    for n in FROZEN_PMP_ENTRIES..PMP_ENTRIES {
        pmpaddr_csr_write(n, 0);
        pmpcfg_csr_write(n, 0); //Note: This only works because the frozen_pmp entry we have
                                //has pmpcfg = 0. If that wasn't the case, clearing would
                                //require fine-grained writes to pmpcfg.
    }
}
