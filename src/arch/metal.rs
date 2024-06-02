//! Bare metal RISC-V
use core::arch::{asm, global_asm};
use core::{ptr, usize};

use super::{Architecture, MCause, Mode, TrapInfo};
use crate::arch::mstatus::{MPP_FILTER, MPP_OFFSET};
use crate::arch::pmp::pmpcfg;
use crate::arch::PmpGroup;
use crate::host::MirageContext;
use crate::virt::VirtContext;
use crate::{_stack_bottom, _stack_top, main};

/// Bare metal RISC-V runtime.
pub struct MetalArch {}

impl Architecture for MetalArch {
    fn init() {
        // Set trap handler
        let handler = _raw_trap_handler as usize;
        unsafe { write_mtvec(handler) };
        let mtvec = Self::read_mtvec();
        assert_eq!(handler, mtvec, "Failed to set trap handler");
    }

    fn read_misa() -> usize {
        let misa: usize;
        unsafe {
            asm!(
                "csrr {x}, misa",
                x = out(reg) misa);
        }
        return misa;
    }

    fn read_mstatus() -> usize {
        let mstatus: usize;
        unsafe {
            asm!(
                "csrr {x}, mstatus",
                x = out(reg) mstatus);
        }
        return mstatus;
    }

    unsafe fn set_mpp(mode: Mode) {
        const MPP_MASK: usize = 0b11_usize << 11;
        let value = mode.to_bits() << 11;
        let mstatus = Self::read_mstatus();
        Self::write_mstatus((mstatus & !MPP_MASK) | value)
    }

    unsafe fn write_mstatus(mstatus: usize) {
        asm!(
            "csrw mstatus, {x}",
            x = in(reg) mstatus
        )
    }

    unsafe fn write_pmp(pmp: &PmpGroup) {
        let pmpaddr = pmp.pmpaddr();
        let pmpcfg = pmp.pmpcfg();
        let nb_pmp = pmp.nb_pmp as usize;

        assert!(
            nb_pmp as usize <= pmpaddr.len() && nb_pmp as usize <= pmpcfg.len() * 8,
            "Invalid number of PMP registers"
        );

        for idx in 0..nb_pmp {
            write_pmpaddr(idx, pmpaddr[idx]);
        }
        for idx in 0..(nb_pmp / 8) {
            let cfg = pmpcfg[idx];
            write_pmpcfg(idx * 2, cfg);
        }
    }

    unsafe fn get_raw_faulting_instr(trap_info: &TrapInfo) -> usize {
        assert!(
            trap_info.mcause == MCause::IllegalInstr as usize,
            "Trying to read faulting instruction but trap is not an illegal instruction"
        );

        // First, try mtval and check if it contains an instruction
        if trap_info.mtval != 0 {
            return trap_info.mtval;
        }

        let instr_ptr = trap_info.mepc as *const u32;

        // With compressed instruction extention ("C") instructions can be misaligned.
        // TODO: add support for 16 bits instructions
        let instr = ptr::read_unaligned(instr_ptr);
        instr as usize
    }

    unsafe fn run_vcpu(ctx: &mut VirtContext) {
        asm!(
            // We need to save some registers manually, the compiler can't handle those
            "sd x3, (8*1)(sp)",
            "sd x4, (8*2)(sp)",
            "sd x8, (8*3)(sp)",
            "sd x9, (8*4)(sp)",
            // Jump into context switch code
            "jal x30, _run_vcpu",
            // Restore registers
            "ld x3, (8*1)(sp)",
            "ld x4, (8*2)(sp)",
            "ld x8, (8*3)(sp)",
            "ld x9, (8*4)(sp)",
            // Clobber all other registers, so that the compiler automatically
            // saves and restores the ones it needs
            inout("x31") ctx => _,
            out("x1") _,
            out("x5") _,
            out("x6") _,
            out("x7") _,
            out("x10") _,
            out("x11") _,
            out("x12") _,
            out("x13") _,
            out("x14") _,
            out("x15") _,
            out("x16") _,
            out("x17") _,
            out("x18") _,
            out("x19") _,
            out("x20") _,
            out("x21") _,
            out("x22") _,
            out("x23") _,
            out("x24") _,
            out("x25") _,
            out("x26") _,
            out("x27") _,
            out("x28") _,
            out("x29") _,
            out("x30") _,
        );
    }

