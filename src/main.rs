//! Miralis entry point
//!
//! The main function is called directly after platform specific minimal setup (such as
//! configuration of the stack).

// Mark the crate as no_std and no_main, but only when not running tests.
// We need both std and main to be able to run tests in user-space on the host architecture.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(
    // used for meaningful panic code
    panic_info_message,
    // used for calculating offsets for assembly
    asm_const,
    // const_mut_ref for LinkedList implementation used in the heap allocator
    const_mut_refs,
    pointer_is_aligned,
    pointer_is_aligned_to,
    result_option_inspect,
    pointer_byte_offsets,
    // used for formal verification framework (RefinedRust annotations)
    register_tool,
    custom_inner_attributes,
    stmt_expr_attributes,
)]

extern crate alloc;

mod arch;
mod benchmark;
mod config;
mod debug;
mod decoder;
mod device;
mod driver;
mod host;
mod logger;
mod platform;
mod policy;
mod utils;
mod virt;
mod ace;
mod device_tree;
mod monitor_switch;

use alloc::vec::Vec;
use core::fmt::Pointer;
use core::ptr;
use arch::{Arch, Architecture};
use benchmark::{Benchmark, Counter, Scope};
use config::PLATFORM_NAME;
use flattened_device_tree::error::FdtError;
use flattened_device_tree::FlattenedDeviceTree;
use platform::{init, Plat, Platform};
use policy::{Policy, PolicyModule};

use crate::arch::{misa, Csr, ExtensionsCapability, HardwareCapability, Register};
use crate::host::MiralisContext;
use crate::virt::traits::*;
use crate::virt::{ExecutionMode, VirtContext};

use fdt_rs::base::{DevTree, DevTreeNode, DevTreeProp};
use fdt_rs::prelude::{FallibleIterator, PropReader};
use spin::{Mutex, Once};
use crate::ace::core::architecture::control_status_registers::ReadWriteRiscvCsr;
use crate::ace::core::architecture::CSR;
use crate::ace::core::architecture::fence::fence_wo;
use crate::ace::core::control_data::HardwareHart;
use crate::ace::core::initialization::HARTS_STATES;
use crate::ace::non_confidential_flow::apply_to_hypervisor::ApplyToHypervisorHart;
use crate::ace::non_confidential_flow::handlers::supervisor_binary_interface::SbiResponse;
use crate::ace::non_confidential_flow::NonConfidentialFlow;

// Defined in the linker script
#[cfg(not(feature = "userspace"))]
extern "C" {
    pub(crate) static _stack_start: u8;
    pub(crate) static _bss_start: u8;
    pub(crate) static _bss_stop: u8;
    pub(crate) static _stack_top: u8;
    pub(crate) static _start_address: u8;
}

// When building for userspace (i.e. to run as a process on the host machine) we do not use the
// custom linker script, so some of the variables from the linker scripts needs to be re-defined.
//
// We define them here with dummy values. The definitions are mutable statics to mimic the `extern
// "C"` behavior.
#[cfg(feature = "userspace")]
#[allow(non_upper_case_globals)]
mod userspace_linker_definitions {
    pub(crate) static mut _stack_start: u8 = 0;
    pub(crate) static mut _start_address: u8 = 0;
}

#[cfg(feature = "userspace")]
use userspace_linker_definitions::*;
use crate::device_tree::divide_memory_region_size;
use crate::monitor_switch::{overwrite_hardware_hart_with_virtctx, overwrite_virtctx_with_hardware_hart};

