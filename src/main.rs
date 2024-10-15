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

use core::fmt::Pointer;
use core::ptr;
use arch::{Arch, Architecture};
use benchmark::{Benchmark, Counter, Scope};
use config::PLATFORM_NAME;
use flattened_device_tree::error::FdtError;
use flattened_device_tree::FlattenedDeviceTree;
use platform::{init, Plat, Platform};
use policy::{Policy, PolicyModule};

use crate::arch::{misa, Csr, Register};
use crate::host::MiralisContext;
use crate::virt::traits::*;
use crate::virt::{ExecutionMode, VirtContext};

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


use fdt_rs::base::{DevTree, DevTreeNode, DevTreeProp};
use fdt_rs::prelude::{FallibleIterator, PropReader};
use crate::ace::core::architecture::control_status_registers::ReadWriteRiscvCsr;
use crate::ace::core::control_data::HardwareHart;
use crate::ace::non_confidential_flow::DeclassifyToHypervisor::SbiResponse;
use crate::ace::non_confidential_flow::NonConfidentialFlow;

fn read_unaligned_u64(ptr: *const u8) -> u64 {
    // Step 1: Create a temporary array to hold the bytes
    let mut buf = [0u8; 8]; // For u64, we need 8 bytes

    // Step 2: Read the bytes from the unaligned pointer
    unsafe {
        // Copy bytes from the pointer into the buffer
        for i in 0..8 {
            buf[i] = *ptr.add(i); // Use add(i) to get each byte
        }
    }

    // Step 3: Convert the byte array to u64
    u64::from_be_bytes(buf) // Convert to u64 (you can use from_be_bytes if needed)
}

fn write_unaligned_u64(ptr: *mut u8, value: u64) {
    // Step 1: Convert the u64 value to a byte array
    let bytes = value.to_be_bytes(); // You can use to_be_bytes() or to_ne_bytes() depending on endianness

    // Step 2: Write the bytes to the unaligned pointer
    unsafe {
        for i in 0..8 {
            *ptr.add(i) = bytes[i]; // Use add(i) to get each byte's address
        }
    }
}


