use core::sync::atomic::{AtomicUsize, Ordering};

use crate::benchmark::default::IntervalCounter;
use crate::benchmark::Counter::{FirmwareExits, WorldSwitches};
use crate::benchmark::{BenchmarkModule, Counter, Scope};

pub struct CounterBenchmark {}

static NB_WORLD_SWITCHES: AtomicUsize = AtomicUsize::new(0);
static NB_FIRMWARE_EXITS: AtomicUsize = AtomicUsize::new(0);

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
            NB_FIRMWARE_EXITS.fetch_add(1, Ordering::Relaxed);
        } else if counter == WorldSwitches {
            NB_WORLD_SWITCHES.fetch_add(1, Ordering::Relaxed);
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

    fn get_counter_value(counter: Counter) -> usize {
        match counter {
            Counter::TotalExits => 0,
            FirmwareExits => NB_FIRMWARE_EXITS.load(Ordering::Relaxed),
            WorldSwitches => NB_WORLD_SWITCHES.load(Ordering::Relaxed),
        }
    }
}
