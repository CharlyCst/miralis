//! Bare metal RISC-V
use core::arch::{asm, global_asm};
use core::marker::PhantomData;
use core::usize;

use super::{menvcfg, Arch, Architecture, Csr, ExtensionsCapability, Mode, RegistersCapability};
use crate::arch::hstatus::GVA_FILTER;
use crate::arch::Csr::{Mtinst, Mtval2};
use crate::arch::{mie, misa, mstatus, parse_mpp_return_mode, set_mpp, HardwareCapability, Width};
use crate::decoder::{LoadInstr, StoreInstr};
use crate::platform::{Plat, Platform};
use crate::virt::VirtContext;
use crate::{utils, RegisterContextGetter, RegisterContextSetter};

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

// ————————————————————————————————— Utils —————————————————————————————————— //

/// Loads or store a value using MPRV (memory privileged).
///
/// This allows to load/store with the previous privileges access rights (in the sense of previous
/// privileged mode in `mstatus`). For instance if S-mode was running last, this will perform a
/// load/store using the installed S-mode page tables.
///
/// This macro sets up a special trap handler, [_mprv_trap_handler], to catch traps if the
/// load/store fails. The regular trap handler can't be used because of the MPRV bit which would
/// cause read/writes with another privilege mode's access rights.
macro_rules! asm_mprv_mem_op {
    ($instr:literal, $addr:expr, $value:ident) => {
        asm!(
            "csrr {1}, mtvec",      // Save regular mtvec
            "csrw mtvec, {mtvec}",  // Set up mtvec to an address-space independent

            // Set MPRV bit to 1
            "li {0}, 1",
            "slli {0}, {0}, 17",
            "csrs mstatus, {0}",

            // The 'norvc' guarantees that instructions are 4 bytes wide.
            // This prevent the compiler from using compressed load/stores.
            ".option push",
            ".option norvc",
            concat!($instr, " {rd}, 0({addr})"),
            ".option pop",

            "csrc mstatus, {0}",    // Restore values
            "csrw mtvec, {1}",
            out(reg) _,
            out(reg) _,
            mtvec = in(reg) (_mprv_trap_handler as usize),
            addr = in(reg) $addr,
            rd = inout(reg) $value,
        )
    };
}

// —————————————————————————— Arch Implementation ——————————————————————————— //

impl Architecture for MetalArch {
    fn init() {
        // Install trap handler
        Self::install_handler(_raw_trap_handler as usize);
        // Initialize `medeleg` to ensure all exceptions trap to Miralis
        unsafe { Arch::write_csr(Csr::Medeleg, 0) };
        // Initialize `mideleg` with read-only ones
        unsafe { Arch::write_csr(Csr::Mideleg, mie::MIDELEG_READ_ONLY_ONE) };
        // Ensure that there are no PT set, so that firmware in U-mode
        // Wouldn't try to read physical address as virtual (with jump, for example)
        unsafe { Arch::write_csr(Csr::Satp, 0) };
        // Ensure that virtualized interrupts are enabled
        unsafe { Arch::set_csr_bits(Csr::Mie, mie::MSIE_FILTER | mie::MTIE_FILTER) };
    }

