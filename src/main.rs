//! Miralis entry point
//!
//! The main function is called directly after platform specific minimal setup (such as
//! configuration of the stack).
//!
//! This file is only expected to compile to RISC-V, so it is allowed to use inline assembly.

// Mark the crate as no_std and no_main, but only when not running tests.
// We need both std and main to be able to run tests in user-space on the host architecture.
#![no_std]
#![no_main]

use core::arch::global_asm;

use miralis::arch::{misa, set_mpp, write_pmp, Arch, Architecture, Csr, Mode, Register};
use miralis::benchmark::{Benchmark, BenchmarkModule};
use miralis::config::{
    DELEGATE_PERF_COUNTER, PLATFORM_BOOT_HART_ID, PLATFORM_NAME, PLATFORM_NB_HARTS,
    TARGET_FIRMWARE_ADDRESS, TARGET_STACK_SIZE,
};
use miralis::host::MiralisContext;
use miralis::platform::{init, Plat, Platform};
use miralis::policy::{Policy, PolicyModule};
use miralis::virt::traits::*;
use miralis::virt::VirtContext;

// Memory layout, defined in the linker script.
extern "C" {
    static _stack_start: u8;
    static _bss_start: u8;
    static _bss_stop: u8;
    static _start_address: u8;
}

pub(crate) extern "C" fn main(_hart_id: usize, device_tree_blob_addr: usize) -> ! {
    // On the VisionFive2 board there is an issue with a hart_id
    // Identification, so we have to reassign it for now
    let hart_id = Arch::read_csr(Csr::Mhartid);

    init();
    log::info!("Hello, world!");
    log::info!("Platform name: {}", Plat::name());
    log::info!("Policy module: {}", Policy::name());
    log::info!("Benchmark module: {}", Benchmark::name());
    log::info!("Hart ID: {}", hart_id);
    log::debug!("misa:    0x{:x}", Arch::read_csr(Csr::Misa));
    log::debug!(
        "vmisa:   0x{:x}",
        Arch::read_csr(Csr::Misa) & !misa::DISABLED
    );
    log::debug!("mstatus: 0x{:x}", Arch::read_csr(Csr::Mstatus));
    log::debug!("DTS address: 0x{:x}", device_tree_blob_addr);

    log::info!(
        "Preparing jump into firmware : {:X}",
        TARGET_FIRMWARE_ADDRESS
    );
    let firmware_addr = Plat::load_firmware();
    log::debug!("Firmware loaded at: {:x}", firmware_addr);

    let mut policy: Policy = Policy::init();

    // Detect hardware capabilities
    // SAFETY: this must happen before hardware initialization
    let hw = unsafe { Arch::detect_hardware() };
    // Initialize Miralis's own context
    let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), get_miralis_size());

    // Initialize the virtual context and configure architecture
    let mut ctx = VirtContext::new(hart_id, mctx.pmp.nb_virt_pmp, mctx.hw.extensions.clone());
    unsafe {
        // Set return address, mode and PMP permissions
        set_mpp(Mode::U);
        // Update the PMPs prior to first entry
        write_pmp(&mctx.pmp).flush();

        // Configure the firmware context
        ctx.set(Register::X10, hart_id);
        ctx.set(Register::X11, device_tree_blob_addr);
        ctx.csr.misa = Arch::read_csr(Csr::Misa) & !misa::DISABLED;
        ctx.pc = firmware_addr;

        if DELEGATE_PERF_COUNTER {
            Arch::write_csr(Csr::Mcounteren, 0x1);
            Arch::write_csr(Csr::Scounteren, 0x1);
        }
    }

    // In case we compile Miralis as firmware, we stop execution at that point for the moment
    // This allows us to run Miralis on top as an integration test for the moment
    // In the future, we plan to run Miralis "as firmware" running a firmware
    if PLATFORM_NAME == "miralis" {
        log::info!("Successfully initialized Miralis as a firmware");
        Plat::exit_success();
    }

    // SAFETY: At this point we initialized the hardware, loaded the firmware, and configured the
    // initial register values.
    unsafe {
        miralis::main_loop(&mut ctx, &mut mctx, &mut policy);
    }

    // If we reach here it means the firmware exited successfully.
    unsafe {
        miralis::debug::log_stack_usage(&raw const _stack_start as usize);
    }
    Plat::exit_success();
}

