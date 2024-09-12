//! Miralis entry point
//!
//! The main function is called directly after platform specific minimal setup (such as
//! configuration of the stack).

// Mark the crate as no_std and no_main, but only when not running tests.
// We need both std and main to be able to run tests in user-space on the host architecture.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

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

use arch::pmp::pmpcfg;
use arch::{pmp, Arch, Architecture};
use benchmark::{Benchmark, Counter, Scope};
use config::PLATFORM_NAME;
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

pub(crate) extern "C" fn main(_hart_id: usize, device_tree_blob_addr: usize) -> ! {
    // For now we simply park all the harts other than the boot one

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
    log::debug!("DTS address: 0x{:x}", device_tree_blob_addr);

    log::info!("Preparing jump into firmware");
    let firmware_addr = Plat::load_firmware();
    log::debug!("Firmware loaded at: {:x}", firmware_addr);

    let nb_virt_pmp;
    let virtual_devices = Plat::create_virtual_devices();

    // Detect hardware capabilities
    // SAFETY: this must happen before hardware initialization
    let hw = unsafe { Arch::detect_hardware() };

    // Initialize Miralis's own context
    let mut mctx = MiralisContext::new(hw);

    // Configure PMP registers, if available
    if mctx.pmp.nb_pmp >= 8 {
        // List of devices to protect (the first one is Miralis itself)
        let devices = [
            Plat::get_miralis_memory_start_and_size(),
            (virtual_devices[0].start_addr, virtual_devices[0].size),
            (virtual_devices[1].start_addr, virtual_devices[1].size),
            // Add more here as needed
        ];
        // Protect memory to trap firmware read/writes
        for (i, &(start, size)) in devices.iter().enumerate() {
            mctx.pmp
                .set(i, pmp::build_napot(start, size).unwrap(), pmpcfg::NAPOT);
        }

        // Add an inactive 0 entry so that the next PMP sees 0 with TOR configuration
        let inactive_index = devices.len();
        mctx.pmp.set(inactive_index, 0, pmpcfg::INACTIVE);

        // Finally, set the last PMP to grant access to the whole memory
        mctx.pmp.set(
            (mctx.pmp.nb_pmp - 1) as usize,
            usize::MAX,
            pmpcfg::RWX | pmpcfg::NAPOT,
        );

        // Set the offset of virtual PMPs (right after the inactive 0 entry)
        mctx.virt_pmp_offset = (inactive_index + 1) as u8;

        // Compute the number of virtual PMPs available
        let remaining_pmp_entries = mctx.pmp.nb_pmp as usize - devices.len() - 2;

        if let Some(max_virt_pmp) = config::VCPU_MAX_PMP {
            nb_virt_pmp = core::cmp::min(remaining_pmp_entries, max_virt_pmp);
        } else {
            nb_virt_pmp = remaining_pmp_entries;
        }
    } else {
        nb_virt_pmp = 0;
    }

    // Initialize the virtual context and configure architecture
    let mut ctx = VirtContext::new(hart_id, nb_virt_pmp);
    unsafe {
        // Set return address, mode and PMP permissions
        Arch::set_mpp(arch::Mode::U);
        // Update the PMPs prior to first entry
        Arch::write_pmp(&mctx.pmp);
        Arch::sfence_vma(None, None);

        // Configure the firmware context
        ctx.set(Register::X10, hart_id);
        ctx.set(Register::X11, device_tree_blob_addr);
        ctx.set_csr(
            Csr::Misa,
            Arch::read_csr(Csr::Misa) & !misa::DISABLED,
            &mctx.hw,
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

    main_loop(&mut ctx, &mut mctx);
}

fn main_loop(ctx: &mut VirtContext, mctx: &mut MiralisContext) -> ! {
    loop {
        Benchmark::start_interval_counters(Scope::RunVCPU);

        unsafe {
            Arch::run_vcpu(ctx);
        }

        Benchmark::stop_interval_counters(Scope::RunVCPU);
        Benchmark::start_interval_counters(Scope::HandleTrap);

        handle_trap(ctx, mctx);

        Benchmark::stop_interval_counters(Scope::HandleTrap);
        Benchmark::increment_counter(Counter::TotalExits);
    }
}

fn handle_trap(ctx: &mut VirtContext, mctx: &mut MiralisContext) {
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
        ExecutionMode::Firmware => ctx.handle_firmware_trap(&mctx),
        ExecutionMode::Payload => ctx.handle_payload_trap(),
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
        }
        (ExecutionMode::Payload, ExecutionMode::Firmware) => {
            log::debug!("Execution mode: Payload -> Firmware");
            unsafe { ctx.switch_from_payload_to_firmware(mctx) };
        }
        _ => {} // No execution mode transition
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
    use crate::virt::VirtContext;

    #[test]
    fn handle_trap_state() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp);

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
        handle_trap(&mut ctx, &mut mctx);

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
