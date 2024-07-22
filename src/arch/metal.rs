//! Bare metal RISC-V
use core::arch::{asm, global_asm};
use core::marker::PhantomData;
use core::{ptr, usize};

use super::{Architecture, Csr, MCause, Mode, RegistersCapability, TrapInfo};
use crate::arch::{mstatus, HardwareCapability, PmpGroup};
use crate::config::PLATFORM_STACK_SIZE;
use crate::virt::VirtContext;
use crate::{_bss_start, _bss_stop, _stack_start, main};

/// Bare metal RISC-V runtime.
pub struct MetalArch {}

impl MetalArch {
    fn install_handler(handler: usize) {
        // Set trap handler
        unsafe { Self::write_csr(Csr::Mtvec, handler) };
        let mtvec: usize = Self::read_csr(Csr::Mtvec);
        assert_eq!(handler, mtvec, "Failed to set trap handler");
    }
}

impl Architecture for MetalArch {
    fn init() {
        // Install trap handler
        Self::install_handler(_raw_trap_handler as usize);
    }

    #[inline]
    fn wfi() {
        unsafe { asm!("wfi") };
    }

    unsafe fn write_csr(csr: Csr, value: usize) -> usize {
        let mut prev_value: usize = 0;

        macro_rules! asm_write_csr {
            ($reg:literal) => {
                asm!(
                    concat!("csrrw {prev}, ", $reg, ", {val}"),
                    val = in(reg) value,
                    prev = out(reg) prev_value,
                    options(nomem)
                )
            };
        }

        match csr {
            Csr::Mhartid => asm_write_csr!("mhartid"),
            Csr::Mstatus => asm_write_csr!("mstatus"),
            Csr::Misa => asm_write_csr!("misa"),
            Csr::Mie => asm_write_csr!("mie"),
            Csr::Mtvec => asm_write_csr!("mtvec"),
            Csr::Mscratch => asm_write_csr!("mscratch"),
            Csr::Mip => asm_write_csr!("mip"),
            Csr::Mvendorid => asm_write_csr!("mvendorid"),
            Csr::Marchid => asm_write_csr!("marchid"),
            Csr::Mimpid => asm_write_csr!("mimpid"),
            Csr::Pmpcfg(_) => todo!(),
            Csr::Pmpaddr(index) => write_pmpaddr(index, value),
            Csr::Mcycle => asm_write_csr!("mcycle"),
            Csr::Minstret => asm_write_csr!("minstret"),
            Csr::Mhpmcounter(_) => todo!(),
            Csr::Mcountinhibit => asm_write_csr!("mcountinhibit"),
            Csr::Mhpmevent(_) => todo!(),
            Csr::Mcounteren => asm_write_csr!("mcounteren"),
            Csr::Menvcfg => asm_write_csr!("menvcfg"),
            Csr::Mseccfg => asm_write_csr!("mseccfg"),
            Csr::Mconfigptr => asm_write_csr!("mconfigptr"),
            Csr::Medeleg => asm_write_csr!("medeleg"),
            Csr::Mideleg => asm_write_csr!("mideleg"),
            Csr::Mtinst => asm_write_csr!("mtinst"),
            Csr::Mtval2 => asm_write_csr!("mtval2"),
            Csr::Tselect => asm_write_csr!("tselect"),
            Csr::Tdata1 => asm_write_csr!("tdata1"),
            Csr::Tdata2 => asm_write_csr!("tdata2"),
            Csr::Tdata3 => asm_write_csr!("tdata3"),
            Csr::Mcontext => asm_write_csr!("mcontext"),
            Csr::Dcsr => asm_write_csr!("dcsr"),
            Csr::Dpc => asm_write_csr!("dpc"),
            Csr::Dscratch0 => asm_write_csr!("dscratch0"),
            Csr::Dscratch1 => asm_write_csr!("dscratch1"),
            Csr::Mepc => asm_write_csr!("mepc"),
            Csr::Mcause => asm_write_csr!("mcause"),
            Csr::Mtval => asm_write_csr!("mtval"),
            Csr::Sstatus => asm_write_csr!("sstatus"),
            Csr::Sie => asm_write_csr!("sie"),
            Csr::Stvec => asm_write_csr!("stvec"),
            Csr::Scounteren => asm_write_csr!("scounteren"),
            Csr::Senvcfg => asm_write_csr!("senvcfg"),
            Csr::Sscratch => asm_write_csr!("sscratch"),
            Csr::Sepc => asm_write_csr!("sepc"),
            Csr::Scause => asm_write_csr!("scause"),
            Csr::Stval => asm_write_csr!("stval"),
            Csr::Sip => asm_write_csr!("sip"),
            Csr::Satp => asm_write_csr!("satp"),
            Csr::Scontext => asm_write_csr!("scontext"),
            Csr::Unknown => (),
        };

        prev_value
    }

