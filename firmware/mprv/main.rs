//! MPRV test
//!
//! This test firmware checks the emulation of loads and stores with mstatus.MPRV = 1.
//!
//! With MPRV = 1 and MPP != M, loads and stores use the privilege in MPP, and Miralis emulates
//! them one by one. With MPP = M, MPRV has no effect. A trap sets MPP = M but preserves MPRV, so
//! the trap handler runs with MPRV = 1 and its loads and stores must not be emulated.

#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::ptr;

use miralis_abi::{setup_binary, success};

setup_binary!(main);

const MPRV: usize = 1 << 17;
const MPP_OFFSET: usize = 11;
const MPP_MASK: usize = 0b11 << MPP_OFFSET;

const MODE_U: usize = 0b00;
const MODE_S: usize = 0b01;
const MODE_M: usize = 0b11;

const LOAD_ACCESS_FAULT: usize = 5;
const STORE_ACCESS_FAULT: usize = 7;
const LOAD_PAGE_FAULT: usize = 13;
const STORE_PAGE_FAULT: usize = 15;

const SUM: usize = 1 << 18;
const MXR: usize = 1 << 19;

/// An address within Miralis's memory.
const MIRALIS_ADDR: usize = 0x80002000;

const GOOD_VALUE: usize = 0x1234_5678_9abc_def0;
const DENIED_VALUE: usize = 0x0fed_cba9_8765_4321;

/// A page accessible with any privilege.
static mut GOOD: usize = GOOD_VALUE;

/// A page denied to S and U-mode through the virtual PMP.
#[repr(C, align(4096))]
struct Page([usize; 512]);
static DENIED: Page = Page([DENIED_VALUE; 512]);
static mut ROOT_PT: Page = Page([0; 512]);

/// Written by the trap handler, with MPRV = 1 and MPP = M.
#[unsafe(no_mangle)]
static mut MPRV_TEST_SCRATCH: usize = 0;