/// Return the size of Miralis, including the stacks, rounded up the nearest power of two.
fn get_miralis_size() -> usize {
    let size = (&raw const _stack_start as usize)
        .checked_sub(&raw const _start_address as usize)
        .and_then(|diff| diff.checked_add(TARGET_STACK_SIZE * PLATFORM_NB_HARTS))
        .unwrap();

    size.next_power_of_two()
}

// ————————————————————————————— Panic Handler —————————————————————————————— //

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panicked at {:#?} ", info);
    unsafe { miralis::debug::log_stack_usage(&raw const _stack_start as usize) };
    Plat::exit_failure();
}

// —————————————————————————————— Entry Point ——————————————————————————————— //

// The global entry point
//
// This is the first assembly snippets that runs, it is responsible for setting up a suitable
// environment for the Rust code (stack, initial register content) and to handle the initial hart
// synchronisation.
global_asm!(
r#"
.attribute arch, "rv64imac"
.align 4
.text
.global _start
_start:
    // We start by setting up the stack:
    // First we find where the stack is for that hart
    ld t0, __stack_start
    li t1, {stack_size}  // Per-hart stack size
    csrr t2, mhartid     // Our current hart ID

    // compute how much space we need to put before this hart's stack
    add t3, x0, x0       // Initialize offset to zero
    add t4, x0, x0       // Initialize counter to zero
stack_start_loop:
    // First we exit the loop once we made enough iterations (N iterations for hart N)
    bgeu t4, t2, stack_start_done
    add t3, t3, t1       // Add space for one more stack
    addi t4, t4, 1       // Increment counter
    j stack_start_loop

stack_start_done:
    add t0, t0, t3       // The actual start of our stack
    add t1, t0, t1       // And the end of our stack

    // Then we fill the stack with a known memory pattern
    li t2, 0x0BADBED0
stack_fill_loop:
    // Exit when reaching the end address
    bgeu t0, t1, stack_fill_done
    sw t2, 0(t0)      // Write the pattern
    addi t0, t0, 4    // increment the cursor
    j stack_fill_loop
stack_fill_done:

    // Now we need to zero-out the BSS section
    // Only the boot hart set the BSS section to avoid race condition.

    csrr t0, mhartid         // Our current hart ID
    li t2, {boot_hart_id}    // Boot hart ID
    ld t3, __boot_bss_set    // Shared boolean, set to 1 to say to other harts that the BSS is not initialized yet
    bne t0, t2, wait_bss_end // Only the boot hart initializes the bss

    ld t4, __bss_start
    ld t5, __bss_stop
zero_bss_loop:
    bgeu t4, t5, zero_bss_done
    sd x0, 0(t4)
    addi t4, t4, 8
    j zero_bss_loop
zero_bss_done:

    // Say to other harts that the initialization is done.
    // This is atomic and memory is ordered in a way that the
    // initialization will then be visible as soon as the
    // boolean is reset. .aqrl means that it is done sequentially.
    amoswap.w.aqrl x0, x0, (t3)
    j end_wait

    // Wait until initialization is done
wait_bss_end:
    lw t2, (t3)
    bnez t2, wait_bss_end
end_wait:

    // And finally we load the stack pointer into sp and jump into main
    mv sp, t1
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
__boot_bss_set:
    .dword {boot_bss_set}
"#,
    main = sym main,
    stack_start = sym _stack_start,
    stack_size = const TARGET_STACK_SIZE,
    bss_start = sym _bss_start,
    bss_stop = sym _bss_stop,
    boot_hart_id = const PLATFORM_BOOT_HART_ID,
    boot_bss_set = sym BOOT_BSS_SET,
);

// Boolean to synchronized harts
static BOOT_BSS_SET: usize = 1;
