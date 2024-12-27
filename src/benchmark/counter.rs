use core::sync::atomic::{AtomicU64, Ordering};

use crate::arch::{Arch, Architecture, Csr};
use crate::benchmark::default::IntervalCounter;
use crate::benchmark::Counter::{FirmwareExits, WorldSwitches};
use crate::benchmark::{BenchmarkModule, Counter, Scope};
use crate::config::PLATFORM_NB_HARTS;

// We use this structure to avoid false sharing in the benchmark.
// The typical size of a cache line is 64 bits
#[repr(C, align(64))]
#[derive(Debug)]
struct PaddedCounter {
    counter: AtomicU64,
    _padding: [u8; 56],
}

static mut NB_WORLD_SWITCHES: [PaddedCounter; PLATFORM_NB_HARTS] = [PaddedCounter {
    counter: AtomicU64::new(0),
    _padding: [0; 56],
}; PLATFORM_NB_HARTS];

static mut NB_FIRMWARE_EXIT: [PaddedCounter; PLATFORM_NB_HARTS] = [PaddedCounter {
    counter: AtomicU64::new(0),
    _padding: [0; 56],
}; PLATFORM_NB_HARTS];
pub struct CounterBenchmark {}

impl BenchmarkModule for CounterBenchmark {
    fn init() -> Self {
        CounterBenchmark {}
    }

    fn name() -> &'static str {
        "Counter benchmark"
    }

    fn start_interval_counters(_scope: Scope) {}

    fn stop_interval_counters(_scope: Scope) {}

    fn increment_counter(counter: Counter) {
        if counter == FirmwareExits {
            unsafe {
                NB_FIRMWARE_EXIT[Arch::read_csr(Csr::Mhartid)]
                    .counter
                    .fetch_add(1, Ordering::Relaxed);
            }
        } else if counter == WorldSwitches {
            unsafe {
                NB_WORLD_SWITCHES[Arch::read_csr(Csr::Mhartid)]
                    .counter
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn update_inteval_counter_stats(
        &mut self,
        _counter: &IntervalCounter,
        _scope: &Scope,
        _value: usize,
    ) {
    }

    fn display_counters() {
        todo!("implement the logic")
    }

    fn get_counter_value(core_id: usize, counter: Counter) -> usize {
        unsafe {
            match counter {
                Counter::TotalExits => 0,
                FirmwareExits => NB_FIRMWARE_EXIT[core_id].counter.load(Ordering::Relaxed) as usize,
                WorldSwitches => {
                    NB_WORLD_SWITCHES[core_id].counter.load(Ordering::Relaxed) as usize
                }
            }
        }
    }
}