    fn read_mtvec() -> usize {
        let mtvec: usize;
        unsafe {
            asm!(
                "csrr {x}, mtvec",
                x = out(reg) mtvec
            )
        }
        return mtvec;
    }

    /// Loads the S-mode CSR registers into the physical registers configures M-mode registers for
    /// payload execution.
    unsafe fn switch_from_firmware_to_payload(ctx: &mut VirtContext, mctx: &mut MirageContext) {
        // First, restore S-mode registers
        asm!(
            "csrw stvec, {stvec}",
            "csrw scounteren, {scounteren}",
            "csrw satp, {satp}",
            satp = in(reg) ctx.csr.satp,
            stvec = in(reg) ctx.csr.stvec,
            scounteren = in(reg) ctx.csr.scounteren,
            options(nomem)
        );
        asm!(
            "csrw sscratch, {sscratch}",
            "csrw sepc, {sepc}",
            "csrw scause, {scause}",
            sscratch = in(reg) ctx.csr.sscratch,
            sepc = in(reg) ctx.csr.sepc,
            scause = in(reg) ctx.csr.scause,
            options(nomem)
        );
        asm!(
            "csrw stval, {stval}",
            stval = in(reg) ctx.csr.stval,
            options(nomem)
        );

        // TODO: add support for senvcfg
        if false {
            asm!(
                "csrw senvcfg, {senvcfg}",
                senvcfg = in(reg) ctx.csr.senvcfg,
                options(nomem)
            );
        }

        // Then configuring M-mode registers
        asm!(
            "csrw mstatus, {mstatus}",
            "csrw mideleg, {mideleg}",
            "csrw medeleg, {medeleg}",
            mstatus = in(reg) ctx.csr.mstatus,
            mideleg = in(reg) ctx.csr.mideleg,
            medeleg = in(reg) ctx.csr.medeleg,
            options(nomem)
        );
        // TODO: should we filter mstatus? What other registers?
        // - mip?
        // - mie?

        // Load virtual PMP registers into Mirage's own registers
        mctx.pmp.load_with_offset(
            &ctx.csr.pmpaddr,
            &ctx.csr.pmpcfg,
            mctx.virt_pmp_offset as usize,
            ctx.nb_pmp,
        );
        // Deny all addresses by default
        let last_pmp_idx = mctx.pmp.nb_pmp as usize - 1;
        mctx.pmp.set(last_pmp_idx, usize::MAX, pmpcfg::NAPOT);
        // Commit the PMP to hardware
        Self::write_pmp(&mctx.pmp);
        Self::sfence_vma();
    }

    /// Loads the S-mode CSR registers into the virtual context and install sensible values (mostly
    /// 0) for running the virtual firmware in U-mode.
    unsafe fn switch_from_payload_to_firmware(ctx: &mut VirtContext, mctx: &mut MirageContext) {
        // Save the registers into the virtual context.
        // We save them 3 by 3 to give the compiler more freedom to choose registers and re-order
        // code (which is possible because of the `nomem` option).
        let stvec: usize;
        let scounteren: usize;
        let senvcfg: usize;
        let sscratch: usize;
        let sepc: usize;
        let scause: usize;
        let stval: usize;
        let satp: usize;

        asm!(
            "csrrw {stvec}, stvec, x0",
            "csrrw {scounteren}, scounteren, x0",
            "csrrw {satp}, satp, x0",
            stvec = out(reg) stvec,
            scounteren = out(reg) scounteren,
            satp = out(reg) satp,
            options(nomem)
        );
        ctx.csr.stvec = stvec;
        ctx.csr.scounteren = scounteren;
        ctx.csr.satp = satp;

        asm!(
            "csrrw {sscratch}, sscratch, x0",
            "csrrw {sepc}, sepc, x0",
            "csrrw {scause}, scause, x0",
            sscratch = out(reg) sscratch,
            sepc = out(reg) sepc,
            scause = out(reg) scause,
            options(nomem)
        );
        ctx.csr.sscratch = sscratch;
        ctx.csr.sepc = sepc;
        ctx.csr.scause = scause;

        asm!(
            "csrrw {stval}, stval, x0",
            stval = out(reg) stval,
            options(nomem)
        );
        ctx.csr.stval = stval;

        // TODO: add support for senvcfg
        if false {
            asm!(
                "csrrw {senvcfg}, senvcfg, x0",
                senvcfg = out(reg) senvcfg,
                options(nomem)
            );
            ctx.csr.senvcfg = senvcfg;
        }

        // Now save M-mode registers which are (partially) exposed as S-mode registers.
        // For mstatus we read the current value and clear the two MPP bits to jump into U-mode
        // (virtual firmware) during the next mret.
        let mstatus: usize;
        let mpp_u_mode: usize = MPP_FILTER << MPP_OFFSET;
        asm!(
            "csrrc {mstatus}, mstatus, {mpp_u_mode}",
            "csrw mideleg, x0", // Do not delegate any interrupts
            "csrw medeleg, x0", // Do not delegate any exceptions
            mstatus = out(reg) mstatus,
            mpp_u_mode = in(reg) mpp_u_mode,
            options(nomem)
        );
        ctx.csr.mstatus = mstatus;
        // TODO: handle S-mode registers which are subsets of M-mode registers, such as:
        // - sip
        // - sie

        // Remove Firmware PMP from the hardware
        mctx.pmp
            .clear_range(mctx.virt_pmp_offset as usize, ctx.nb_pmp);
        // Allow all addresses by default
        let last_pmp_idx = mctx.pmp.nb_pmp as usize - 1;
        mctx.pmp
            .set(last_pmp_idx, usize::MAX, pmpcfg::RWX | pmpcfg::NAPOT);
        // Commit the PMP to hardware
        Self::write_pmp(&mctx.pmp);
        Self::sfence_vma();
    }