    #[inline]
    fn wfi() {
        unsafe { asm!("wfi") };
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
            Csr::Pmpaddr(index) => Arch::write_pmpaddr(index, value),
            Csr::Mcycle => asm_write_csr!("mcycle"),
            Csr::Minstret => asm_write_csr!("minstret"),
            Csr::Cycle => todo!(),
            Csr::Time => todo!(),
            Csr::Instret => todo!(),
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
            Csr::Stimecmp => asm_write_csr!("stimecmp"),
            Csr::Hstatus => asm_write_csr!("hstatus"),
            Csr::Hedeleg => asm_write_csr!("hedeleg"),
            Csr::Hideleg => asm_write_csr!("hideleg"),
            Csr::Hvip => asm_write_csr!("hvip"),
            Csr::Hip => asm_write_csr!("hip"),
            Csr::Hie => asm_write_csr!("hie"),
            Csr::Hgeip => {} // Read-only register
            Csr::Hgeie => asm_write_csr!("hgeie"),
            Csr::Henvcfg => asm_write_csr!("henvcfg"),
            Csr::Hcounteren => asm_write_csr!("hcounteren"),
            Csr::Htimedelta => asm_write_csr!("htimedelta"),
            Csr::Htval => asm_write_csr!("htval"),
            Csr::Htinst => asm_write_csr!("htinst"),
            Csr::Hgatp => asm_write_csr!("hgatp"),
            Csr::Vsstatus => asm_write_csr!("vsstatus"),
            Csr::Vsie => asm_write_csr!("vsie"),
            Csr::Vstvec => asm_write_csr!("vstvec"),
            Csr::Vsscratch => asm_write_csr!("vsscratch"),
            Csr::Vsepc => asm_write_csr!("vsepc"),
            Csr::Vscause => asm_write_csr!("vscause"),
            Csr::Vstval => asm_write_csr!("vstval"),
            Csr::Vsip => asm_write_csr!("vsip"),
            Csr::Vsatp => asm_write_csr!("vsatp"),
            Csr::Vstart => todo!(),
            Csr::Vxsat => todo!(),
            Csr::Vxrm => todo!(),
            Csr::Vcsr => todo!(),
            Csr::Vl => todo!(),
            Csr::Vtype => todo!(),
            Csr::Vlenb => todo!(),
            Csr::Seed => todo!(),
            Csr::Custom(_) => panic!("Custom CSR must be handled by the platform"),
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
            Csr::Cycle => todo!(),
            Csr::Time => {
                value = Plat::get_clint().read_mtime();
            }
            Csr::Instret => todo!(),
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
            Csr::Stimecmp => asm_read_csr!("stimecmp"),
            Csr::Hstatus => asm_read_csr!("hstatus"),
            Csr::Hedeleg => asm_read_csr!("hedeleg"),
            Csr::Hideleg => asm_read_csr!("hideleg"),
            Csr::Hvip => asm_read_csr!("hvip"),
            Csr::Hip => asm_read_csr!("hip"),
            Csr::Hie => asm_read_csr!("hie"),
            Csr::Hgeip => asm_read_csr!("hgeip"),
            Csr::Hgeie => asm_read_csr!("hgeie"),
            Csr::Henvcfg => asm_read_csr!("henvcfg"),
            Csr::Hcounteren => asm_read_csr!("hcounteren"),
            Csr::Htimedelta => asm_read_csr!("htimedelta"),
            Csr::Htval => asm_read_csr!("htval"),
            Csr::Htinst => asm_read_csr!("htinst"),
            Csr::Hgatp => asm_read_csr!("hgatp"),
            Csr::Vsstatus => asm_read_csr!("vsstatus"),
            Csr::Vsie => asm_read_csr!("vsie"),
            Csr::Vstvec => asm_read_csr!("vstvec"),
            Csr::Vsscratch => asm_read_csr!("vsscratch"),
            Csr::Vsepc => asm_read_csr!("vsepc"),
            Csr::Vscause => asm_read_csr!("vscause"),
            Csr::Vstval => asm_read_csr!("vstval"),
            Csr::Vsip => asm_read_csr!("vsip"),
            Csr::Vsatp => asm_read_csr!("vsatp"),
            Csr::Vstart => todo!(),
            Csr::Vxsat => todo!(),
            Csr::Vxrm => todo!(),
            Csr::Vcsr => todo!(),
            Csr::Vl => todo!(),
            Csr::Vtype => todo!(),
            Csr::Vlenb => todo!(),
            Csr::Seed => todo!(),
            Csr::Custom(_) => panic!("Custom CSR must be handled by the platform"),
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
        let is_henvcfg_present: bool = register_present!("henvcfg");
        let is_senvcfg_present: bool = register_present!("senvcfg");
        log::debug!(
            "Detecting available envcfg registers [menvcfg : {} | henvcfg: {} | senvcfg : {} ]",
            is_menvcfg_present,
            is_henvcfg_present,
            is_senvcfg_present,
        );

        // Check extension that enables bits in menvcfg
        let mut has_sstc_extension = false;
        let mut has_zicbom_extension = false;
        let mut has_zicboz_extension = false;
        if is_menvcfg_present {
            // Write bits for all known extensions to check if they are hardwired to 0
            let prev_menvcfg = Self::set_csr_bits(Csr::Menvcfg, menvcfg::ALL);
            let menvcfg_all = Self::read_csr(Csr::Menvcfg);
            Self::write_csr(Csr::Menvcfg, prev_menvcfg);

            // If a bit is not hardwired to 0 the corresponding extension is available
            if (menvcfg_all & menvcfg::STCE_FILTER) != 0 {
                has_sstc_extension = true;
            }
            if (menvcfg_all & (menvcfg::CBIE_FILTER | menvcfg::CBCFE_FILTER)) != 0 {
                has_zicbom_extension = true;
            }
            if (menvcfg_all & menvcfg::CBZE_FILTER) != 0 {
                has_zicboz_extension = true;
            }
        }

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

        // Detect performance counter extensions
        let is_mcycle_present: bool = register_present!("mcycle");

        // Save current CSRs
        let mstatus = Self::read_csr(Csr::Mstatus);
        let mtvec = Self::read_csr(Csr::Mtvec);

        // Read hart ID
        let hart = Self::read_csr(Csr::Mhartid);

        // Detect available interrupt IDs
        let available_int: usize;
        asm!(
            "csrc mstatus, {clear_mie}",   // Disable interrupts by clearing MIE in mstatus
            "csrrw {tmp}, mie, {all_int}", // Set all bits in the mie register
            "csrr {available_int}, mie",   // Read back wich bits are set to 1
            "csrw mie, {tmp}",             // Clear all bits in mie
            clear_mie = in(reg) mstatus::MIE_FILTER,
            all_int = in(reg) usize::MAX,
            available_int = out(reg) available_int,
            tmp = out(reg) _,
            options(nomem)
        );

        // Restore CSRs
        Self::write_csr(Csr::Mstatus, mstatus);
        Self::write_csr(Csr::Mtvec, mtvec);

        let misa = Self::read_csr(Csr::Misa);

        // Return hardware configuration
        HardwareCapability {
            interrupts: available_int,
            hart,
            _marker: PhantomData,
            available_reg: RegistersCapability {
                menvcfg: is_menvcfg_present,
                henvcfg: is_henvcfg_present,
                senvcfg: is_senvcfg_present,
                nb_pmp,
            },
            extensions: ExtensionsCapability {
                has_h_extension: (misa as usize & misa::H) != 0,
                has_s_extension: (misa as usize & misa::S) != 0,
                has_c_extension: (misa as usize & misa::C) != 0,
                has_sstc_extension,
                has_zicbom_extension,
                has_zicboz_extension,
                is_sstc_enabled: false, // Since the virtual menvcfg is initialized with 0
                has_v_extension: false,
                has_crypto_extension: false,
                has_zicntr: is_mcycle_present,
                has_zfinx: false,
                has_zihpm_extension: true,
                has_tee_extension: true,
            },
        }
    }

