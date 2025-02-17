//! Tracing firmware payload
//!
//! This payload measure the cost of a context switch in two situations
//! Situation 1: VM-mode firmware <--> Miralis
//! Situation 2: S-mode payload <--> VM-mode firmware

#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use config_helpers::parse_str_or;
use miralis_abi::{failure, log, setup_binary, success};
use miralis_core::sbi_codes;

setup_binary!(main);

const POLICY_NAME: &str = parse_str_or(option_env!("MIRALIS_POLICY_NAME"), "default_policy");

const PROTECT_PAYLOAD_POLICY: &str = "protect_payload";
const OFFLOAD_POLICY: &str = "offload";

fn enable_mcycle_in_smode() {
    unsafe {
        // This allows to read cycle in S-mode - for the payload
        let mcounteren: u32;
        asm!("csrr {}, mcounteren", out(reg) mcounteren);
        asm!("csrw mcounteren, {}", in(reg) mcounteren | 1);
    }
}

fn main() -> ! {
    let trap: usize = _empty_handler as usize;

    enable_mcycle_in_smode();

    unsafe {
        asm!(
        "csrw mtvec, {mtvec}", // Write mtvec with trap handler
        mtvec = in(reg) trap,
        );
    }

    log::info!("Start benchmarking from Firmware");

    measure(true);

    log::info!("Start benchmarking from Payload");

    let os: usize = operating_system as usize;
    let mpp = 0b1 << 11; // MPP = S-mode

    unsafe {
        asm!(
        "li t4, 0xfffffffff",
        "csrw pmpcfg0, 0xf",   // XRW TOR
        "csrw pmpaddr0, t4",   // All memory
        "auipc t4, 0",
        "addi t4, t4, 24",
        "csrw mstatus, {mpp}", // Write MPP of mstatus to S-mode
        "csrw mepc, {os}",     // Write MEPC
        "mret",                // Jump to OS


        os = in(reg) os,
        mpp = in(reg) mpp,
        out("t4") _,
        );
    }

    failure();
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _empty_handler
_empty_handler:
    // Skip illegal instruction (pc += 4)
    csrrw x5, mepc, x5
    addi x5, x5, 4
    csrrw x5, mepc, x5
    // Return back to OS
    mret
"#,
);

extern "C" {
    fn _empty_handler();
}

// —————————————————————————————— Benchmark operating system —————————————————————————————— //

const NB_REPEATS: usize = 1000;

pub fn bubble_sort(arr: &mut [usize; NB_REPEATS]) {
    let len = arr.len();
    let mut swapped;

    for i in 0..len {
        swapped = false;

        for j in 0..len - 1 - i {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
                swapped = true;
            }
        }

        if !swapped {
            break;
        }
    }

    for i in 1..len {
        if arr[i - 1] > arr[i] {
            log::error!("Error in sorting, results aren't reliable");
            failure();
        }
    }
}

fn operating_system() {
    unsafe {
        asm!("la sp, 0x80700000");
    }

    measure(false);

    if POLICY_NAME == PROTECT_PAYLOAD_POLICY {
        measure_misaligned();
    }

    // These are the most two frequent type of traps we receive in Miralis from the payload
    if POLICY_NAME == OFFLOAD_POLICY {
        measure_time_ecall();
        measure_time_read();
    }

    success();
}

fn measure(is_firmware: bool) {
    let mut values: [usize; NB_REPEATS] = [0; NB_REPEATS];

    for i in 0..NB_REPEATS {
        values[i] = trigger_ctx_switch_to_firmware()
    }

    let stats = get_statistics(values);
    let average_measure = trigger_ctx_switch_to_firmware_batched();

    if is_firmware {
        log::info!("Firmware cost {} : {}", POLICY_NAME, average_measure);
    } else {
        log::info!("Payload cost {} : {}", POLICY_NAME, average_measure);
    }

    print_statistics(stats);
}

fn measure_misaligned() {
    let mut values: [usize; NB_REPEATS] = [0; NB_REPEATS];

    for i in 0..NB_REPEATS {
        values[i] = trigger_misaligned_op()
    }

    let stats = get_statistics(values);
    let average_measure = trigger_misaligned_op_batched();

    log::info!("Misaligned cost {} : {}", POLICY_NAME, average_measure);

    print_statistics(stats);
}

fn measure_time_ecall() {
    let mut values: [usize; NB_REPEATS] = [0; NB_REPEATS];

    for i in 0..NB_REPEATS {
        values[i] = trigger_ecall_op()
    }

    let stats = get_statistics(values);
    let average_measure = trigger_ecall_op_batched();

    log::info!(
        "Ecall cost to set time {} : {}",
        POLICY_NAME,
        average_measure
    );

    print_statistics(stats);
}

fn measure_time_read() {
    let mut values: [usize; NB_REPEATS] = [0; NB_REPEATS];

    for i in 0..NB_REPEATS {
        values[i] = trigger_time_read_op()
    }

    let stats = get_statistics(values);
    let average_measure = trigger_time_read_op_batched();

    log::info!(
        "CSRRS Cost to read time {} : {}",
        POLICY_NAME,
        average_measure
    );

    print_statistics(stats);
}

