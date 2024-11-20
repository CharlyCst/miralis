#![feature(asm)]

use core::arch::asm;

use crate::arch::pmp::pmpcfg;

/// Macro to read the `pmpcfgx` register where `x` is an argument (0-3).
/// Returns the value of the specified `pmpcfgx` register as a 64-bit unsigned integer.
fn read_pmpcfg(idx: usize) -> usize {
    let value: usize;
    unsafe {
        match idx {
            0 => asm!("csrr {}, pmpcfg0", out(reg) value),
            1 => asm!("csrr {}, pmpcfg1", out(reg) value),
            2 => asm!("csrr {}, pmpcfg2", out(reg) value),
            3 => asm!("csrr {}, pmpcfg3", out(reg) value),
            4 => asm!("csrr {}, pmpcfg4", out(reg) value),
            5 => asm!("csrr {}, pmpcfg5", out(reg) value),
            6 => asm!("csrr {}, pmpcfg6", out(reg) value),
            7 => asm!("csrr {}, pmpcfg7", out(reg) value),
            _ => panic!("Invalid pmpcfg index: {}", idx),
        }
    }
    value
}

pub fn read_pmpaddr(index: usize) -> u64 {
    if index >= 16 {
        panic!("Invalid PMP address register index: {}", index);
    }

    // The `pmpaddr` CSR indices start at 0x3B0 for `pmpaddr0`.
    let pmpaddr_base = 0x3B0;

    // Compute the CSR address for the specified `pmpaddr` register.
    let csr_address = pmpaddr_base + index;

    // Read the CSR value using inline assembly.
    let value: u64;
    unsafe {
        asm!(
        "csrr {value}, {csr}",
        csr = in(reg) csr_address,
        value = out(reg) value
        );
    }

    value
}

pub fn get_configuration(index: usize) -> u8 {
    let reg_idx = index / 8;
    let inner_idx = index % 8;
    let reg = read_pmpcfg(reg_idx);
    let cfg = (reg >> (inner_idx * 8)) & 0xff;
    cfg as u8
}

fn print_pmp() {
    let mut prev_addr2: u64 = 0;
    for idx in 0..16 {
        let cfg = get_configuration(idx);
        let addr = read_pmpaddr(idx);
        let prev_addr = prev_addr2;

        prev_addr2 = addr;

        match cfg & pmpcfg::A_MASK {
            pmpcfg::NA4 => {
                let addr = addr << 2;
                log::info!("Implement A-mask ;");
            }
            pmpcfg::NAPOT => {
                let trailing_ones = addr.trailing_ones();
                let addr_mask = !((1 << trailing_ones) - 1);
                let addr = (addr & addr_mask) << 2;
                let shift = trailing_ones + 3;
                log::info!(
                    " From - To : 0x{:x} --> 0x{:x} and permissions : 0x{:x}",
                    addr,
                    (1 << shift),
                    cfg & pmpcfg::RWX
                );
            }
            pmpcfg::TOR => {
                // if prev_addr is bigger then that entry does not match anything
                if prev_addr >= addr {
                    continue;
                }
                let size = addr - prev_addr;
                log::info!(
                    " From - To : 0x{:x} --> 0x{:x} and permissions : 0x{:x}",
                    prev_addr,
                    size,
                    cfg & pmpcfg::RWX
                );
            }
            _ => {
                log::info!("Inactive pmp entry");
                // Inactive PMP entry
                continue;
            }
        }
    }
}