    unsafe fn sfence_vma() {
        asm!("sfence.vma")
    }
}

unsafe fn write_mtvec(value: usize) {
    asm!(
        "csrw mtvec, {x}",
        x = in(reg) value
    )
}

unsafe fn write_pmpaddr(index: usize, pmpaddr: usize) {
    macro_rules! asm_write_pmpaddr {
        ($idx:literal, $addr:expr) => {
            asm!(
                concat!("csrw pmpaddr", $idx, ", {addr}"),
                addr = in(reg) pmpaddr,
                options(nomem)
            )
        };
    }

    match index {
        0 => asm_write_pmpaddr!(0, pmpaddr),
        1 => asm_write_pmpaddr!(1, pmpaddr),
        2 => asm_write_pmpaddr!(2, pmpaddr),
        3 => asm_write_pmpaddr!(3, pmpaddr),
        4 => asm_write_pmpaddr!(4, pmpaddr),
        5 => asm_write_pmpaddr!(5, pmpaddr),
        6 => asm_write_pmpaddr!(6, pmpaddr),
        7 => asm_write_pmpaddr!(7, pmpaddr),
        8 => asm_write_pmpaddr!(8, pmpaddr),
        9 => asm_write_pmpaddr!(9, pmpaddr),
        10 => asm_write_pmpaddr!(10, pmpaddr),
        11 => asm_write_pmpaddr!(11, pmpaddr),
        12 => asm_write_pmpaddr!(12, pmpaddr),
        13 => asm_write_pmpaddr!(13, pmpaddr),
        14 => asm_write_pmpaddr!(14, pmpaddr),
        15 => asm_write_pmpaddr!(15, pmpaddr),
        16 => asm_write_pmpaddr!(16, pmpaddr),
        17 => asm_write_pmpaddr!(17, pmpaddr),
        18 => asm_write_pmpaddr!(18, pmpaddr),
        19 => asm_write_pmpaddr!(19, pmpaddr),
        20 => asm_write_pmpaddr!(20, pmpaddr),
        21 => asm_write_pmpaddr!(21, pmpaddr),
        22 => asm_write_pmpaddr!(22, pmpaddr),
        23 => asm_write_pmpaddr!(23, pmpaddr),
        24 => asm_write_pmpaddr!(24, pmpaddr),
        25 => asm_write_pmpaddr!(25, pmpaddr),
        26 => asm_write_pmpaddr!(26, pmpaddr),
        27 => asm_write_pmpaddr!(27, pmpaddr),
        28 => asm_write_pmpaddr!(28, pmpaddr),
        29 => asm_write_pmpaddr!(29, pmpaddr),
        30 => asm_write_pmpaddr!(30, pmpaddr),
        31 => asm_write_pmpaddr!(31, pmpaddr),
        32 => asm_write_pmpaddr!(32, pmpaddr),
        33 => asm_write_pmpaddr!(33, pmpaddr),
        34 => asm_write_pmpaddr!(34, pmpaddr),
        35 => asm_write_pmpaddr!(35, pmpaddr),
        36 => asm_write_pmpaddr!(36, pmpaddr),
        37 => asm_write_pmpaddr!(37, pmpaddr),
        38 => asm_write_pmpaddr!(38, pmpaddr),
        39 => asm_write_pmpaddr!(39, pmpaddr),
        40 => asm_write_pmpaddr!(40, pmpaddr),
        41 => asm_write_pmpaddr!(41, pmpaddr),
        42 => asm_write_pmpaddr!(42, pmpaddr),
        43 => asm_write_pmpaddr!(43, pmpaddr),
        44 => asm_write_pmpaddr!(44, pmpaddr),
        45 => asm_write_pmpaddr!(45, pmpaddr),
        46 => asm_write_pmpaddr!(46, pmpaddr),
        47 => asm_write_pmpaddr!(47, pmpaddr),
        48 => asm_write_pmpaddr!(48, pmpaddr),
        49 => asm_write_pmpaddr!(49, pmpaddr),
        50 => asm_write_pmpaddr!(50, pmpaddr),
        51 => asm_write_pmpaddr!(51, pmpaddr),
        52 => asm_write_pmpaddr!(52, pmpaddr),
        53 => asm_write_pmpaddr!(53, pmpaddr),
        54 => asm_write_pmpaddr!(54, pmpaddr),
        55 => asm_write_pmpaddr!(55, pmpaddr),
        56 => asm_write_pmpaddr!(56, pmpaddr),
        57 => asm_write_pmpaddr!(57, pmpaddr),
        58 => asm_write_pmpaddr!(58, pmpaddr),
        59 => asm_write_pmpaddr!(59, pmpaddr),
        60 => asm_write_pmpaddr!(60, pmpaddr),
        61 => asm_write_pmpaddr!(61, pmpaddr),
        62 => asm_write_pmpaddr!(62, pmpaddr),
        63 => asm_write_pmpaddr!(63, pmpaddr),
        _ => panic!("Invalid pmpaddr register"),
    }
}