fn trigger_ctx_switch_to_firmware() -> usize {
    let begin: u64;
    let end: u64;

    unsafe {
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) begin);
        // We trigger an illegal instruction
        asm!("csrw mscratch, zero");
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) end);
    }

    (end - begin) as usize
}

fn trigger_ctx_switch_to_firmware_batched() -> usize {
    let begin: u64;
    let end: u64;

    unsafe {
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) begin);
        for _ in 0..NB_REPEATS {
            // We can trigger an illegal instruction
            asm!("csrw mscratch, zero");
        }

        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) end);
    }

    (end - begin) as usize / NB_REPEATS
}

pub fn trigger_misaligned_op() -> usize {
    let begin: u64;
    let end: u64;

    let misaligned_address_8_bytes: usize = 0x80600301;

    unsafe {
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) begin);
        // We trigger a misaligned operation
        asm!(
        "ld {r}, 0({addr})",
        addr = in(reg) misaligned_address_8_bytes,
        r = out(reg) _,
        );
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) end);
    }

    (end - begin) as usize
}

pub fn trigger_misaligned_op_batched() -> usize {
    let begin: u64;
    let end: u64;

    let misaligned_address_8_bytes: usize = 0x80600301;

    unsafe {
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) begin);

        for _ in 0..NB_REPEATS {
            // We trigger a misaligned operation
            asm!(
            "ld {r}, 0({addr})",
            addr = in(reg) misaligned_address_8_bytes,
            r = out(reg) _,
            );
        }
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) end);
    }

    (end - begin) as usize / NB_REPEATS
}

pub fn trigger_ecall_op() -> usize {
    let begin: u64;
    let end: u64;

    unsafe {
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) begin);
        // We trigger a misaligned operation
        asm!(
        "mv a0, {0}",
        "li a6, {1}",
        "li a7, {2}",
        "ecall",
        in(reg) usize::MAX,                  // a0
        const sbi_codes::SBI_TIMER_FID,      // a6 (Use `const` for small immediates)
        const sbi_codes::SBI_TIMER_EID,      // a7
        out("a0") _,                         // syscall may overwrite a0
        out("a6") _,                         // syscall may overwrite a6
        out("a7") _,                         // syscall may overwrite a7
        );

        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) end);
    }

    (end - begin) as usize
}

pub fn trigger_ecall_op_batched() -> usize {
    let begin: u64;
    let end: u64;

    unsafe {
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) begin);

        for _ in 0..NB_REPEATS {
            // We trigger a misaligned operation
            asm!(
            "mv a0, {0}",
            "li a6, {1}",
            "li a7, {2}",
            "ecall",
            in(reg) usize::MAX,                  // a0
            const sbi_codes::SBI_TIMER_FID,      // a6 (Use `const` for small immediates)
            const sbi_codes::SBI_TIMER_EID,      // a7
            out("a0") _,                         // syscall may overwrite a0
            out("a6") _,                         // syscall may overwrite a6
            out("a7") _,                         // syscall may overwrite a7
            );
        }
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) end);
    }

    (end - begin) as usize / NB_REPEATS
}

pub fn trigger_time_read_op() -> usize {
    let begin: u64;
    let end: u64;

    unsafe {
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) begin);
        // Read time to measure the offload latency
        asm!("csrrs x15, time, x0");
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) end);
    }

    (end - begin) as usize
}

pub fn trigger_time_read_op_batched() -> usize {
    let begin: u64;
    let end: u64;

    unsafe {
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) begin);

        for _ in 0..NB_REPEATS {
            // Read time to measure the offload latency
            asm!("csrrs x15, time, x0");
        }
        // Read the `mcycle` register (assuming 64-bit RISC-V)
        asm!("csrr {}, cycle", out(reg) end);
    }

    (end - begin) as usize / NB_REPEATS
}

#[derive(Debug)]
pub struct Statistics {
    mean: usize,
    min: usize,
    max: usize,

    p25: usize,
    p50: usize,
    p75: usize,
    p95: usize,
    p99: usize,
}

fn get_statistics(mut arr: [usize; NB_REPEATS]) -> Statistics {
    bubble_sort(&mut arr);

    let mut output: Statistics = Statistics {
        mean: 0,
        min: 0,
        max: 0,
        p25: 0,
        p50: 0,
        p75: 0,
        p95: 0,
        p99: 0,
    };

    output.min = arr[0];
    output.max = arr[arr.len() - 1];
    output.mean = arr.iter().sum::<usize>() / arr.len();

    let percentile = |per: f64| -> usize { arr[(per * arr.len() as f64) as usize] };

    output.p25 = percentile(0.25);
    output.p50 = percentile(0.50);
    output.p75 = percentile(0.75);
    output.p95 = percentile(0.95);
    output.p99 = percentile(0.99);

    output
}

fn print_statistics(stats: Statistics) {
    log::info!("{:?}", stats);
}
