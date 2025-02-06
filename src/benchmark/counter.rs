use core::sync::atomic::{AtomicU64, Ordering};

use crate::arch::{Arch, Architecture, Csr, Register};
use crate::benchmark::default::IntervalCounter;
use crate::benchmark::Counter::{FirmwareExits, WorldSwitches};
use crate::benchmark::{BenchmarkModule, Counter, Scope};
use crate::config::PLATFORM_NB_HARTS;
use crate::virt::traits::*;
use crate::virt::VirtContext;

// We use this structure to avoid false sharing in the benchmark.
// The typical size of a cache line is 64 bytes
#[repr(C, align(64))]
#[derive(Debug)]
struct PaddedCounter {
    counter: AtomicU64,
    _padding: [u8; 56],
}

// NOTE: Clippy is triggering a warning here but it's fine as we use the const only for array
// initialization.
#[allow(clippy::declare_interior_mutable_const)]
const ZEROED_COUNTER: PaddedCounter = PaddedCounter {
    counter: AtomicU64::new(0),
    _padding: [0; 56],
};

static NB_WORLD_SWITCHES: [PaddedCounter; PLATFORM_NB_HARTS] = [ZEROED_COUNTER; PLATFORM_NB_HARTS];
static NB_FIRMWARE_EXIT: [PaddedCounter; PLATFORM_NB_HARTS] = [ZEROED_COUNTER; PLATFORM_NB_HARTS];

const SINGLE_CORE_BENCHMARK: usize = 0;
const ALL_CORES_BENCHMARK: usize = 1;

/// A simple and efficient benchmark module based on atomic counters.
///
/// This benchmark module explicitly avoid computing any advanced statistics (e.g. standard
/// deviation) to keep the code simple and efficient.
pub struct CounterBenchmark {}

impl BenchmarkModule for CounterBenchmark {
    fn init() -> Self {
        CounterBenchmark {}
    }

    fn name() -> &'static str {
        "Counter benchmark"
    }

    fn increment_counter(counter: Counter) {
        if counter == FirmwareExits {
            NB_FIRMWARE_EXIT[hard_id()]
                .counter
                .fetch_add(1, Ordering::Relaxed);
        } else if counter == WorldSwitches {
            NB_WORLD_SWITCHES[hard_id()]
                .counter
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn update_inteval_counter_stats(
        &mut self,
        _counter: &IntervalCounter,
        _scope: &Scope,
        _value: usize,
    ) {
    }

    fn read_counters(ctx: &mut VirtContext) {
        let mut nb_firmware_exits: usize = 0;
        let mut nb_world_switch: usize = 0;

        match ctx.get(Register::X10) {
            SINGLE_CORE_BENCHMARK => {
                let hart = hard_id();
                nb_firmware_exits = get_nb_firmware_exits(hart) as usize;
                nb_world_switch = get_nb_world_switch(hart) as usize;
            }
            ALL_CORES_BENCHMARK => {
                for current_hart in 0..PLATFORM_NB_HARTS {
                    nb_firmware_exits += get_nb_firmware_exits(current_hart) as usize;
                    nb_world_switch += get_nb_world_switch(current_hart) as usize;
                }
            }
            _ => log::error!(
                "Invalid argument for register a0 [0 ==> Core 0 | 1 ==> All cores] {}",
                ctx.get(Register::X10)
            ),
        }

        ctx.set(Register::X10, nb_firmware_exits);
        ctx.set(Register::X11, nb_world_switch);
    }

    fn display_counters() {
        let current = hard_id();
        log::info!(
            "Core {}: {} firmware exits, {} world switches",
            current,
            get_nb_firmware_exits(current),
            get_nb_world_switch(current)
        )
    }

    fn get_counter_value(hart_id: usize, counter: Counter) -> usize {
        match counter {
            Counter::TotalExits => 0,
            FirmwareExits => get_nb_firmware_exits(hart_id) as usize,
            WorldSwitches => get_nb_world_switch(hart_id) as usize,
        }
    }
}

// ———————————————————————————————— Helpers ————————————————————————————————— //

/// Return the current hart id
fn hard_id() -> usize {
    Arch::read_csr(Csr::Mhartid)
}

/// Return the number of firmware exits on the given hart
fn get_nb_firmware_exits(hart: usize) -> u64 {
    NB_FIRMWARE_EXIT[hart].counter.load(Ordering::Relaxed)
}

/// Return the number of world switches on the given hart
fn get_nb_world_switch(hart: usize) -> u64 {
    NB_WORLD_SWITCHES[hart].counter.load(Ordering::Relaxed)
}