pub(crate) extern "C" fn main(_hart_id: usize, device_tree_blob_addr: usize) -> ! {
    // On the VisionFive2 board there is an issue with a hart_id
    // Identification, so we have to reassign it for now
    let hart_id = Arch::read_csr(Csr::Mhartid);

    init();
    log::info!("Hello, world!");
    log::info!("Platform name: {}", Plat::name());
    log::info!("Policy module: {}", Policy::name());
    log::info!("Hart ID: {}", hart_id);
    log::debug!("misa:    0x{:x}", Arch::read_csr(Csr::Misa));
    log::debug!(
        "vmisa:   0x{:x}",
        Arch::read_csr(Csr::Misa) & !misa::DISABLED
    );
    log::debug!("mstatus: 0x{:x}", Arch::read_csr(Csr::Mstatus));
    log::info!("DTS address: 0x{:x}", device_tree_blob_addr);

    // INIT ACE
    // Step 1: Break forward tree
    divide_memory_region_size(device_tree_blob_addr);

    // Step 2: Initialise
    match ace::core::initialization::init_security_monitor(device_tree_blob_addr as *const u8) {
        Ok(_) => log::info!("Initialized ACE security monitor."),
        Err(e) => log::info!("Error occurred: {:?}", e),
    }
    // END INIT ACE

    log::info!("Preparing jump into firmware");
    let firmware_addr = Plat::load_firmware();
    log::debug!("Firmware loaded at: {:x}", firmware_addr);

    let mut policy: Policy = Policy::init();

    // Detect hardware capabilities
    // SAFETY: this must happen before hardware initialization
    let hw = unsafe { Arch::detect_hardware() };
    // Initialize Miralis's own context
    let mut mctx = MiralisContext::new(hw);

    // Initialize the virtual context and configure architecture
    let mut ctx = VirtContext::new(hart_id, mctx.pmp.nb_virt_pmp, mctx.hw.extensions.clone());
    unsafe {
        // Set return address, mode and PMP permissions
        Arch::set_mpp(arch::Mode::U);
        // Update the PMPs prior to first entry
        Arch::write_pmp(&mctx.pmp).flush();

        // Configure the firmware context
        ctx.set(Register::X10, hart_id);
        ctx.set(Register::X11, device_tree_blob_addr);
        ctx.set_csr(
            Csr::Misa,
            Arch::read_csr(Csr::Misa) & !misa::DISABLED,
            &mut mctx,
        );
        ctx.pc = firmware_addr;
    }

    // In case we compile Miralis as firmware, we stop execution at that point for the moment
    // This allows us to run Miralis on top as an integration test for the moment
    // In the future, we plan to run Miralis "as firmware" running a firmware
    if PLATFORM_NAME == "miralis" {
        log::info!("Successfully initialized Miralis as a firmware");
        Plat::exit_success();
    }

    main_loop(&mut ctx, &mut mctx, &mut policy);
}

extern "C" {
    // Assembly function that is an entry point to the security monitor from the hypervisor or a virtual machine.
    fn enter_from_hypervisor_or_vm_asm() -> !;

    /// To ensure safety, specify all possible valid states that KVM expects to see and prove that security monitor
    /// never returns to KVM with other state. For example, only a subset of exceptions/interrupts can be handled by KVM.
    /// KVM kill the vcpu if it receives unexpected exception because it does not know what to do with it.
    fn exit_to_hypervisor_asm() -> !;

    // Miralis raw trap handler
    fn _raw_trap_handler();
}

/// This functions transfers the control to the ACE security monitor from Miralis
fn miralis_to_ace_ctx_switch(virt_ctx: &mut VirtContext) {
    // Step 0: Get ACE Context
    let hart_id = virt_ctx.hart_id;
    assert!(hart_id == 0, "Implement this code for multihart");

    while !HARTS_STATES.is_completed() {
        fence_wo();
    }

    let mut harts = HARTS_STATES.get().expect("Bug. Could not set mscratch before initializing memory region for harts states").lock();
    let mut ace_ctx: &mut HardwareHart = harts.get_mut(hart_id).expect("Bug. Incorrectly setup memory region for harts states");

    // Step 1: Overwrite Hardware hart with virtcontext
    overwrite_hardware_hart_with_virtctx(ace_ctx, virt_ctx);
    // Step 1-bis: Set mepc value to pc before jumping
    ace_ctx.hypervisor_hart.hypervisor_hart_state.csrs.mepc = ReadWriteRiscvCsr(virt_ctx.pc);

    // TODO: Is it enough?
    // Step 2: Change mscratch value
    unsafe {
        CSR.mscratch.write(ace_ctx as *const _ as usize);
    }

    // Step 3: Change trap handler
    let trap_vector_address = enter_from_hypervisor_or_vm_asm as usize;
    ace_ctx.hypervisor_hart_mut().csrs_mut().mtvec.write((trap_vector_address >> 2) << 2);

    // Step 4: Jump to the payload - todo: Do we need to apply a response? It seems not to be the case
    unsafe {
        exit_to_hypervisor_asm();
    }
}

// TODO: Make 3 input variables global
fn ace_to_miralis_ctx_switch(ace_ctx: &mut HardwareHart, ctx: &mut VirtContext, mctx: &mut MiralisContext, policy: &mut Policy) {
    // Step 0: Get Virt context
    // TODO: Comment on handle cela?

    // Step 1: Overwrite Hardware hart with virtcontext
    // overwrite_virtctx_with_hardware_hart();
    // Step 1-bis: Set mepc value to pc before jumping
    // TODO: Is it the correct register to set pc?
    ctx.pc = ace_ctx.hypervisor_hart.hypervisor_hart_state.csrs.mepc.read();

    // Step 2: Change mscratch value
    // Normally here we should not do anything as miralis installs mepc in _run_vcpu

    // Step 3: Change trap handler - install Miralis trap handler
    Arch::install_handler(_raw_trap_handler as usize);

    // Step 4: Jump in the Miralis trap handler - and enter the main loop
    handle_trap(ctx, mctx, policy);
    main_loop(ctx, mctx, policy)
}


