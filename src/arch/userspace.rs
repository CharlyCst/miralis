//! A mock of architecture specific features when running in user space.
//!
//! This implementation is useful for running Miralis on the host (potentially non-riscv)
//! architecture, such as when running unit tests.

use core::cell::RefCell;
use core::marker::PhantomData;
use std::thread_local;

use softcore_rv64::{config, new_core, registers as reg, Core};

use super::{mie, Architecture, Csr, ExtensionsCapability, Mode};
use crate::arch::HardwareCapability;
use crate::decoder::{LoadInstr, StoreInstr};
use crate::logger;
use crate::virt::VirtContext;

// Each thread gets its own copy of the core, this prevent tests using different threads inside a
// same process to share the same core.
thread_local! {
    /// A software RISC-V core that emulates a real CPU.
    ///
    /// When running Miralis in user-space (on any architecture) we use an in-process software
    /// implementation of a RISC-V core. We perform all privileged operations that would normally
    /// require assembly against that software core. For instance, all CSR operations are executed
    /// against the software core.
    ///
    /// The software core can be queried, for instance to check which addresses are currently
    /// protected by the PMP, or if there are any pending interrupts. This enables testing how
    /// Miralis interacts with the hardware directly from unit tests, rather than within QEMU.
    ///
    /// We use one core per thread to prevent interference among threads, such as when running
    /// `cargo test`. Therefore, the core lives in threat local storage and must be access using
    /// the `thread_loca!` API.
    ///
    /// Usage:
    ///
    /// ```
    /// SOFT_CORE.with_borrow_mut(|core| {
    ///     // The `core` can be accessed within the closure
    ///     core.set(reg::X1, 0x42);
    ///     core.csrrw(reg::X0, csr::MSCRATCH, reg::X1).unwrap();
    /// });
    /// ```
    pub static SOFT_CORE: RefCell<Core> = {
        let mut core = new_core(config::U74);
        core.reset();
        RefCell::new(core)
    };
}

/// User space mock, running on the host architecture.
pub struct HostArch {}

impl Architecture for HostArch {
    fn init() {
        todo!()
    }

    fn wfi() {
        logger::debug!("Userspace wfi");
    }

    unsafe fn write_pmpaddr(idx: usize, value: usize) {
        const PMPADDR_BASE: u64 = 0x3B0;
        let csr_id = PMPADDR_BASE + idx as u64;

        SOFT_CORE.with_borrow_mut(|core| {
            core.set(reg::X1, value as u64);
            core.csrrw(reg::X0, csr_id, reg::X1).unwrap();
        });
    }

    unsafe fn write_pmpcfg(idx: usize, configuration: usize) {
        const PMPCFG_BASE: u64 = 0x3A0;
        let csr_id = PMPCFG_BASE + idx as u64;

        SOFT_CORE.with_borrow_mut(|core| {
            core.set(reg::X1, configuration as u64);
            core.csrrw(reg::X0, csr_id, reg::X1).unwrap();
        });
    }

    unsafe fn run_vcpu(_ctx: &mut crate::virt::VirtContext) {
        todo!()
    }

    unsafe fn sfencevma(_vaddr: Option<usize>, _asid: Option<usize>) {
        logger::debug!("Userspace sfencevma");
    }

    unsafe fn hfencegvma(_: Option<usize>, _: Option<usize>) {
        logger::debug!("Userspace hfencegvma")
    }

    unsafe fn hfencevvma(_: Option<usize>, _: Option<usize>) {
        logger::debug!("Userspace hfencevvma")
    }

    unsafe fn ifence() {
        logger::debug!("Userspace ifence");
    }

    unsafe fn detect_hardware() -> HardwareCapability {
        HardwareCapability {
            interrupts: mie::ALL_INT,
            hart: 0,
            _marker: PhantomData,
            available_reg: super::RegistersCapability {
                menvcfg: true,
                henvcfg: false,
                senvcfg: true,
                nb_pmp: 16,
            },
            extensions: ExtensionsCapability {
                has_h_extension: false,
                has_s_extension: true,
                has_v_extension: false,
                has_c_extension: true,
                has_crypto_extension: false,
                has_zicntr: true,
                has_zfinx: false,
                has_sstc_extension: false,
                is_sstc_enabled: false,
                has_zihpm_extension: false,
                has_zicboz_extension: false,
                has_zicbom_extension: false,
                has_tee_extension: false,
            },
        }
    }

    fn read_csr(csr: Csr) -> usize {
        SOFT_CORE.with_borrow_mut(|core| {
            core.csrrs(reg::X1, csr.idx() as u64, reg::X0).unwrap();
            core.get(reg::X1) as usize
        })
    }

    unsafe fn write_csr(csr: Csr, value: usize) -> usize {
        SOFT_CORE.with_borrow_mut(|core| {
            core.set(reg::X1, value as u64);
            core.csrrw(reg::X2, csr.idx() as u64, reg::X1).unwrap();
            core.get(reg::X2) as usize
        })
    }

    unsafe fn clear_csr_bits(csr: Csr, bits_mask: usize) -> usize {
        Self::write_csr(csr, Self::read_csr(csr) & !bits_mask)
    }

    unsafe fn set_csr_bits(csr: Csr, bits_mask: usize) -> usize {
        Self::write_csr(csr, Self::read_csr(csr) | bits_mask)
    }

    unsafe fn handle_virtual_load(_instr: LoadInstr, _ctx: &mut VirtContext) {
        todo!()
    }

    unsafe fn handle_virtual_store(_instr: StoreInstr, _ctx: &mut VirtContext) {
        todo!()
    }

    unsafe fn read_bytes_from_mode(
        _src: *const u8,
        _dest: &mut [u8],
        _mode: Mode,
    ) -> Result<(), ()> {
        todo!();
    }

    unsafe fn store_bytes_from_mode(
        _src: &mut [u8],
        _dest: *const u8,
        _mode: Mode,
    ) -> Result<(), ()> {
        todo!()
    }
}
