//! Benchmark Modules
//!
//! This modules holds the different values for the benchmarks

mod default;
mod empty;

use config_select::select_env;

use crate::benchmark::default::IntervalCounter;

pub type Benchmark = select_env!["MIRALIS_BENCHMARK_TYPE":
    "default"      => default::DefaultBenchmark
    _ => empty::EmptyBenchmark
];

pub trait BenchmarkModule {
    fn init() -> Self;
    fn name() -> &'static str;

    fn start_interval_counters(scope: Scope);
    fn stop_interval_counters(scope: Scope);
    fn increment_counter(counter: Counter);

    fn update_inteval_counter_stats(
        &mut self,
        counter: &IntervalCounter,
        scope: &Scope,
        value: usize,
    );
    /// Print formated string with value of the counters
    fn display_counters();

    fn get_counter_value(counter: Counter) -> usize;
}

pub enum Scope {
    HandleTrap,
    RunVCPU,
}

impl Scope {
    fn base(&self) -> usize {
        match self {
            Self::HandleTrap => 0,
            Self::RunVCPU => 1,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::HandleTrap => "handle_trap",
            Self::RunVCPU => "run_vcpu",
        }
    }
}

/// Benchmark counters.
/// This kind of counter aims to be incremented to count occurences of an event.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Counter {
    TotalExits = 0,
    FirmwareExits = 1,
    WorldSwitches = 2,
}