fn main() -> ! {
    let good = &raw mut GOOD as usize;
    let denied = &raw const DENIED as usize;

    unsafe {
        // Install the trap handler
        asm!(
            "csrw mtvec, {handler}",
            handler = in(reg) _mprv_test_trap_handler as *const () as usize,
        );

        // Deny the DENIED page to S and U-mode (pmp0) and allow everything else (pmp1)
        let napot = 0b11 << 3;
        let rwx = 0b111;
        asm!(
            "csrw pmpaddr0, {denied}",
            "csrw pmpaddr1, {all}",
            "csrw pmpcfg0, {cfg}",
            denied = in(reg) (denied >> 2) | ((4096 / 8) - 1),
            all = in(reg) usize::MAX,
            cfg = in(reg) napot | ((napot | rwx) << 8),
        );
    }

    // MPP = S or U: accesses are emulated with the privileges of MPP
    let access = mprv_load(good, MODE_S);
    assert_eq!(access.mcause, 0, "MPRV load with MPP = S trapped");
    assert_eq!(
        access.value, GOOD_VALUE,
        "MPRV load with MPP = S read a wrong value"
    );

    let access = mprv_store(good, MODE_S, 0x42);
    assert_eq!(access.mcause, 0, "MPRV store with MPP = S trapped");
    assert_eq!(
        unsafe { ptr::read_volatile(good as *const usize) },
        0x42,
        "MPRV store with MPP = S did not write memory"
    );

    let access = mprv_load(good, MODE_U);
    assert_eq!(access.mcause, 0, "MPRV load with MPP = U trapped");
    assert_eq!(
        access.value, 0x42,
        "MPRV load with MPP = U read a wrong value"
    );

    // MPP = S: the virtual PMP applies, faults are forwarded to the firmware. The trap sets
    // MPP = M and keeps MPRV = 1, the trap handler then performs a store and a load which must
    // succeed.
    let access = mprv_load(denied, MODE_S);
    access.check_fault(LOAD_ACCESS_FAULT, denied, "MPRV load from a denied page");

    let access = mprv_store(denied, MODE_S, 0x42);
    access.check_fault(STORE_ACCESS_FAULT, denied, "MPRV store to a denied page");

    let access = mprv_load(MIRALIS_ADDR, MODE_S);
    access.check_fault(LOAD_ACCESS_FAULT, MIRALIS_ADDR, "MPRV load from Miralis");

    let access = mprv_store(MIRALIS_ADDR, MODE_S, 0x42);
    access.check_fault(STORE_ACCESS_FAULT, MIRALIS_ADDR, "MPRV store to Miralis");

    // MPP = M: MPRV has no effect, the virtual PMP does not apply (the entries are not locked),
    // but Miralis must still be protected.
    let access = mprv_load(denied, MODE_M);
    assert_eq!(access.mcause, 0, "MPRV load with MPP = M trapped");
    assert_eq!(
        access.value, DENIED_VALUE,
        "MPRV load with MPP = M read a wrong value"
    );

    let access = mprv_load(MIRALIS_ADDR, MODE_M);
    access.check_fault(
        LOAD_ACCESS_FAULT,
        MIRALIS_ADDR,
        "MPRV load from Miralis, MPP = M",
    );

    let access = mprv_store(MIRALIS_ADDR, MODE_M, 0x42);
    access.check_fault(
        STORE_ACCESS_FAULT,
        MIRALIS_ADDR,
        "MPRV store to Miralis, MPP = M",
    );

    // Compressed stack-relative load/store (c.sdsp / c.ldsp) under MPRV = 1, MPP = S
    unsafe {
        let mut loaded_sp: usize;
        asm!(
            "mv t4, sp",
            "mv sp, {addr}",
            "csrr t5, mstatus",
            "and t5, t5, {clear}",
            "or t5, t5, {set}",
            "csrw mstatus, t5",
            ".option push",
            ".option rvc",
            "c.sdsp a1, 0(sp)",
            "c.ldsp a0, 0(sp)",
            ".option pop",
            "csrc mstatus, {mprv}",
            "mv sp, t4",
            addr = in(reg) good,
            clear = in(reg) !MPP_MASK,
            set = in(reg) (MODE_S << MPP_OFFSET) | MPRV,
            mprv = in(reg) MPRV,
            in("a1") 0xfeed_cafe_usize,
            out("a0") loaded_sp,
            out("t4") _,
            out("t5") _,
        );
        assert_eq!(loaded_sp, 0xfeed_cafe, "c.sdsp/c.ldsp under MPRV failed");
        ptr::write_volatile(good as *mut usize, GOOD_VALUE);
    }

    // Sv39 translation, SUM, MXR, virtual-address device collision, and page-fault propagation
    unsafe {
        let root_pa = &raw mut ROOT_PT as usize;
        let gigapage_ppn = (0x8000_0000_usize >> 12) << 10;
        // VPN[2] = 2 (0x8000_0000..0xc000_0000): U-accessible RW gigapage (V|R|W|U|A|D = 0xd7)
        ROOT_PT.0[2] = gigapage_ppn | 0xd7;
        // VPN[2] = 3 (0xc000_0000..0x1_0000_0000): S-mode X-only gigapage (V|X|A|D = 0xc9)
        ROOT_PT.0[3] = gigapage_ppn | 0xc9;
        let satp_sv39 = (8usize << 60) | (root_pa >> 12);
        asm!("csrw satp, {satp}", "sfence.vma", satp = in(reg) satp_sv39);

        // Unmapped VA 0x2000000 (collides numerically with physical CLINT) must page-fault, not hit vCLINT
        let access = mprv_load(0x0200_0000, MODE_S);
        access.check_fault(LOAD_PAGE_FAULT, 0x0200_0000, "Unmapped VA at CLINT address");

        // SUM = 0: S-mode access to U page faults with LoadPageFault / StorePageFault
        let access = mprv_load(good, MODE_S);
        access.check_fault(LOAD_PAGE_FAULT, good, "S-mode load from U page with SUM=0");
        let access = mprv_store(good, MODE_S, 0x99);
        access.check_fault(STORE_PAGE_FAULT, good, "S-mode store to U page with SUM=0");

        // SUM = 1: S-mode access to U page succeeds
        asm!("csrs mstatus, {sum}", sum = in(reg) SUM);
        let access = mprv_load(good, MODE_S);
        assert_eq!(
            access.mcause, 0,
            "S-mode load from U page with SUM=1 trapped"
        );
        assert_eq!(access.value, GOOD_VALUE);
        asm!("csrc mstatus, {sum}", sum = in(reg) SUM);

        // MXR = 0 vs MXR = 1 on X-only page
        let xonly_va = good + 0x4000_0000;
        let access = mprv_load(xonly_va, MODE_S);
        access.check_fault(
            LOAD_PAGE_FAULT,
            xonly_va,
            "Load from X-only page with MXR=0",
        );
        asm!("csrs mstatus, {mxr}", mxr = in(reg) MXR);
        let access = mprv_load(xonly_va, MODE_S);
        assert_eq!(access.mcause, 0, "Load from X-only page with MXR=1 trapped");
        assert_eq!(access.value, GOOD_VALUE);
        asm!("csrc mstatus, {mxr}", mxr = in(reg) MXR);

        asm!("csrw satp, zero", "sfence.vma");
    }

    // MPRV is now 0, regular accesses must not be affected anymore
    unsafe { ptr::write_volatile(good as *mut usize, GOOD_VALUE) };
    assert_eq!(
        unsafe { ptr::read_volatile(good as *const usize) },
        GOOD_VALUE
    );

    success();
}