    unsafe fn run_vcpu(ctx: &mut VirtContext) {
        asm!(
            // We need to save some registers manually, the compiler can't handle those
            "add sp, sp, -32",
            "sd x3, (8*0)(sp)",
            "sd x4, (8*1)(sp)",
            "sd x8, (8*2)(sp)",
            "sd x9, (8*3)(sp)",
            // Jump into context switch code
            "jal x30, _run_vcpu",
            // Restore registers
            "ld x3, (8*0)(sp)",
            "ld x4, (8*1)(sp)",
            "ld x8, (8*2)(sp)",
            "ld x9, (8*3)(sp)",
            "add sp, sp, 32",
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

        // Fill the trap_info data structure
        ctx.trap_info.mepc = Arch::read_csr(Csr::Mepc);
        ctx.trap_info.mtval = Arch::read_csr(Csr::Mtval);
        ctx.trap_info.mip = Arch::read_csr(Csr::Mip);
        ctx.trap_info.mcause = Arch::read_csr(Csr::Mcause);
        ctx.trap_info.mstatus = Arch::read_csr(Csr::Mstatus);

        if ctx.extensions.has_h_extension {
            ctx.trap_info.mtval2 = Arch::read_csr(Mtval2);
            ctx.trap_info.mtinst = Arch::read_csr(Mtinst);
            ctx.trap_info.gva = ctx.trap_info.mstatus & GVA_FILTER != 0;
        }
    }

    unsafe fn sfencevma(vaddr: Option<usize>, asid: Option<usize>) {
        match (vaddr, asid) {
            (None, None) => asm!("sfence.vma"),
            (None, Some(asid)) => {
                asm!(
                    "sfence.vma x0, {asid}",
                    asid = in(reg) asid,
                )
            }
            (Some(vaddr), None) => {
                asm!(
                    "sfence.vma {vaddr}, x0",
                    vaddr = in(reg) vaddr
                )
            }
            (Some(vaddr), Some(asid)) => {
                asm!(
                    "sfence.vma {vaddr}, {asid}",
                    vaddr = in(reg) vaddr,
                    asid = in(reg) asid
                )
            }
        }
    }

    unsafe fn hfencegvma(vaddr: Option<usize>, asid: Option<usize>) {
        match (vaddr, asid) {
            (None, None) => asm!("hfence.gvma"),
            (None, Some(asid)) => {
                asm!(
                "hfence.gvma x0, {asid}",
                asid = in(reg) asid,
                )
            }
            (Some(vaddr), None) => {
                asm!(
                "hfence.gvma {vaddr}, x0",
                vaddr = in(reg) vaddr
                )
            }
            (Some(vaddr), Some(asid)) => {
                asm!(
                "hfence.gvma {vaddr}, {asid}",
                vaddr = in(reg) vaddr,
                asid = in(reg) asid
                )
            }
        }
    }

    unsafe fn hfencevvma(vaddr: Option<usize>, asid: Option<usize>) {
        match (vaddr, asid) {
            (None, None) => asm!("hfence.vvma"),
            (None, Some(asid)) => {
                asm!(
                "hfence.vvma x0, {asid}",
                asid = in(reg) asid,
                )
            }
            (Some(vaddr), None) => {
                asm!(
                "hfence.vvma {vaddr}, x0",
                vaddr = in(reg) vaddr
                )
            }
            (Some(vaddr), Some(asid)) => {
                asm!(
                "hfence.vvma {vaddr}, {asid}",
                vaddr = in(reg) vaddr,
                asid = in(reg) asid
                )
            }
        }
    }

    unsafe fn ifence() {
        asm!("fence.i");
    }

    unsafe fn clear_csr_bits(csr: Csr, bits_mask: usize) -> usize {
        let mut prev_value: usize = 0;
        macro_rules! asm_clear_csr_bits {
            ($reg:literal) => {
                asm!(
                    concat!("csrrc {prev}, ", $reg, ", {x}"),
                    x = in(reg) bits_mask,
                    prev = out(reg) prev_value,
                    options(nomem)
                )
            };
        }

        match csr {
            Csr::Mhartid => asm_clear_csr_bits!("mhartid"),
            Csr::Mstatus => asm_clear_csr_bits!("mstatus"),
            Csr::Misa => asm_clear_csr_bits!("misa"),
            Csr::Mie => asm_clear_csr_bits!("mie"),
            Csr::Mtvec => asm_clear_csr_bits!("mtvec"),
            Csr::Mscratch => asm_clear_csr_bits!("mscratch"),
            Csr::Mip => asm_clear_csr_bits!("mip"),
            Csr::Mvendorid => asm_clear_csr_bits!("mvendorid"),
            Csr::Marchid => asm_clear_csr_bits!("marchid"),
            Csr::Mimpid => asm_clear_csr_bits!("mimpid"),
            Csr::Pmpcfg(_) => todo!(),
            Csr::Pmpaddr(_) => todo!(),
            Csr::Mcycle => asm_clear_csr_bits!("mcycle"),
            Csr::Minstret => asm_clear_csr_bits!("minstret"),
            Csr::Cycle => todo!(),
            Csr::Time => todo!(),
            Csr::Instret => todo!(),
            Csr::Mhpmcounter(_) => todo!(),
            Csr::Mcountinhibit => asm_clear_csr_bits!("mcountinhibit"),
            Csr::Mhpmevent(_) => todo!(),
            Csr::Mcounteren => asm_clear_csr_bits!("mcounteren"),
            Csr::Menvcfg => asm_clear_csr_bits!("menvcfg"),
            Csr::Mseccfg => asm_clear_csr_bits!("mseccfg"),
            Csr::Mconfigptr => asm_clear_csr_bits!("mconfigptr"),
            Csr::Medeleg => asm_clear_csr_bits!("medeleg"),
            Csr::Mideleg => asm_clear_csr_bits!("mideleg"),
            Csr::Mtinst => asm_clear_csr_bits!("mtinst"),
            Csr::Mtval2 => asm_clear_csr_bits!("mtval2"),
            Csr::Tselect => asm_clear_csr_bits!("tselect"),
            Csr::Tdata1 => asm_clear_csr_bits!("tdata1"),
            Csr::Tdata2 => asm_clear_csr_bits!("tdata2"),
            Csr::Tdata3 => asm_clear_csr_bits!("tdata3"),
            Csr::Mcontext => asm_clear_csr_bits!("mcontext"),
            Csr::Dcsr => asm_clear_csr_bits!("dcsr"),
            Csr::Dpc => asm_clear_csr_bits!("dpc"),
            Csr::Dscratch0 => asm_clear_csr_bits!("dscratch0"),
            Csr::Dscratch1 => asm_clear_csr_bits!("dscratch1"),
            Csr::Mepc => asm_clear_csr_bits!("mepc"),
            Csr::Mcause => asm_clear_csr_bits!("mcause"),
            Csr::Mtval => asm_clear_csr_bits!("mtval"),
            Csr::Sstatus => asm_clear_csr_bits!("sstatus"),
            Csr::Sie => asm_clear_csr_bits!("sie"),
            Csr::Stvec => asm_clear_csr_bits!("stvec"),
            Csr::Scounteren => asm_clear_csr_bits!("scounteren"),
            Csr::Senvcfg => asm_clear_csr_bits!("senvcfg"),
            Csr::Sscratch => asm_clear_csr_bits!("sscratch"),
            Csr::Sepc => asm_clear_csr_bits!("sepc"),
            Csr::Scause => asm_clear_csr_bits!("scause"),
            Csr::Stval => asm_clear_csr_bits!("stval"),
            Csr::Sip => asm_clear_csr_bits!("sip"),
            Csr::Satp => asm_clear_csr_bits!("satp"),
            Csr::Scontext => asm_clear_csr_bits!("scontext"),
            Csr::Stimecmp => asm_clear_csr_bits!("stimecmp"),
            Csr::Hstatus => asm_clear_csr_bits!("hstatus"),
            Csr::Hedeleg => asm_clear_csr_bits!("hedeleg"),
            Csr::Hideleg => asm_clear_csr_bits!("hideleg"),
            Csr::Hvip => asm_clear_csr_bits!("hvip"),
            Csr::Hip => asm_clear_csr_bits!("hip"),
            Csr::Hie => asm_clear_csr_bits!("hie"),
            Csr::Hgeip => {} // Read only register
            Csr::Hgeie => asm_clear_csr_bits!("hgeie"),
            Csr::Henvcfg => asm_clear_csr_bits!("henvcfg"),
            Csr::Hcounteren => asm_clear_csr_bits!("hcounteren"),
            Csr::Htimedelta => asm_clear_csr_bits!("htimedelta"),
            Csr::Htval => asm_clear_csr_bits!("htval"),
            Csr::Htinst => asm_clear_csr_bits!("htinst"),
            Csr::Hgatp => asm_clear_csr_bits!("hgatp"),
            Csr::Vsstatus => asm_clear_csr_bits!("vsstatus"),
            Csr::Vsie => asm_clear_csr_bits!("vsie"),
            Csr::Vstvec => asm_clear_csr_bits!("vstvec"),
            Csr::Vsscratch => asm_clear_csr_bits!("vsscratch"),
            Csr::Vsepc => asm_clear_csr_bits!("vsepc"),
            Csr::Vscause => asm_clear_csr_bits!("vscause"),
            Csr::Vstval => asm_clear_csr_bits!("vstval"),
            Csr::Vsip => asm_clear_csr_bits!("vsip"),
            Csr::Vsatp => asm_clear_csr_bits!("vsatp"),
            Csr::Vstart => todo!(),
            Csr::Vxsat => todo!(),
            Csr::Vxrm => todo!(),
            Csr::Vcsr => todo!(),
            Csr::Vl => todo!(),
            Csr::Vtype => todo!(),
            Csr::Vlenb => todo!(),
            Csr::Seed => todo!(),
            Csr::Custom(_) => panic!("Custom CSR must be handled by the platform"),
            Csr::Unknown => (),
        };

        prev_value
    }

    unsafe fn set_csr_bits(csr: Csr, bits_mask: usize) -> usize {
        let mut prev_value: usize = 0;

        macro_rules! asm_set_csr_bits {
            ($reg:literal) => {
                unsafe {
                    asm!(
                        concat!("csrrs {prev}, ", $reg, ", {x}"),
                        x = in(reg) bits_mask,
                        prev = out(reg) prev_value,
                        options(nomem)
                    )
                }
            };
        }

        match csr {
            Csr::Mhartid => asm_set_csr_bits!("mhartid"),
            Csr::Mstatus => asm_set_csr_bits!("mstatus"),
            Csr::Misa => asm_set_csr_bits!("misa"),
            Csr::Mie => asm_set_csr_bits!("mie"),
            Csr::Mtvec => asm_set_csr_bits!("mtvec"),
            Csr::Mscratch => asm_set_csr_bits!("mscratch"),
            Csr::Mip => asm_set_csr_bits!("mip"),
            Csr::Mvendorid => asm_set_csr_bits!("mvendorid"),
            Csr::Marchid => asm_set_csr_bits!("marchid"),
            Csr::Mimpid => asm_set_csr_bits!("mimpid"),
            Csr::Pmpcfg(_) => todo!(),
            Csr::Pmpaddr(_) => todo!(),
            Csr::Mcycle => asm_set_csr_bits!("mcycle"),
            Csr::Minstret => asm_set_csr_bits!("minstret"),
            Csr::Cycle => todo!(),
            Csr::Time => todo!(),
            Csr::Instret => todo!(),
            Csr::Mhpmcounter(_) => todo!(),
            Csr::Mcountinhibit => asm_set_csr_bits!("mcountinhibit"),
            Csr::Mhpmevent(_) => todo!(),
            Csr::Mcounteren => asm_set_csr_bits!("mcounteren"),
            Csr::Menvcfg => asm_set_csr_bits!("menvcfg"),
            Csr::Mseccfg => asm_set_csr_bits!("mseccfg"),
            Csr::Mconfigptr => asm_set_csr_bits!("mconfigptr"),
            Csr::Medeleg => asm_set_csr_bits!("medeleg"),
            Csr::Mideleg => asm_set_csr_bits!("mideleg"),
            Csr::Mtinst => asm_set_csr_bits!("mtinst"),
            Csr::Mtval2 => asm_set_csr_bits!("mtval2"),
            Csr::Tselect => asm_set_csr_bits!("tselect"),
            Csr::Tdata1 => asm_set_csr_bits!("tdata1"),
            Csr::Tdata2 => asm_set_csr_bits!("tdata2"),
            Csr::Tdata3 => asm_set_csr_bits!("tdata3"),
            Csr::Mcontext => asm_set_csr_bits!("mcontext"),
            Csr::Dcsr => asm_set_csr_bits!("dcsr"),
            Csr::Dpc => asm_set_csr_bits!("dpc"),
            Csr::Dscratch0 => asm_set_csr_bits!("dscratch0"),
            Csr::Dscratch1 => asm_set_csr_bits!("dscratch1"),
            Csr::Mepc => asm_set_csr_bits!("mepc"),
            Csr::Mcause => asm_set_csr_bits!("mcause"),
            Csr::Mtval => asm_set_csr_bits!("mtval"),
            Csr::Sstatus => asm_set_csr_bits!("sstatus"),
            Csr::Sie => asm_set_csr_bits!("sie"),
            Csr::Stvec => asm_set_csr_bits!("stvec"),
            Csr::Scounteren => asm_set_csr_bits!("scounteren"),
            Csr::Senvcfg => asm_set_csr_bits!("senvcfg"),
            Csr::Sscratch => asm_set_csr_bits!("sscratch"),
            Csr::Sepc => asm_set_csr_bits!("sepc"),
            Csr::Scause => asm_set_csr_bits!("scause"),
            Csr::Stval => asm_set_csr_bits!("stval"),
            Csr::Sip => asm_set_csr_bits!("sip"),
            Csr::Satp => asm_set_csr_bits!("satp"),
            Csr::Scontext => asm_set_csr_bits!("scontext"),
            Csr::Stimecmp => asm_set_csr_bits!("stimecmp"),
            Csr::Hstatus => asm_set_csr_bits!("hstatus"),
            Csr::Hedeleg => asm_set_csr_bits!("hedeleg"),
            Csr::Hideleg => asm_set_csr_bits!("hideleg"),
            Csr::Hvip => asm_set_csr_bits!("hvip"),
            Csr::Hip => asm_set_csr_bits!("hip"),
            Csr::Hie => asm_set_csr_bits!("hie"),
            Csr::Hgeip => asm_set_csr_bits!("hgeip"),
            Csr::Hgeie => asm_set_csr_bits!("hgeie"),
            Csr::Henvcfg => asm_set_csr_bits!("henvcfg"),
            Csr::Hcounteren => asm_set_csr_bits!("hcounteren"),
            Csr::Htimedelta => asm_set_csr_bits!("htimedelta"),
            Csr::Htval => asm_set_csr_bits!("htval"),
            Csr::Htinst => asm_set_csr_bits!("htinst"),
            Csr::Hgatp => asm_set_csr_bits!("hgatp"),
            Csr::Vsstatus => asm_set_csr_bits!("vsstatus"),
            Csr::Vsie => asm_set_csr_bits!("vsie"),
            Csr::Vstvec => asm_set_csr_bits!("vstvec"),
            Csr::Vsscratch => asm_set_csr_bits!("vsscratch"),
            Csr::Vsepc => asm_set_csr_bits!("vsepc"),
            Csr::Vscause => asm_set_csr_bits!("vscause"),
            Csr::Vstval => asm_set_csr_bits!("vstval"),
            Csr::Vsip => asm_set_csr_bits!("vsip"),
            Csr::Vsatp => asm_set_csr_bits!("vsatp"),
            Csr::Vstart => todo!(),
            Csr::Vxsat => todo!(),
            Csr::Vxrm => todo!(),
            Csr::Vcsr => todo!(),
            Csr::Vl => todo!(),
            Csr::Vtype => todo!(),
            Csr::Vlenb => todo!(),
            Csr::Seed => todo!(),
            Csr::Custom(_) => panic!("Custom CSR must be handled by the platform"),
            Csr::Unknown => (),
        };

        prev_value
    }

    /// Emulates a load instruction using MPRV = 1.
    ///
    /// # Safety
    /// This function performs a load using MPRV = 1, whose behavior depends on the privilegd state
    /// and in particular PMP and page tables. The privileged state must be configured properly to
    /// ensure the proper access rights are enforced.
    unsafe fn handle_virtual_load(instr: LoadInstr, ctx: &mut VirtContext) {
        // Zero out mcause to check if a trap occured during emulation
        Self::write_csr(Csr::Mcause, 0);

        // Set the MPP mode to match the vMPP
        let prev_mpp = set_mpp(parse_mpp_return_mode(ctx.csr.mstatus));
        let prev_satp = Self::write_csr(Csr::Satp, ctx.csr.satp);

        // Changes to SATP require an sfence instruction to take effect
        Self::sfencevma(None, None);

        let mut value: usize = 0;
        let addr: usize = utils::calculate_addr(ctx.get(instr.rs1), instr.imm);

        match (instr.len, instr.is_unsigned) {
            (Width::Byte, false) => asm_mprv_mem_op!("lb", addr, value),
            (Width::Byte2, false) => asm_mprv_mem_op!("lh", addr, value),
            (Width::Byte4, false) => asm_mprv_mem_op!("lw", addr, value),
            (Width::Byte8, false) => asm_mprv_mem_op!("ld", addr, value),
            (Width::Byte, true) => asm_mprv_mem_op!("lbu", addr, value),
            (Width::Byte2, true) => asm_mprv_mem_op!("lhu", addr, value),
            (Width::Byte4, true) => asm_mprv_mem_op!("lwu", addr, value),
            _ => panic!("Unknown load instruction"),
        };

        // If `mcause` is not zero then the load caused a trap.
        // In that case we need to update the trap info and inject the trap back.
        let cause = Self::read_csr(Csr::Mcause);
        if cause != 0 {
            ctx.emulate_firmware_trap();
        } else {
            ctx.set(instr.rd, value);
            ctx.pc += if instr.is_compressed { 2 } else { 4 };
        }

        // Restore the original values
        Self::write_csr(Csr::Satp, prev_satp);
        set_mpp(prev_mpp);

        // Ensure memory consistency
        Self::sfencevma(None, None);
    }

    /// Emulates a store instruction using MPRV = 1.
    ///
    /// # Safety
    /// This function performs a store using MPRV = 1, whose behavior depends on the privilegd
    /// state and in particular PMP and page tables. The privileged state must be configured
    /// properly to ensure the proper access rights are enforced.
    unsafe fn handle_virtual_store(instr: StoreInstr, ctx: &mut VirtContext) {
        // Zero out mcause to check if a trap occured during emulation
        Self::write_csr(Csr::Mcause, 0);

        // Set the MPP mode to match the vMPP
        let prev_mpp = set_mpp(parse_mpp_return_mode(ctx.csr.mstatus));
        let prev_satp = Self::write_csr(Csr::Satp, ctx.csr.satp);

        // Changes to SATP require an sfence instruction to take effect
        Self::sfencevma(None, None);

        let mut value: usize = ctx.get(instr.rs2);
        let addr: usize = utils::calculate_addr(ctx.get(instr.rs1), instr.imm);

        match instr.len {
            Width::Byte => asm_mprv_mem_op!("sb", addr, value),
            Width::Byte2 => asm_mprv_mem_op!("sh", addr, value),
            Width::Byte4 => asm_mprv_mem_op!("sw", addr, value),
            Width::Byte8 => asm_mprv_mem_op!("sd", addr, value),
        };

        // Silence unused warning caused by the macro
        let _ = value;

        // If `mcause` is not zero then the load caused a trap.
        // In that case we need to update the trap info and inject the trap back.
        let cause = Self::read_csr(Csr::Mcause);
        if cause != 0 {
            ctx.emulate_firmware_trap();
        } else {
            ctx.pc += if instr.is_compressed { 2 } else { 4 };
        }

        // Restore the original values
        Self::write_csr(Csr::Satp, prev_satp);
        set_mpp(prev_mpp);

        // Ensure memory consistency
        Self::sfencevma(None, None);
    }

    unsafe fn read_bytes_from_mode(src: *const u8, dest: &mut [u8], mode: Mode) -> Result<(), ()> {
        let mut src = src as usize;
        let mut success: usize = 1;

        // Save the state of exception-related CSRs, as we might overwrite them if an error occurs
        let prev_mepc = Self::read_csr(Csr::Mepc);
        let prev_mcause = Self::read_csr(Csr::Mcause);
        let prev_mstatus = Self::read_csr(Csr::Mstatus);

        // Set mstatus.MPP to mode
        let prev_mode = set_mpp(mode);
        for i in 0..dest.len() {
            let mut byte_read: u8 = 0;
            unsafe {
                asm!(
                // Try
                "la {r_mtvec}, 0f",
                "csrrw {r_mtvec}, mtvec, {r_mtvec}",  // Trap to catch-block if an exception occurs

                // Set the mstatus.MPRV bit to 1
                "csrs mstatus, {mprv_filter}",
                // Read byte at src
                "lb {byte}, 0x00({src})",
                // Set the mstatus.MPRV bit to 0
                "csrc mstatus, {mprv_filter}",
                "j 1f", // Jump to finally if the read was successful

                // Catch
                ".align 4",
                "0:",
                "li {success}, 0",
                "la {byte}, 1f",
                "csrw mepc, {byte}",
                "mret",  // Jump to finally and set mstatus.MPRV to 0

                // Finally
                ".align 4",
                "1:",
                "csrw mtvec, {r_mtvec}", // Restore mtvec
                src = in(reg) src,
                mprv_filter = in(reg) mstatus::MPRV_FILTER,
                byte = inout(reg) byte_read,
                success = inout(reg) success,
                r_mtvec = out(reg) _,
                )
            }

            if success == 0 {
                Self::write_csr(Csr::Mepc, prev_mepc);
                Self::write_csr(Csr::Mcause, prev_mcause);
                Self::write_csr(Csr::Mstatus, prev_mstatus);
                return Err(());
            }

            dest[i] = byte_read;
            src += 1;
        }

        set_mpp(prev_mode);
        Ok(())
    }

    unsafe fn store_bytes_from_mode(src: &mut [u8], dest: *const u8, mode: Mode) -> Result<(), ()> {
        let mut dest = dest as usize;
        let mut success: usize = 1;

        // Save the state of exception-related CSRs, as we might overwrite them if an error occurs
        let prev_mepc = Self::read_csr(Csr::Mepc);
        let prev_mcause = Self::read_csr(Csr::Mcause);
        let prev_mstatus = Self::read_csr(Csr::Mstatus);

        // Set mstatus.MPP to mode
        let prev_mode = set_mpp(mode);
        for i in 0..src.len() {
            let byte_value: u8 = src[i];
            unsafe {
                asm!(
                // Try
                "la {r_mtvec}, 0f",
                "csrrw {r_mtvec}, mtvec, {r_mtvec}",  // Trap to catch-block if an exception occurs

                // Set the mstatus.MPRV bit to 1
                "csrs mstatus, {mprv_filter}",
                // Store byte at src
                "sb {byte}, 0x00({dest})",
                // Set the mstatus.MPRV bit to 0
                "csrc mstatus, {mprv_filter}",
                "j 1f", // Jump to finally if the read was successful

                // Catch
                ".align 4",
                "0:",
                "li {success}, 0",
                "la {byte}, 1f",
                "csrw mepc, {byte}",
                "mret",  // Jump to finally and set mstatus.MPRV to 0

                // Finally
                ".align 4",
                "1:",
                "csrw mtvec, {r_mtvec}", // Restore mtvec
                dest = in(reg) dest,
                mprv_filter = in(reg) mstatus::MPRV_FILTER,
                byte = in(reg) byte_value,
                success = inout(reg) success,
                r_mtvec = out(reg) _,
                )
            }
            if success == 0 {
                Self::write_csr(Csr::Mepc, prev_mepc);
                Self::write_csr(Csr::Mcause, prev_mcause);
                Self::write_csr(Csr::Mstatus, prev_mstatus);
                return Err(());
            }
            dest += 1;
        }

        set_mpp(prev_mode);
        Ok(())
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
                addr = inout(reg) usize::MAX => read_addr,
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

// —————————————————————————————— Virtual Trap Handler —————————————————————————————— //

// When the pMPRV (Modify Privilege) bit is set and a trap occurs, the default behavior in
// _raw_trap_handler is to attempt context storage using address translation.
// However, this approach is incorrect.
//
// To address this issue, we set mtvec to point to this custom trap handler.
// The purpose of this handler is straightforward: it skips the illegal instruction, assuming it is
// 4 bytes wide.
global_asm!(
    r#"
.text
.align 4
.global _mprv_trap_handler
_mprv_trap_handler:
    csrrw t5, mepc, t5
    addi  t5, t5, 4
    csrrw t5, mepc, t5
    mret
"#,
);

extern "C" {
    fn _raw_trap_handler();
    fn _tracing_trap_handler();
    fn _mprv_trap_handler();
}