fn main_loop(ctx: &mut VirtContext, mctx: &mut MiralisContext, policy: &mut Policy) -> ! {
    loop {
        Benchmark::start_interval_counters(Scope::RunVCPU);

        unsafe {
            Arch::run_vcpu(ctx);
        }

        Benchmark::stop_interval_counters(Scope::RunVCPU);
        Benchmark::start_interval_counters(Scope::HandleTrap);

        handle_trap(ctx, mctx, policy);

        Benchmark::stop_interval_counters(Scope::HandleTrap);
        Benchmark::increment_counter(Counter::TotalExits);
    }
}

fn handle_trap(ctx: &mut VirtContext, mctx: &mut MiralisContext, policy: &mut Policy) {
    if log::log_enabled!(log::Level::Trace) {
        log_ctx(ctx);
    }

    if let Some(max_exit) = config::MAX_FIRMWARE_EXIT {
        if ctx.nb_exits + 1 >= max_exit {
            log::error!("Reached maximum number of exits: {}", ctx.nb_exits);
            Plat::exit_failure();
        }
    }

    if ctx.trap_info.is_from_mmode() {
        // Trap comes from M mode: Miralis
        handle_miralis_trap(ctx);
        return;
    }

    // Perform emulation
    let exec_mode = ctx.mode.to_exec_mode();

    // Keep track of the number of exit
    ctx.nb_exits += 1;
    match exec_mode {
        ExecutionMode::Firmware => ctx.handle_firmware_trap(mctx, policy),
        ExecutionMode::Payload => ctx.handle_payload_trap(mctx, policy),
    }

    if exec_mode == ExecutionMode::Firmware {
        Benchmark::increment_counter(Counter::FirmwareExits);
    }

    if exec_mode != ctx.mode.to_exec_mode() {
        Benchmark::increment_counter(Counter::WorldSwitches);
    }

    // Check for execution mode change
    match (exec_mode, ctx.mode.to_exec_mode()) {
        (ExecutionMode::Firmware, ExecutionMode::Payload) => {
            //log::warn!("Execution mode: Firmware -> Payload");
            unsafe { ctx.switch_from_firmware_to_payload(mctx) };
            policy.switch_from_firmware_to_payload(ctx, mctx);
            miralis_to_ace_ctx_switch(ctx);
        }
        (ExecutionMode::Payload, ExecutionMode::Firmware) => {
            //log::warn!("Execution mode: Payload -> Firmware {:?}", ctx.trap_info);
            unsafe { ctx.switch_from_payload_to_firmware(mctx) };
            policy.switch_from_payload_to_firmware(ctx, mctx);
        }
        _ => {} // No execution mode transition
    }

    unsafe {
        // Commit the PMP to hardware
        Arch::write_pmp(&mctx.pmp).flush_if_required(&mut mctx.pmp);
    }
}

/// Handle the trap coming from miralis
fn handle_miralis_trap(ctx: &mut VirtContext) {
    let trap = &ctx.trap_info;
    log::error!("Unexpected trap while executing Miralis");
    log::error!("  cause:   {} ({:?})", trap.mcause, trap.get_cause());
    log::error!("  mepc:    0x{:x}", trap.mepc);
    log::error!("  mtval:   0x{:x}", trap.mtval);
    log::error!("  mstatus: 0x{:x}", trap.mstatus);
    log::error!("  mip:     0x{:x}", trap.mip);

    todo!("Miralis trap handler entered");
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    unsafe { debug::log_stack_usage() };
    Plat::exit_failure();
}

// —————————————————————————————— Debug Helper —————————————————————————————— //

