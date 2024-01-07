//! Bare metal RISC-V

use core::arch::{asm, global_asm};
use core::ptr;

use super::{Architecture, Mode, MCause};
use crate::virt::VirtContext;

/// Bare metal RISC-V runtime.
pub struct Metal {}

impl Architecture for Metal {
    fn init() {
        // Set trap handler
        let handler = _raw_trap_handler as usize;
        unsafe { write_mtvec(handler) };
        let mtvec = read_mtvec();
        assert_eq!(handler, mtvec, "Failed to set trap handler");
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

    fn read_mcause() -> MCause {
        let mcause: usize;
        unsafe {
            asm!(
                "csrr {x}, mcause",
                x = out(reg) mcause);
        }
        return MCause::new(mcause);
    }

    fn read_mepc() -> usize {
        let mepc: usize;
        unsafe {
            asm!(
                "csrr {x}, mepc",
                x = out(reg) mepc);
        }
        return mepc;
    }

    fn read_mtval() -> usize {
        let mtval: usize;
        unsafe {
            asm!(
                "csrr {x}, mtval",
                x = out(reg) mtval);
        }
        return mtval;
    }

    unsafe fn set_mpp(mode: Mode) {
        const MPP_MASK: usize = 0b11_usize << 11;
        let value = mode.to_bits() << 11;
        let mstatus = Self::read_mstatus();
        Self::write_mstatus((mstatus & !MPP_MASK) | value)
    }

    unsafe fn write_mepc(mepc: usize) {
        asm!(
            "csrw mepc, {x}",
            x = in(reg) mepc
        )
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

    unsafe fn get_raw_faulting_instr() -> usize {
        // First, try mtval and check if it contains an instruction
        let mtval = Self::read_mtval();
        if mtval != 0 {
            return mtval;
        }

        let epc = Self::read_mepc();
        let instr_ptr = epc as *const u32;

        // With compressed instruction extention ("C") instructions can be misaligned.
        // TODO: add support for 16 bits instructions
        let instr = ptr::read_unaligned(instr_ptr);
        instr as usize
    }

    unsafe fn enter_virt_firmware(ctx: &mut VirtContext) {
        asm!(
            // We need to save some registers manually, the compiler can't handle those
            "sd x3, (8*1)(sp)",
            "sd x4, (8*2)(sp)",
            "sd x8, (8*3)(sp)",
            "sd x9, (8*4)(sp)",
            // Jump into context switch code
            "jal x30, _enter_virt_firmware",
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
}

unsafe fn write_mtvec(value: usize) {
    asm!(
        "csrw mtvec, {x}",
        x = in(reg) value
    )
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

// ————————————————————————————— Context Switch ————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _enter_virt_firmware
_enter_virt_firmware:
    csrw mscratch, x31      // Save context in mscratch
    sd x30, (0)(sp)         // Store return address
    sd sp,(8*0)(x31)        // Store host stack
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
    ld sp,(8*0)(x31)      // Restore host stack
    ld x30,(sp)           // Load return address from stack
    jr x30                // Return
"#,
);

extern "C" {
    fn _raw_trap_handler();
}
