//! A mock of architecture specific features when running in user space.
//!
//! This implementation is useful for running Mirage on the host (potentially non-riscv)
//! architecutre, such as when running unit tests.

use super::Architecture;
use crate::arch::{HardwareCapability, PmpGroup};
use crate::host::MirageContext;
use crate::main;
use crate::virt::VirtContext;

/// User space mock, running on the host architecture.
pub struct HostArch {}

impl Architecture for HostArch {
    fn init() {
        // Use main to avoid "never used" warnings.
        let _ = main;

        todo!()
    }

    fn wfi() {
        todo!()
    }

    fn read_misa() -> usize {
        todo!()
    }

    fn read_mstatus() -> usize {
        todo!()
    }

    fn read_mhartid() -> usize {
        todo!()
    }

    unsafe fn set_mpp(mode: super::Mode) {
        let _ = mode.to_bits();
        todo!()
    }

    unsafe fn write_mstatus(_mstatus: usize) {
        todo!()
    }

    unsafe fn write_pmp(_pmp: &PmpGroup) {
        todo!()
    }

    unsafe fn run_vcpu(_ctx: &mut crate::virt::VirtContext) {
        todo!()
    }

    unsafe fn get_raw_faulting_instr(_trap_info: &super::TrapInfo) -> usize {
        todo!()
    }

    fn read_mtvec() -> usize {
        todo!()
    }

    unsafe fn sfence_vma() {
        todo!()
    }

    unsafe fn switch_from_firmware_to_payload(_ctx: &mut VirtContext, _mctx: &mut MirageContext) {
        todo!()
    }

    unsafe fn switch_from_payload_to_firmware(_ctx: &mut VirtContext, _mctx: &mut MirageContext) {
        todo!()
    }

    unsafe fn detect_hardware() -> HardwareCapability {
        todo!()
    }
}
