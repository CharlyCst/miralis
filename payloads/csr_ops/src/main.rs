#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    log::debug!("Testing mscratch register");
    test_mscratch();
    log::debug!("Testing CSR operations");
    test_csr_op();
    log::debug!("Testing CSR ID registers");
    test_csr_id();
    log::debug!("Done!");
    success();
}

// ———————————————————————————— Simple Mscratch ————————————————————————————— //

/// Test the mscratch register with a simple read and write
fn test_mscratch() {
    let res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mscratch, {0}",
            "csrr {1}, mscratch",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0x42);
}

// ————————————————————————————— CSR Operations ————————————————————————————— //

/// The fixed immediate for instructions with hard-coded immediate.
const IMMEDIATE: usize = 27;

/// Test the CSR registers operations (csrrw, csrrs, csrrc, csrrwi, csrrsi, csrrci).
fn test_csr_op() {
    // List of vallues to try the CSR operations with
    let regs = &[
        (0, 0, 0),
        (1, 2, 3),
        (1, 2, 4),
        (0xffff, 0x9999, 0x5555),
        (usize::MAX, 0, usize::MAX),
        (1234567, 7890123, 567890),
    ];
    // CSRRW
    log::trace!("Testing CSSRW");
    for (in_rd, in_rs1, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrw(*in_csr, *in_rd, *in_rs1) };
        check_csrrw(*in_rs1, *in_csr, out_csr, out_rd);
    }
    // CSRRS
    log::trace!("Testing CSSRS");
    for (in_rd, in_rs1, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrs(*in_csr, *in_rd, *in_rs1) };
        check_csrrs(*in_rs1, *in_csr, out_csr, out_rd);
    }
    // CSRRC
    log::trace!("Testing CSSRC");
    for (in_rd, in_rs1, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrc(*in_csr, *in_rd, *in_rs1) };
        check_csrrc(*in_rs1, *in_csr, out_csr, out_rd);
    }
    // CSRRWI
    log::trace!("Testing CSSRWI");
    for (in_rd, _, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrwi(*in_csr, *in_rd) };
        check_csrrwi(*in_csr, out_csr, out_rd);
    }
    // CSRRSI
    log::trace!("Testing CSSRSI");
    for (in_rd, _, in_csr) in regs {
        let (out_csr, out_rd) = unsafe { csrrsi(*in_csr, *in_rd) };
        check_csrrsi(*in_csr, out_csr, out_rd);
    }
    // CSRRCI
    log::trace!("Testing CSSRCI");
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

// ———————————————————————————— CSR ID registers ———————————————————————————— //

/// Test CSR ID registers
///
/// For now, they should all be zero.
fn test_csr_id() {
    let mut res: usize;
    unsafe {
        asm!(
            "csrr {0}, mvendorid",
            out(reg) res
        );
    };
    assert_eq!(res, 0, "Invalid mvendorid");

    unsafe {
        asm!(
            "csrr {0}, marchid",
            out(reg) res
        );
    };
    assert_eq!(res, 0, "Invalid marchid");

    unsafe {
        asm!(
            "csrr {0}, mimpid",
            out(reg) res
        );
    };
    assert_eq!(res, 0, "Invalid mimpid");

    unsafe {
        asm!(
            "csrr {0}, mhartid",
            out(reg) res
        );
    };
    assert_eq!(res, 0, "Invalid mhartid");
}