/// The outcome of a load or store performed with MPRV = 1.
struct Access {
    /// The loaded value, for loads that did not trap.
    value: usize,
    /// The mcause of the trap taken during the access, or 0 if the access did not trap.
    mcause: usize,
    /// The mtval observed by the trap handler.
    mtval: usize,
    /// The mstatus observed by the trap handler.
    mstatus: usize,
}

impl Access {
    fn check_fault(&self, mcause: usize, addr: usize, msg: &str) {
        assert_eq!(self.mcause, mcause, "{}: wrong mcause", msg);
        assert_eq!(self.mtval, addr, "{}: wrong mtval", msg);
        assert_eq!(self.mstatus & MPRV, MPRV, "{}: MPRV not preserved", msg);
        assert_eq!(self.mstatus & MPP_MASK, MPP_MASK, "{}: MPP is not M", msg);
        assert_eq!(
            unsafe { ptr::read_volatile(&raw const MPRV_TEST_SCRATCH) },
            mcause,
            "{}: the trap handler could not access memory",
            msg
        );
        unsafe { ptr::write_volatile(&raw mut MPRV_TEST_SCRATCH, 0) };
    }
}

/// Load from `addr` with MPRV = 1 and the given MPP.
fn mprv_load(addr: usize, mpp: usize) -> Access {
    let value: usize;
    let mcause: usize;
    let mtval: usize;
    let mstatus: usize;
    // The whole sequence runs with MPRV = 1, it must not perform any other load or store.
    unsafe {
        asm!(
            "li t6, 0",
            "csrr t5, mstatus",
            "and t5, t5, {clear}",
            "or t5, t5, {set}",
            "csrw mstatus, t5",
            // The trap handler assumes a 4 bytes instruction
            ".option push",
            ".option norvc",
            "ld {value}, 0({addr})",
            ".option pop",
            "csrc mstatus, {mprv}",
            clear = in(reg) !MPP_MASK,
            set = in(reg) (mpp << MPP_OFFSET) | MPRV,
            mprv = in(reg) MPRV,
            addr = in(reg) addr,
            value = inout(reg) 0usize => value,
            out("t3") mtval,
            out("t4") mstatus,
            out("t5") _,
            out("t6") mcause,
        );
    }
    Access {
        value,
        mcause,
        mtval,
        mstatus,
    }
}

/// Store `value` to `addr` with MPRV = 1 and the given MPP.
fn mprv_store(addr: usize, mpp: usize, value: usize) -> Access {
    let mcause: usize;
    let mtval: usize;
    let mstatus: usize;
    // The whole sequence runs with MPRV = 1, it must not perform any other load or store.
    unsafe {
        asm!(
            "li t6, 0",
            "csrr t5, mstatus",
            "and t5, t5, {clear}",
            "or t5, t5, {set}",
            "csrw mstatus, t5",
            // The trap handler assumes a 4 bytes instruction
            ".option push",
            ".option norvc",
            "sd {value}, 0({addr})",
            ".option pop",
            "csrc mstatus, {mprv}",
            clear = in(reg) !MPP_MASK,
            set = in(reg) (mpp << MPP_OFFSET) | MPRV,
            mprv = in(reg) MPRV,
            addr = in(reg) addr,
            value = in(reg) value,
            out("t3") mtval,
            out("t4") mstatus,
            out("t5") _,
            out("t6") mcause,
        );
    }
    Access {
        value: 0,
        mcause,
        mtval,
        mstatus,
    }
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _mprv_test_trap_handler
_mprv_test_trap_handler:
    csrr t6, mcause
    csrr t3, mtval
    csrr t4, mstatus

    // The handler runs with MPRV = 1 and MPP = M: these accesses must behave as regular M-mode
    // accesses
    la t5, MPRV_TEST_SCRATCH
    sd t6, 0(t5)
    ld t5, 0(t5)
    bne t5, t6, _mprv_test_trap_handler_fail

    csrr t5, mepc  // Skip the faulting instruction
    addi t5, t5, 4
    csrw mepc, t5
    mret

_mprv_test_trap_handler_fail:
    // Report a mismatch through the scratch value checked by the test
    li t6, -1
    csrr t5, mepc
    addi t5, t5, 4
    csrw mepc, t5
    mret
"#,
);

unsafe extern "C" {
    fn _mprv_test_trap_handler();
}
