#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use miralis_abi::{setup_binary, success};

setup_binary!(main);

#[repr(C, align(4096))]
struct Page([usize; 512]);
static LOCKED_PAGE: Page = Page([0xdead_beef; 512]);

fn main() -> ! {
    let secret_addr: usize = 0x0000000000000042;
    let secret_cfg: usize = 0b00000111;
    let mut res: usize;

    // Test normal write to config
    unsafe {
        asm!(
            "csrw pmpcfg0, {0}",
            "csrr {1}, pmpcfg0",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }
    assert_eq!(res, secret_cfg, "Could not write pmpcfg0");

    // Test invalid write to config
    unsafe {
        asm!(
            "csrw pmpcfg0, {0}",
            "csrr {1}, pmpcfg0",
            in(reg) 0b01100111,
            out(reg) res,
        );
    }
    assert_eq!(res, secret_cfg, "Could set invalid bits in pmpcfg0");

    // Test implemented but inactive config CSR (pmpcfg2 covers entries 8..15, with 8 virtual PMPs)
    unsafe {
        asm!(
            "csrw pmpcfg2, {0}",
            "csrr {1}, pmpcfg2",
            in(reg) 0b01100111,
            out(reg) res,
        );
    }
    assert_eq!(res, 0, "Could write to read-only zero pmpcfg2");

    // Test normal write to address
    unsafe {
        asm!(
            "csrw pmpaddr0, {0}",
            "csrr {1}, pmpaddr0",
            in(reg) 0x42,
            out(reg) res,
        );
    }
    assert_eq!(res, secret_addr, "Could not write to pmpaddr0");

    // Test invalid write to address
    unsafe {
        asm!(
            "li {0}, 0xf000000000000042",
            "csrw pmpaddr0, {0}",
            "csrr {1}, pmpaddr0",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }
    assert_eq!(res, secret_addr, "Could write an invalid address");

    // Test implemented but inactive address CSR (pmpaddr9 < 16 implemented PMPs, with 8 virtual PMPs)
    unsafe {
        asm!(
            "csrw pmpaddr9, {0}",
            "csrr {1}, pmpaddr9",
            in(reg) 0x42,
            out(reg) res,
        );
    }
    assert_eq!(res, 0, "Could write to read-only zero pmpaddr9");

    // Test unimplemented or hardwired-zero PMP CSRs (>= 16: read-only 0 on 64-PMP cores, trap on
    // 16-PMP cores) and odd RV64 pmpcfg1 (always traps on RV64)
    let mut mcause: usize;
    unsafe {
        let handler = _pmp_test_trap_handler as *const () as usize;
        asm!(
            "csrw mtvec, {handler}",
            "li t6, 0",
            ".option push",
            ".option norvc",
            "csrw pmpcfg4, {val}",
            "csrr {res}, pmpcfg4",
            ".option pop",
            handler = in(reg) handler,
            val = in(reg) 0x07usize,
            res = inout(reg) 0usize => res,
            out("t4") _,
            out("t5") _,
            out("t6") mcause,
        );
        if mcause != 0 {
            assert_eq!(mcause, 2, "Expected illegal instruction on pmpcfg4");
        } else {
            assert_eq!(res, 0, "Could write to read-only zero pmpcfg4");
        }

        asm!(
            "li t6, 0",
            ".option push",
            ".option norvc",
            "csrw pmpaddr17, {val}",
            "csrr {res}, pmpaddr17",
            ".option pop",
            val = in(reg) 0x42usize,
            res = inout(reg) 0usize => res,
            out("t4") _,
            out("t5") _,
            out("t6") mcause,
        );
        if mcause != 0 {
            assert_eq!(mcause, 2, "Expected illegal instruction on pmpaddr17");
        } else {
            assert_eq!(res, 0, "Could write to read-only zero pmpaddr17");
        }

        asm!(
            "li t6, 0",
            ".option push",
            ".option norvc",
            "csrr {res}, 0x3a1",
            ".option pop",
            res = out(reg) _,
            out("t4") _,
            out("t5") _,
            out("t6") mcause,
        );
        assert_eq!(mcause, 2, "Odd RV64 pmpcfg1 did not trap");
    }

    // Test PMP lock bit (L = 1) write protection and M-mode enforcement
    let locked_base = &raw const LOCKED_PAGE as usize;
    let locked_napot = (locked_base >> 2) | ((4096 / 8) - 1);
    let locked_ro_cfg: usize = 0x80 | 0x18 | 0x01; // L | NAPOT | R
    let mtval: usize;
    unsafe {
        asm!(
            "csrw pmpaddr0, {addr}",
            "csrw pmpcfg0, {cfg}",
            // Locked entry ignores subsequent writes to pmpcfg0 and pmpaddr0
            "csrw pmpcfg0, zero",
            "csrw pmpaddr0, zero",
            "csrr {res}, pmpcfg0",
            addr = in(reg) locked_napot,
            cfg = in(reg) locked_ro_cfg,
            res = out(reg) res,
        );
        assert_eq!(res, locked_ro_cfg, "Locked pmpcfg0 was overwritten");

        // M-mode load succeeds, M-mode store faults on locked R-only entry
        let loaded = core::ptr::read_volatile(locked_base as *const usize);
        assert_eq!(loaded, 0xdead_beef);

        asm!(
            "li t6, 0",
            ".option push",
            ".option norvc",
            "sd {val}, 0({ptr})",
            ".option pop",
            val = in(reg) 0x1234usize,
            ptr = in(reg) locked_base,
            out("t4") mtval,
            out("t5") _,
            out("t6") mcause,
        );
        assert_eq!(
            mcause, 7,
            "Store to locked read-only PMP entry did not fault"
        );
        assert_eq!(mtval, locked_base, "Store fault reported wrong mtval");
    }

    success();
}

global_asm!(
    r#"
.text
.balign 4
.global _pmp_test_trap_handler
_pmp_test_trap_handler:
    csrr t6, mcause
    csrr t4, mtval
    csrr t5, mepc
    addi t5, t5, 4
    csrw mepc, t5
    mret
"#,
);

unsafe extern "C" {
    fn _pmp_test_trap_handler();
}