    fn read_csr(csr: Csr) -> usize {
        let value: usize;

        macro_rules! asm_read_csr {
            ($reg:literal) => {
                unsafe {
                    asm!(
                        concat!("csrr {x}, ", $reg),
                        x = out(reg) value,
                        options(nomem)
                    )
                }
            };
        }

        match csr {
            Csr::Mhartid => asm_read_csr!("mhartid"),
            Csr::Mstatus => asm_read_csr!("mstatus"),
            Csr::Misa => asm_read_csr!("misa"),
            Csr::Mie => asm_read_csr!("mie"),
            Csr::Mtvec => asm_read_csr!("mtvec"),
            Csr::Mscratch => asm_read_csr!("mscratch"),
            Csr::Mip => asm_read_csr!("mip"),
            Csr::Mvendorid => asm_read_csr!("mvendorid"),
            Csr::Marchid => asm_read_csr!("marchid"),
            Csr::Mimpid => asm_read_csr!("mimpid"),
            Csr::Pmpcfg(_) => todo!(),
            Csr::Pmpaddr(_) => todo!(),
            Csr::Mcycle => asm_read_csr!("mcycle"),
            Csr::Minstret => asm_read_csr!("minstret"),
            Csr::Mhpmcounter(_) => todo!(),
            Csr::Mcountinhibit => asm_read_csr!("mcountinhibit"),
            Csr::Mhpmevent(_) => todo!(),
            Csr::Mcounteren => asm_read_csr!("mcounteren"),
            Csr::Menvcfg => asm_read_csr!("menvcfg"),
            Csr::Mseccfg => asm_read_csr!("mseccfg"),
            Csr::Mconfigptr => asm_read_csr!("mconfigptr"),
            Csr::Medeleg => asm_read_csr!("medeleg"),
            Csr::Mideleg => asm_read_csr!("mideleg"),
            Csr::Mtinst => asm_read_csr!("mtinst"),
            Csr::Mtval2 => asm_read_csr!("mtval2"),
            Csr::Tselect => asm_read_csr!("tselect"),
            Csr::Tdata1 => asm_read_csr!("tdata1"),
            Csr::Tdata2 => asm_read_csr!("tdata2"),
            Csr::Tdata3 => asm_read_csr!("tdata3"),
            Csr::Mcontext => asm_read_csr!("mcontext"),
            Csr::Dcsr => asm_read_csr!("dcsr"),
            Csr::Dpc => asm_read_csr!("dpc"),
            Csr::Dscratch0 => asm_read_csr!("dscratch0"),
            Csr::Dscratch1 => asm_read_csr!("dscratch1"),
            Csr::Mepc => asm_read_csr!("mepc"),
            Csr::Mcause => asm_read_csr!("mcause"),
            Csr::Mtval => asm_read_csr!("mtval"),
            Csr::Sstatus => asm_read_csr!("sstatus"),
            Csr::Sie => asm_read_csr!("sie"),
            Csr::Stvec => asm_read_csr!("stvec"),
            Csr::Scounteren => asm_read_csr!("scounteren"),
            Csr::Senvcfg => asm_read_csr!("senvcfg"),
            Csr::Sscratch => asm_read_csr!("sscratch"),
            Csr::Sepc => asm_read_csr!("sepc"),
            Csr::Scause => asm_read_csr!("scause"),
            Csr::Stval => asm_read_csr!("stval"),
            Csr::Sip => asm_read_csr!("sip"),
            Csr::Satp => asm_read_csr!("satp"),
            Csr::Scontext => asm_read_csr!("scontext"),
            Csr::Unknown => value = 0,
        };

        value
    }

