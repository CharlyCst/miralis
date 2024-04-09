//! Bare metal RISC-V

use core::arch::{asm, global_asm};
use core::ptr;

use super::{Architecture, MCause, Mode, TrapInfo};
use crate::arch::mstatus::{MPP_FILTER, MPP_OFFSET};
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

    unsafe fn write_pmpcfg(idx: usize, pmpcfg: usize) {
        match idx {
            0 => {
                asm!(
                    "csrw pmpcfg0, {x}",
                    x = in(reg) pmpcfg
                )
            }
            _ => todo!("pmpcfg{} not yet implemented", idx),
        }
    }

    unsafe fn write_pmpaddr(idx: usize, pmpaddr: usize) {
        match idx {
            0 => {
                asm!(
                    "csrw pmpaddr0, {x}",
                    x = in(reg) pmpaddr
                )
            }
            1 => {
                asm!(
                    "csrw pmpaddr1, {x}",
                    x = in(reg) pmpaddr
                )
            }
            _ => todo!("pmpaddr{} not yet implemented", idx),
        }
    }

    unsafe fn mret() -> ! {
        asm!("mret", options(noreturn))
    }

    unsafe fn ecall() {
        asm!("ecall")
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
    unsafe fn switch_from_firmware_to_payload(ctx: &mut VirtContext) {
        // First, restore S-mode registers
        asm!(
            "csrw stvec, {stvec}",
            "csrw scounteren, {scounteren}",
            "csrw senvcfg, {senvcfg}",
            stvec = in(reg) ctx.csr.stvec,
            scounteren = in(reg) ctx.csr.scounteren,
            senvcfg = in(reg) ctx.csr.senvcfg,
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
            "csrw satp, {satp}",
            stval = in(reg) ctx.csr.stval,
            satp = in(reg) ctx.csr.satp,
            options(nomem)
        );

        // Then configuring M-mode registers
        asm!(
            "csrw mstatus, {mstatus}",
            mstatus = in(reg) ctx.csr.mstatus,
            options(nomem)
        );
        // TODO: should we filter mstatus? What other registers?
        // - mip?
        // - mie?
    }

    /// Loads the S-mode CSR registers into the virtual context and install sensible values (mostly
    /// 0) for running the virtual firmware in U-mode.
    unsafe fn switch_from_payload_to_firmware(ctx: &mut VirtContext) {
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
            "csrrw {senvcfg}, senvcfg, x0",
            stvec = out(reg) stvec,
            scounteren = out(reg) scounteren,
            senvcfg = out(reg) senvcfg,
            options(nomem)
        );
        ctx.csr.stvec = stvec;
        ctx.csr.scounteren = scounteren;
        ctx.csr.senvcfg = senvcfg;

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
            "csrrw {satp}, satp, x0",
            stval = out(reg) stval,
            satp = out(reg) satp,
            options(nomem)
        );
        ctx.csr.stval = stval;
        ctx.csr.satp = satp;

        // Now save M-mode registers which are (partially) exposed as S-mode registers.
        // For mstatus we read the current value and clear the two MPP bits to jump into U-mode
        // (virtual firmware) during the next mret.
        let mstatus: usize;
        let mpp_u_mode: usize = MPP_FILTER << MPP_OFFSET;
        asm!(
            "csrrc {mstatus}, mstatus, {mpp_u_mode}",
            mstatus = out(reg) mstatus,
            mpp_u_mode = in(reg) mpp_u_mode,
            options(nomem)
        );
        ctx.csr.mstatus = mstatus;
        // TODO: handle S-mode registers which are subsets of M-mode registers, such as:
        // - sip
        // - sie
    }
}

unsafe fn write_mtvec(value: usize) {
    asm!(
        "csrw mtvec, {x}",
        x = in(reg) value
    )
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