fn divide_memory_region_size(device_tree_blob_addr:usize) -> Result<(), FdtError> {
    let fdt: FlattenedDeviceTree;
    unsafe {
        fdt = FlattenedDeviceTree::from_raw_pointer(device_tree_blob_addr as *const u8)?
    }

    let mem_prop = fdt.inner.props()
        .find(|p| Ok(p.name()? == "device_type" && p.str()? == "memory"))?
        .ok_or_else(|| FdtError::NoMemoryNode())?;

    let reg_prop = mem_prop.node()
        .props()
        .find(|p| Ok(p.name().unwrap_or("empty") == "reg"))?
        .ok_or_else(|| FdtError::NoMemoryNode())?;

    unsafe {
        let ptr: *const u8 = reg_prop.propbuf().as_ptr().add(8);

        let memory_size = read_unaligned_u64(ptr);
        write_unaligned_u64(ptr as *mut u8, memory_size / 2);
    }

    Ok(())
}

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
    let result = ace::core::initialization::init_security_monitor(device_tree_blob_addr as *const u8);
    match result {
        Ok(_) => log::info!("Operation succeeded."),
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

fn overwrite_hardware_hart_with_virtctx(hw: &mut HardwareHart, ctx: &mut VirtContext) {
    // Save normal registers
    for i in 0..32 {
        hw.hypervisor_hart.hypervisor_hart_state.gprs[i] = ctx.regs[i]
    }

    // Save CSR registers

    // M mode registers
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mepc = ReadWriteRiscvCsr(ctx.csr.mepc);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mcause = ReadWriteRiscvCsr(ctx.csr.mcause);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.medeleg = ReadWriteRiscvCsr(ctx.csr.medeleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mideleg = ReadWriteRiscvCsr(ctx.csr.mideleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mie = ReadWriteRiscvCsr(ctx.csr.mie);
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.mip = ReadWriteRiscvCsr(ctx.csr.mip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mstatus = ReadWriteRiscvCsr(ctx.csr.mstatus);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mtinst = ReadWriteRiscvCsr(ctx.csr.mtinst);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mtval = ReadWriteRiscvCsr(ctx.csr.mtval);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mtval2 = ReadWriteRiscvCsr(ctx.csr.mtval2);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mtvec = ReadWriteRiscvCsr(ctx.csr.mtvec);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mscratch = ReadWriteRiscvCsr(ctx.csr.mscratch);

    // S mode registers
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sstatus = ReadWriteRiscvCsr(ctx.csr.sstatus);
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sie = ReadWriteRiscvCsr(ctx.csr.sie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.stvec = ReadWriteRiscvCsr(ctx.csr.stvec);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.scounteren = ReadWriteRiscvCsr(ctx.csr.scounteren);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.senvcfg = ReadWriteRiscvCsr(ctx.csr.senvcfg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.sscratch = ReadWriteRiscvCsr(ctx.csr.sscratch);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.sepc = ReadWriteRiscvCsr(ctx.csr.sepc);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.scause = ReadWriteRiscvCsr(ctx.csr.scause);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.stval = ReadWriteRiscvCsr(ctx.csr.stval);
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sip = ReadWriteRiscvCsr(ctx.csr.sip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.satp = ReadWriteRiscvCsr(ctx.csr.satp);

    // HS mode registers
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hstatus = ReadWriteRiscvCsr(ctx.csr.hstatus);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hedeleg = ReadWriteRiscvCsr(ctx.csr.hedeleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hideleg = ReadWriteRiscvCsr(ctx.csr.hideleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hie = ReadWriteRiscvCsr(ctx.csr.hie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hcounteren = ReadWriteRiscvCsr(ctx.csr.hcounteren);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgeie = ReadWriteRiscvCsr(ctx.csr.hgeie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htval = ReadWriteRiscvCsr(ctx.csr.htval);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hip = ReadWriteRiscvCsr(ctx.csr.hip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hvip = ReadWriteRiscvCsr(ctx.csr.hvip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htinst = ReadWriteRiscvCsr(ctx.csr.htinst);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgeip = ReadWriteRiscvCsr(ctx.csr.hgeip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.henvcfg = ReadWriteRiscvCsr(ctx.csr.henvcfg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgatp = ReadWriteRiscvCsr(ctx.csr.hgatp);

    // HS mode debug registers
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.hcontext = ReadWriteRiscvCsr(ctx.csr.hcontext);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htimedelta = ReadWriteRiscvCsr(ctx.csr.htimedelta);

    // VS mode registers
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsstatus = ReadWriteRiscvCsr(ctx.csr.vsstatus);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsie = ReadWriteRiscvCsr(ctx.csr.vsie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsip = ReadWriteRiscvCsr(ctx.csr.vsip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vstvec = ReadWriteRiscvCsr(ctx.csr.vstvec);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsscratch = ReadWriteRiscvCsr(ctx.csr.vsscratch);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsepc = ReadWriteRiscvCsr(ctx.csr.vsepc);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vscause = ReadWriteRiscvCsr(ctx.csr.vscause);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vstval = ReadWriteRiscvCsr(ctx.csr.vstval);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsatp = ReadWriteRiscvCsr(ctx.csr.vsatp);
}

extern "C" {
    // Assembly function that is an entry point to the security monitor from the hypervisor or a virtual machine.
    fn enter_from_hypervisor_or_vm_asm() -> !;
}


/// This functions transfers the control to the ACE security monitor from Miralis
fn miralis_to_ace_ctx_switch(ace_ctx: &mut HardwareHart, virt_ctx: &mut VirtContext) {
    // todo!("Cool we are jumping here!");
    // Step 1: Overwrite Hardware hart with virtcontext
    overwrite_hardware_hart_with_virtctx(ace_ctx, virt_ctx);

    // Step 2: Change mscratch value
    // todo: this is incomplete
    ace_ctx.swap_mscratch();

    // Step 3: Change trap handler
    let trap_vector_address = enter_from_hypervisor_or_vm_asm as usize;
    ace_ctx.hypervisor_hart_mut().csrs_mut().mtvec.write((trap_vector_address >> 2) << 2);

    // Step 4: Jump to the payload
    let ace_flow = unsafe { NonConfidentialFlow::create(ace_ctx.as_mut().expect(NonConfidentialFlow::CTX_SWITCH_ERROR_MSG)) };
    ace_flow.apply_and_exit_to_hypervisor(SbiResponse);
}

fn ace_to_miralis_ctx_switch(virt_ctx: &mut VirtContext, ace_ctx: &mut HardwareHart) {
    todo!("Implement this function");
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
            log::debug!("Execution mode: Firmware -> Payload");
            unsafe { ctx.switch_from_firmware_to_payload(mctx) };
            policy.switch_from_firmware_to_payload(ctx, mctx);
        }
        (ExecutionMode::Payload, ExecutionMode::Firmware) => {
            log::debug!("Execution mode: Payload -> Firmware");
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