unsafe fn write_pmpcfg(index: usize, pmpcfg: usize) {
    macro_rules! asm_write_pmpcfg {
        ($idx:literal, $cfg:expr) => {
            asm!(
                concat!("csrw pmpcfg", $idx, ", {cfg}"),
                cfg = in(reg) $cfg,
                options(nomem)
            )
        };
    }

    match index {
        0 => asm_write_pmpcfg!(0, pmpcfg),
        2 => asm_write_pmpcfg!(2, pmpcfg),
        4 => asm_write_pmpcfg!(4, pmpcfg),
        6 => asm_write_pmpcfg!(6, pmpcfg),
        8 => asm_write_pmpcfg!(8, pmpcfg),
        10 => asm_write_pmpcfg!(10, pmpcfg),
        12 => asm_write_pmpcfg!(12, pmpcfg),
        14 => asm_write_pmpcfg!(14, pmpcfg),
        _ => panic!("Invalid pmpcfg register"),
    }
}

// —————————————————————————————— Entry Point ——————————————————————————————— //

global_asm!(
r#"
.text
.global _start
_start:
    // Start by filling the stack with a known memory pattern
    ld t0, __stack_bottom
    ld t1, __stack_top
    li t2, 0x0BADBED0
loop:
    bgeu t0, t1, done // Exit when reaching the end address
    sw t2, 0(t0)      // Write the pattern
    addi t0, t0, 4    // increment the cursor
    j loop
done:

    // Then load the stack pointer and jump into main
    ld sp, __stack_top
    j {main}

// Store the address of the stack in memory
// That way it can be loaded as an absolute value
__stack_top:
    .dword {stack_top}
__stack_bottom:
    .dword {stack_bottom}
"#,
    main = sym main,
    stack_top = sym _stack_top,
    stack_bottom = sym _stack_bottom,
);