/// Log the current context using the trace log level.
fn log_ctx(ctx: &VirtContext) {
    let trap_info = &ctx.trap_info;
    log::trace!(
        "Trapped on hart {}:  {:?}",
        ctx.hart_id,
        ctx.trap_info.get_cause()
    );
    log::trace!(
        "  mstatus: 0x{:<16x} mepc: 0x{:x}",
        trap_info.mstatus,
        trap_info.mepc
    );
    log::trace!(
        "  mtval:   0x{:<16x} exits: {}  {:?}-mode",
        ctx.trap_info.mtval,
        ctx.nb_exits,
        ctx.mode
    );
    log::trace!(
        "  x1  {:<16x}  x2  {:<16x}  x3  {:<16x}",
        ctx.get(Register::X1),
        ctx.get(Register::X2),
        ctx.get(Register::X3)
    );
    log::trace!(
        "  x4  {:<16x}  x5  {:<16x}  x6  {:<16x}",
        ctx.get(Register::X4),
        ctx.get(Register::X5),
        ctx.get(Register::X6)
    );
    log::trace!(
        "  x7  {:<16x}  x8  {:<16x}  x9  {:<16x}",
        ctx.get(Register::X7),
        ctx.get(Register::X8),
        ctx.get(Register::X9)
    );
    log::trace!(
        "  x10 {:<16x}  x11 {:<16x}  x12 {:<16x}",
        ctx.get(Register::X10),
        ctx.get(Register::X11),
        ctx.get(Register::X12)
    );
    log::trace!(
        "  x13 {:<16x}  x14 {:<16x}  x15 {:<16x}",
        ctx.get(Register::X13),
        ctx.get(Register::X14),
        ctx.get(Register::X15)
    );
    log::trace!(
        "  x16 {:<16x}  x17 {:<16x}  x18 {:<16x}",
        ctx.get(Register::X16),
        ctx.get(Register::X17),
        ctx.get(Register::X18)
    );
    log::trace!(
        "  x19 {:<16x}  x20 {:<16x}  x21 {:<16x}",
        ctx.get(Register::X19),
        ctx.get(Register::X20),
        ctx.get(Register::X21)
    );
    log::trace!(
        "  x22 {:<16x}  x23 {:<16x}  x24 {:<16x}",
        ctx.get(Register::X22),
        ctx.get(Register::X23),
        ctx.get(Register::X24)
    );
    log::trace!(
        "  x25 {:<16x}  x26 {:<16x}  x27 {:<16x}",
        ctx.get(Register::X25),
        ctx.get(Register::X26),
        ctx.get(Register::X27)
    );
    log::trace!(
        "  x28 {:<16x}  x29 {:<16x}  x30 {:<16x}",
        ctx.get(Register::X28),
        ctx.get(Register::X29),
        ctx.get(Register::X30)
    );
    log::trace!(
        "  x31 {:<16x}  mie {:<16x}  mip {:<16x}",
        ctx.get(Register::X31),
        ctx.get(Csr::Mie),
        ctx.get(Csr::Mip)
    );
}

// ————————————————————————————————— Tests —————————————————————————————————— //

/// We test some properties after handling a trap from firmware.
/// We simulate a trap by creating a dummy trap state for the context of the machine.
///
/// Mideleg must be 0: don't allow nested interrupts when running Miralis.
/// ctx.pc must be set to the handler start address.
/// Mie, vMie, vMideleg must not change.
/// vMepc and vMstatus.MIE must be set to corresponding values in ctx.trap_info.
/// vMip must be updated to the value of Mip.
/// In case of an interrupt, Mip must be cleared: avoid Miralis to trap again.
#[cfg(test)]
mod tests {

    use crate::arch::{mstatus, Arch, Architecture, Csr, MCause, Mode};
    use crate::handle_trap;
    use crate::host::MiralisContext;
    use crate::policy::{Policy, PolicyModule};
    use crate::virt::VirtContext;

    #[test]
    fn handle_trap_state() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw);
        let mut policy = Policy::init();
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        // Firmware is running
        ctx.mode = Mode::M;

        ctx.csr.mstatus = 0;
        ctx.csr.mie = 0b1;
        ctx.csr.mideleg = 0;
        ctx.csr.mtvec = 0x80200024; // Dummy mtvec

        // Simulating a trap
        ctx.trap_info.mepc = 0x80200042; // Dummy address
        ctx.trap_info.mstatus = 0b1000;
        ctx.trap_info.mcause = MCause::Breakpoint as usize; // TODO : use a real int.
        ctx.trap_info.mip = 0b1;
        ctx.trap_info.mtval = 0;

        unsafe {
            Arch::write_csr(Csr::Mie, 0b1);
            Arch::write_csr(Csr::Mip, 0b1);
            Arch::write_csr(Csr::Mideleg, 0);
        };
        handle_trap(&mut ctx, &mut mctx, &mut policy);

        assert_eq!(Arch::read_csr(Csr::Mideleg), 0, "mideleg must be 0");
        assert_eq!(Arch::read_csr(Csr::Mie), 0b1, "mie must be 1");
        // assert_eq!(Arch::read_csr(Csr::Mip), 0, "mip must be 0"); // TODO : uncomment if using a real int.
        assert_eq!(ctx.pc, 0x80200024, "pc must be at handler start");
        assert_eq!(ctx.csr.mip, 0b1, "mip must to be updated");
        assert_eq!(ctx.csr.mie, 1, "mie must not change");
        assert_eq!(ctx.csr.mideleg, 0, "mideleg must not change");
        assert_eq!(ctx.csr.mepc, 0x80200042);
        assert_eq!(
            (ctx.csr.mstatus & mstatus::MIE_FILTER) >> mstatus::MIE_OFFSET,
            0b1,
            "mstatus.MIE must be set to trap_info.mstatus.MIE"
        );
    }
}
