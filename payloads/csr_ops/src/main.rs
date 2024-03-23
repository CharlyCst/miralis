#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_payload, success};

setup_payload!(main);

/// The fixed immediate for instructions with hard-coded immediate.
const IMMEDIATE: usize = 27;

fn main() -> ! {
    let regs = [
        (0, 0, 0),
        (1, 2, 3),
        (1, 2, 4),
        (0xffff, 0x9999, 0x5555),
        (usize::MAX, 0, usize::MAX),
        (1234567, 7890123, 567890),
    ];

    test_csr_op(&regs);
    success();
}

fn test_csr_op(regs: &[(usize, usize, usize)]) {
    // CSRRW
    for (in_rd, in_rs1, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrw(*in_csr, *in_rd, *in_rs1) };
        check_csrrw(*in_rs1, *in_csr, out_csr, out_rd);
    }
    // CSRRS
    for (in_rd, in_rs1, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrs(*in_csr, *in_rd, *in_rs1) };
        check_csrrs(*in_rs1, *in_csr, out_csr, out_rd);
    }
    // CSRRWI
    for (in_rd, _, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrwi(*in_csr, *in_rd) };
        check_csrrwi(*in_csr, out_csr, out_rd);
    }
    // CSRRSI
    for (in_rd, _, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrsi(*in_csr, *in_rd) };
        check_csrrsi(*in_csr, out_csr, out_rd);
    }
    // CSRRC
    for (in_rd, in_rs1, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrc(*in_csr, *in_rd, *in_rs1) };
        check_csrrc(*in_rs1, *in_csr, out_csr, out_rd);
    }
    // CSRRCI
    for (in_rd, _, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrci(*in_csr, *in_rd) };
        check_csrrci(*in_csr, out_csr, out_rd);
    }
}

fn check_csrrw(in_rs1: usize, in_csr: usize, out_csr: usize, out_rd: usize) {
    assert_eq!(out_csr, in_rs1);
    assert_eq!(out_rd, in_csr);
}

unsafe fn csrrw(csr: usize, rd: usize, rs1: usize) -> (usize, usize) {
    let mut rd = rd;
    let mut csr = csr;
    asm!(
        "csrw mscratch, {2}",       // Initialize mscratch
        "csrrw {0}, mscratch, {1}", // Perform the CSR operation
        "csrr {2}, mscratch",       // Retrieve new mscratch value
        inout(reg) rd,
        in(reg) rs1,
        inout(reg) csr,
    );

    (csr, rd)
}

fn check_csrrs(in_rs1: usize, in_csr: usize, out_csr: usize, out_rd: usize) {
    assert_eq!(out_csr, in_csr | in_rs1);
    assert_eq!(out_rd, in_csr);
}

unsafe fn csrrs(csr: usize, rd: usize, rs1: usize) -> (usize, usize) {
    let mut rd = rd;
    let mut csr = csr;
    asm!(
        "csrw mscratch, {2}",       // Initialize mscratch
        "csrrs {0}, mscratch, {1}", // Perform the CSR operation
        "csrr {2}, mscratch",       // Retrieve new mscratch value
        inout(reg) rd,
        in(reg) rs1,
        inout(reg) csr,
    );

    (csr, rd)
}

fn check_csrrwi(in_csr: usize, out_csr: usize, out_rd: usize) {
    assert_eq!(out_csr, IMMEDIATE);
    assert_eq!(out_rd, in_csr);
}

unsafe fn csrrwi(csr: usize, rd: usize) -> (usize, usize) {
    let mut rd = rd;
    let mut csr = csr;
    asm!(
        "csrw mscratch, {1}",       // Initialize mscratch
        "csrrwi {0}, mscratch, 27", // Perform the CSR operation
        "csrr {1}, mscratch",       // Retrieve new mscratch value
        inout(reg) rd,
        inout(reg) csr,
    );

    (csr, rd)
}

fn check_csrrsi(in_csr: usize, out_csr: usize, out_rd: usize) {
    assert_eq!(out_csr, in_csr | IMMEDIATE);
    assert_eq!(out_rd, in_csr);
}

unsafe fn csrrsi(csr: usize, rd: usize) -> (usize, usize) {
    let mut rd = rd;
    let mut csr = csr;
    asm!(
        "csrw mscratch, {1}",       // Initialize mscratch
        "csrrsi {0}, mscratch, 27", // Perform the CSR operation
        "csrr {1}, mscratch",       // Retrieve new mscratch value
        inout(reg) rd,
        inout(reg) csr,
    );

    (csr, rd)
}

fn check_csrrc(in_rs1: usize, in_csr: usize, out_csr: usize, out_rd: usize) {
    assert_eq!(out_csr, in_csr & !in_rs1);
    assert_eq!(out_rd, in_csr);
}

unsafe fn csrrc(csr: usize, rd: usize, rs1: usize) -> (usize, usize) {
    let mut rd = rd;
    let mut csr = csr;
    asm!(
        "csrw mscratch, {2}",       // Initialize mscratch
        "csrrc {0}, mscratch, {1}", // Perform the CSR operation
        "csrr {2}, mscratch",       // Retrieve new mscratch value
        inout(reg) rd,
        in(reg) rs1,
        inout(reg) csr,
    );

    (csr, rd)
}

fn check_csrrci(in_csr: usize, out_csr: usize, out_rd: usize) {
    assert_eq!(out_csr, in_csr & !IMMEDIATE);
    assert_eq!(out_rd, in_csr);
}

unsafe fn csrrci(csr: usize, rd: usize) -> (usize, usize) {
    let mut rd = rd;
    let mut csr = csr;
    asm!(
        "csrw mscratch, {1}",       // Initialize mscratch
        "csrrci {0}, mscratch, 27", // Perform the CSR operation
        "csrr {1}, mscratch",       // Retrieve new mscratch value
        inout(reg) rd,
        inout(reg) csr,
    );

    (csr, rd)
}