    unsafe fn detect_hardware() -> HardwareCapability {
        macro_rules! register_present {
             ($reg:expr) => {{
                 // Install "tracer" handler, it allows miralis to know if it executed an illegal instruction
                 // and thus detects which registers aren't available
                 Self::install_handler(_tracing_trap_handler as usize);

                 // Perform detection
                 let mut _dummy_variable: usize = 0;
                 let mut tracer_var: usize;
                 unsafe {
                     asm!(
                        "csrw mscratch, zero",
                        concat!("csrr {0}, ", $reg),
                        "csrr {1}, mscratch",
                        out(reg) _dummy_variable,
                        out(reg) tracer_var,
                        );
                    }

                 // Restore normal handler
                 Self::install_handler(_raw_trap_handler as usize);

                 // Present if value is 0
                 tracer_var == 0
             }};
        }

        // Test menvcfg & senvcfg
        // Hint: to simulate a missing register, one can add "ecall" after the first line in asm! of the macro
        let is_menvcfg_present: bool = register_present!("menvcfg");
        let is_senvcfg_present: bool = register_present!("senvcfg");
        log::debug!(
            "Detecting available envcfg registers [menvcfg : {} | senvcfg : {} ]",
            is_menvcfg_present,
            is_senvcfg_present,
        );

        // Detect available PMP registers:
        // - On RV64 platforms only even-numbered pmpcfg registers are present
        // - The spec mandates that there is either 0, 16 or 64 PMP registers implemented
        // Thus:
        // - If pmpcfg14 is implemented there is 64 implemented PMP registers
        // - if pmpcfg2 is implemented there is 16 implemented PMP registers
        // - Otherwise there is 0
        let is_pmpcfg14_present: bool = register_present!("pmpcfg14");
        let is_pmpcfg2_present: bool = register_present!("pmpcfg2");
        let nb_implemented_pmp = if is_pmpcfg14_present {
            64
        } else if is_pmpcfg2_present {
            16
        } else {
            0
        };
        let nb_pmp = find_nb_of_non_zero_pmp(nb_implemented_pmp);

        log::debug!("Number of PMP: {}", nb_pmp);

        // Save current CSRs
        let mstatus = Self::read_csr(Csr::Mstatus);
        let mtvec = Self::read_csr(Csr::Mtvec);

        // Read hart ID
        let hart = Self::read_csr(Csr::Mhartid);

        // Detect available interrupt IDs
        let available_int: usize;
        asm!(
            "csrc mstatus, {clear_mie}", // Disable interrupts by clearing MIE in mstatus
            "csrw mie, {all_int}",       // Set all bits in the mie register
            "csrr {available_int}, mie", // Read back wich bits are set to 1
            "csrw mie, x0",              // Clear all bits in mie
            clear_mie = in(reg) mstatus::MIE_FILTER,
            all_int = in(reg) usize::MAX,
            available_int = out(reg) available_int,
            options(nomem)
        );

        // Restore CSRs
        Self::write_csr(Csr::Mstatus, mstatus);
        Self::write_csr(Csr::Mtvec, mtvec);

        // Return hardware configuration
        HardwareCapability {
            interrupts: available_int,
            hart,
            _marker: PhantomData,
            available_reg: RegistersCapability {
                menvcfg: is_menvcfg_present,
                senvcfg: is_senvcfg_present,
                nb_pmp,
            },
        }
    }