// ————————————————————————————— Context Switch ————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _run_vcpu
_run_vcpu:
    csrw mscratch, x31        // Save context in mscratch
    sd x30, (0)(sp)           // Store return address
    sd sp,(8*0)(x31)          // Store host stack
    ld x1,(8+8*32)(x31)       // Read payload PC
    csrw mepc,x1              // Restore payload PC in mepc

    ld x1,(8+8*1)(x31)        // Load guest general purpose registers
    ld x2,(8+8*2)(x31)
    ld x3,(8+8*3)(x31)
    ld x4,(8+8*4)(x31)
    ld x5,(8+8*5)(x31)
    ld x6,(8+8*6)(x31)
    ld x7,(8+8*7)(x31)
    ld x8,(8+8*8)(x31)
    ld x9,(8+8*9)(x31)
    ld x10,(8+8*10)(x31)
    ld x11,(8+8*11)(x31)
    ld x12,(8+8*12)(x31)
    ld x13,(8+8*13)(x31)
    ld x14,(8+8*14)(x31)
    ld x15,(8+8*15)(x31)
    ld x16,(8+8*16)(x31)
    ld x17,(8+8*17)(x31)
    ld x18,(8+8*18)(x31)
    ld x19,(8+8*19)(x31)
    ld x20,(8+8*20)(x31)
    ld x21,(8+8*21)(x31)
    ld x22,(8+8*22)(x31)
    ld x23,(8+8*23)(x31)
    ld x24,(8+8*24)(x31)
    ld x25,(8+8*25)(x31)
    ld x26,(8+8*26)(x31)
    ld x27,(8+8*27)(x31)
    ld x28,(8+8*28)(x31)
    ld x29,(8+8*29)(x31)
    ld x30,(8+8*30)(x31)
    ld x31,(8+8*31)(x31)
    mret                      // Jump into firmware
"#,
);

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _raw_trap_handler
_raw_trap_handler:
    csrrw x31, mscratch, x31 // Restore context by swapping x31 and mscratch
    sd x0,(8+8*0)(x31)       // Save all general purpose registers
    sd x1,(8+8*1)(x31)
    sd x2,(8+8*2)(x31)
    sd x3,(8+8*3)(x31)
    sd x4,(8+8*4)(x31)
    sd x5,(8+8*5)(x31)
    sd x6,(8+8*6)(x31)
    sd x7,(8+8*7)(x31)
    sd x8,(8+8*8)(x31)
    sd x9,(8+8*9)(x31)
    sd x10,(8+8*10)(x31)
    sd x11,(8+8*11)(x31)
    sd x12,(8+8*12)(x31)
    sd x13,(8+8*13)(x31)
    sd x14,(8+8*14)(x31)
    sd x15,(8+8*15)(x31)
    sd x16,(8+8*16)(x31)
    sd x17,(8+8*17)(x31)
    sd x18,(8+8*18)(x31)
    sd x19,(8+8*19)(x31)
    sd x20,(8+8*20)(x31)
    sd x21,(8+8*21)(x31)
    sd x22,(8+8*22)(x31)
    sd x23,(8+8*23)(x31)
    sd x24,(8+8*24)(x31)
    sd x25,(8+8*25)(x31)
    sd x26,(8+8*26)(x31)
    sd x27,(8+8*27)(x31)
    sd x28,(8+8*28)(x31)
    sd x29,(8+8*29)(x31)
    sd x30,(8+8*30)(x31)
    csrr x30, mscratch    // Restore x31 into x30 from mscratch
    sd x30,(8+8*31)(x31)  // Save x31 (whose value is stored in x30)

    // TODO: restore host misa

    csrr x30, mepc              // Read payload PC
    sd x30, (8+8*32)(x31)       // Save the PC
    sd x30, (8+8*32+8+8*0)(x31) // Save mepc
    csrr x30, mstatus           // Fill the TrapInfo :  Read mstatus
    sd x30, (8+8*32+8+8*1)(x31) // Save mstatus
    csrr x30, mcause            // Fill the TrapInfo :  Read mcause
    sd x30, (8+8*32+8+8*2)(x31) // Save mcause
    csrr x30, mip               // Fill the TrapInfo : Read mip
    sd x30, (8+8*32+8+8*3)(x31) // Save mip
    csrr x30, mtval             // Fill the TrapInfo : Read mtval
    sd x30, (8+8*32+8+8*4)(x31) // Save mtval

    ld sp,(8*0)(x31)      // Restore host stack
    ld x30,(sp)           // Load return address from stack
    jr x30                // Return
"#,
);

extern "C" {
    fn _raw_trap_handler();
}