    unsafe fn set_mpp(mode: Mode) {
        let value = mode.to_bits() << mstatus::MPP_OFFSET;
        let mstatus = Self::read_csr(Csr::Mstatus);
        Self::write_csr(Csr::Mstatus, (mstatus & !mstatus::MPP_FILTER) | value);
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
        if trap_info.mcause == MCause::IllegalInstr as usize {
            // First, try mtval and check if it contains an instruction
            if trap_info.mtval != 0 {
                return trap_info.mtval;
            }
        }

        let instr_ptr = trap_info.mepc as *const u32;

        // With compressed instruction extention ("C") instructions can be misaligned.
        // TODO: add support for 16 bits instructions
        let instr = ptr::read_unaligned(instr_ptr);
        instr as usize
    }

    unsafe fn run_vcpu(ctx: &mut VirtContext) {
        // When M-mode, patch mie register in order to trap on i only when mstatus.MIE
        // is set and mideleg[i] is not set.
        if ctx.mode == Mode::M {
            let mie = (mstatus::MIE_FILTER & ctx.csr.mstatus) >> mstatus::MIE_OFFSET;
            let mie_patched = if mie == 1 {
                ctx.csr.mie & !ctx.csr.mideleg
            } else {
                0
            };
            unsafe {
                Self::write_csr(Csr::Mie, mie_patched);
            }
        }

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
    unsafe fn sfence_vma() {
        asm!("sfence.vma")
    }
}

/// Finds the number of non-zero PMP registers, i.e. the effective number of PMP registers
/// available on the current core.
///
/// SAFETY: This function assumes that at least `nb_implemented` PMP registers are _implemented_
/// (but some can be hard-wired at 0). If that is not the case this function might trap with an
/// illegal instruction exception.
unsafe fn find_nb_of_non_zero_pmp(nb_implemented: usize) -> usize {
    // According to the spec either 0, 16 or 64 entries are implemented
    assert!(nb_implemented == 0 || nb_implemented == 16 || nb_implemented == 64);

    // This macro tests if a PMP address register can be written with a non-zero value, and return
    // the index of the PMP. If we test all PMP sequentially in increasing order the returned value
    // is number of implemented pmp.
    //
    // E.g. if PMP 0 and 1 are implemented but not PMP 2, then
    // ```
    // test_pmp!(0);
    // test_pmp!(1);
    // test_pmp!(2);
    // ```
    // Does return the number of implemented PMPs (2).
    //
    // Unfortunately we have to rely on macros because each pmp entry needs a different assembly
    // instruction to be written. Because of this we can't check all entries in a for loop, but we
    // need to manually write (or with a macro) 64 tests.
    macro_rules! test_pmp {
        ($idx:literal) => {{
            let read_addr: usize;
            asm!(
                // We save try to write a non-zero value in pmpaddr and read it back
                concat!("csrrw {tmp}, pmpaddr", $idx, ", {addr}"),
                concat!("csrrw {addr}, pmpaddr", $idx, ", {tmp}"),
                tmp = out(reg) _,
                addr = inout(reg) 1_usize => read_addr,
                options(nomem)
            );
            if read_addr == 0 {
                return $idx;
            }
        }};
    }

    if nb_implemented == 0 {
        return 0;
    }

    test_pmp!(0);
    test_pmp!(1);
    test_pmp!(2);
    test_pmp!(3);
    test_pmp!(4);
    test_pmp!(5);
    test_pmp!(6);
    test_pmp!(7);
    test_pmp!(8);
    test_pmp!(9);
    test_pmp!(10);
    test_pmp!(11);
    test_pmp!(12);
    test_pmp!(13);
    test_pmp!(14);
    test_pmp!(15);

    if nb_implemented == 16 {
        return 16;
    }

    test_pmp!(16);
    test_pmp!(17);
    test_pmp!(18);
    test_pmp!(19);
    test_pmp!(20);
    test_pmp!(21);
    test_pmp!(22);
    test_pmp!(23);
    test_pmp!(24);
    test_pmp!(25);
    test_pmp!(26);
    test_pmp!(27);
    test_pmp!(28);
    test_pmp!(29);
    test_pmp!(30);
    test_pmp!(31);
    test_pmp!(32);
    test_pmp!(33);
    test_pmp!(34);
    test_pmp!(35);
    test_pmp!(36);
    test_pmp!(37);
    test_pmp!(38);
    test_pmp!(39);
    test_pmp!(40);
    test_pmp!(41);
    test_pmp!(42);
    test_pmp!(43);
    test_pmp!(44);
    test_pmp!(45);
    test_pmp!(46);
    test_pmp!(47);
    test_pmp!(48);
    test_pmp!(49);
    test_pmp!(50);
    test_pmp!(51);
    test_pmp!(52);
    test_pmp!(53);
    test_pmp!(54);
    test_pmp!(55);
    test_pmp!(56);
    test_pmp!(57);
    test_pmp!(58);
    test_pmp!(59);
    test_pmp!(60);
    test_pmp!(61);
    test_pmp!(62);
    test_pmp!(63);

    return 64;
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
.align 4
.text
.global _start
_start:
    // We start by setting up the stack:
    // First we find where the stack is for that hart
    ld t0, __stack_start
    ld t1, {stack_size}  // Per-hart stack size
    csrr t2, mhartid     // Our current hart ID
    mul t3, t2, t1       // How me space before our own stack 
    add t0, t0, t3       // The actual start of our stack
    add t1, t0, t1       // And the end of our stack

    ld t4, __bss_start
    ld t5, __bss_stop

    // Then we fill the stack with a known memory pattern
    li t2, 0x0BADBED0

loop:
    bgeu t0, t1, zero_loop // Exit when reaching the end address
    sw t2, 0(t0)      // Write the pattern
    addi t0, t0, 4    // increment the cursor
    j loop

zero_loop:
    bgeu t4, t5, verify
    sb x0, 0(t4)
    addi t4, t4, 1
    j zero_loop

verify:
    ld t4, __bss_start
    ld t5, __bss_stop
    j verify_loop

verify_loop:
    bgeu t4, t5, done
    lb t6, 0(t4)  
    beqz t6, continue_verification

continue_verification:
    addi t4, t4, 1
    j verify_loop

halt_execution:
    j halt_execution

done:
    // And finally we load the stack pointer into sp and jump into main
    mv sp, t1
    la t5, 0x43

    lui t6, %hi(0x10000000)
    addi t6, t6, %lo(0x10000000)
    sb t5, 0(t6)
    j {main}

// Store the address of the stack in memory
// That way it can be loaded as an absolute value
.align 8
__stack_start:
    .dword {stack_start}
__bss_start:
    .dword {bss_start}
__bss_stop:
    .dword {bss_stop}
"#,
    main = sym main,
    stack_start = sym _stack_start,
    stack_size = sym STACK_SIZE,
    bss_start = sym _bss_start,
    bss_stop = sym _bss_stop,
);

// NOTE: We need to use a static here because constant in `asm!` blocks are not yet supported.
// The workaround is to create a static (so a variable in memory) holding the value and using the
// symbol (so the address) in assembly to load it into a register.
//
// This can be removed once `asm_const` gets stabilized. See:
// https://github.com/rust-lang/rust/issues/93332
static STACK_SIZE: usize = PLATFORM_STACK_SIZE;

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
    ld x1,(8+8*32)(x31)       // Read guest PC
    csrw mepc,x1              // Restore guest PC in mepc

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
    mret                      // Jump into firmware or payload
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

    csrr x30, mepc              // Read guest PC
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

// —————————————————————————————— Tracing trap Handler —————————————————————————————— //

global_asm!(
    r"
.text
.align 4
.global _tracing_trap_handler
_tracing_trap_handler:
    // Skip illegal instruction (pc += 4)
    csrrw x5, mepc, x5
    addi x5, x5, 4
    csrrw x5, mepc, x5
    // Set mscratch to 1
    csrrw x5, mscratch, x5
    addi x5, x0, 1
    csrrw x5, mscratch, x5
    // Return back to miralis
    mret
#"
);

extern "C" {
    fn _raw_trap_handler();
    fn _tracing_trap_handler();
}
