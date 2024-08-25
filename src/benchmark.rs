//! Benchmark features to implement different specific benchmarks.
//!
//! This is useful for creating different benchmark on time of execution or
//! the number of instruction for example.
use spin::Mutex;

use crate::arch::{Arch, Architecture, Csr};
use crate::config;
use crate::platform::{Plat, Platform};

#[macro_export]
macro_rules! _benchmark_print {
    ($($arg:tt)*) => {
        Plat::debug_print(log::Level::Info, core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! benchmark_print {
    () => (if config::BENCHMARK { $crate::_benchmark_print!("\r\n")});
    ($($arg:tt)*) => (if config::BENCHMARK { $crate::_benchmark_print!("{}\r\n", core::format_args!($($arg)*))})
}

pub static BENCH: Mutex<Benchmark> = Mutex::new(Benchmark::new());

const NB_COUNTER: usize = 3;

/// Benchmark counters.
/// This kind of counter aims to be incremented to count occurences of an event.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Counter {
    TotalExits = 0,
    FirmwareExits = 1,
    WorldSwitches = 2,
}

const NB_INTERVAL_COUNTER: usize = 2;

const NB_SCOPES: usize = 2;

/// Benchmark interval counters.
/// This kind of counter aims to measure difference beetween two events.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum IntervalCounter {
    ExecutionTime = 0,
    InstructionRet = 1,
}

#[derive(Copy, Clone)]
struct IntervalCounterStats {
    previous: usize,
    count: usize,
    min: usize,
    max: usize,
    mean: usize,
    sum: usize,
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

enum Either {
    IntervalCounter(IntervalCounter),
    Counter(Counter),
}

/// Either enum represent either classic counters or interval counters.
/// The purpose of this is to unify common functionnalities.
impl Either {
    /// Whether the config enabled the counter.
    fn is_enabled(&self) -> bool {
        match self {
            Either::Counter(c) => match c {
                Counter::TotalExits => config::BENCHMARK_NB_EXITS,
                Counter::FirmwareExits => config::BENCHMARK_NB_FIRMWARE_EXITS,
                Counter::WorldSwitches => config::BENCHMARK_WORLD_SWITCHES,
            },
            Either::IntervalCounter(c) => match c {
                IntervalCounter::ExecutionTime => config::BENCHMARK_TIME,
                IntervalCounter::InstructionRet => config::BENCHMARK_INSTRUCTION,
            },
        }
    }

    /// Default value of the counter: Usually zero for occurence counters and current
    /// value for interval counters.
    fn reset_value(&self) -> usize {
        match self {
            Either::Counter(_) => 0,
            Either::IntervalCounter(c) => match c {
                IntervalCounter::ExecutionTime => Plat::get_clint().lock().read_mtime(),
                IntervalCounter::InstructionRet => Arch::read_csr(Csr::Minstret),
            },
        }
    }

    /// The name of the counter. Name are intended to be used to regroup measures.
    fn name(&self) -> &'static str {
        match self {
            Either::Counter(c) => match c {
                Counter::TotalExits => "Total exits",
                Counter::FirmwareExits => "Firmware exits",
                Counter::WorldSwitches => "World Switches",
            },
            Either::IntervalCounter(c) => match c {
                IntervalCounter::ExecutionTime => " Execution time ",
                IntervalCounter::InstructionRet => " Instruction retired ",
            },
        }
    }
}

pub struct Benchmark {
    // Temporary value to store previous state (e.g. state when the benchmark started to compare).
    interval_counters: [IntervalCounterStats; NB_INTERVAL_COUNTER * NB_SCOPES],

    // Counters that could be incremented and reset to 0.
    counters: [usize; NB_COUNTER],
}

impl Benchmark {
    pub const fn new() -> Benchmark {
        Benchmark {
            interval_counters: [IntervalCounterStats {
                previous: 0,
                count: 0,
                min: usize::MAX,
                max: 0,
                mean: 0,
                sum: 0,
            }; NB_INTERVAL_COUNTER * 2],

            counters: [0; NB_COUNTER],
        }
    }

    /// Reset counter value to default and return previous one.
    fn reset(&mut self, counter: &Either, scope: &Scope) -> usize {
        let value = counter.reset_value();
        match counter {
            Either::Counter(c) => {
                let index = *c as usize;
                let previous = self.counters[index];
                self.counters[index] = value;
                previous
            }
            Either::IntervalCounter(c) => {
                let index = Self::interval_counter_index(c, scope);
                let previous = self.interval_counters[index].previous;
                self.interval_counters[index].previous = value;
                previous
            }
        }
    }

    fn interval_counter_index(counter: &IntervalCounter, scope: &Scope) -> usize {
        *counter as usize + scope.base() * NB_INTERVAL_COUNTER
    }

    /// Read value of a counter.
    fn read_interval_counters(&self, counter: &IntervalCounter, scope: &Scope) -> usize {
        let index = Self::interval_counter_index(counter, scope);
        self.interval_counters[index].previous
    }

    /// Reset interval counters.
    pub fn start_interval_counters(scope: Scope) {
        if !config::BENCHMARK {
            return;
        }

        for counter in [
            IntervalCounter::ExecutionTime,
            IntervalCounter::InstructionRet,
        ]
        .map(Either::IntervalCounter)
        {
            if !counter.is_enabled() {
                continue;
            }

            BENCH.lock().reset(&counter, &scope);
        }
    }

    /// Stop and record interval counter.
    pub fn stop_interval_counters(scope: Scope) {
        if !config::BENCHMARK {
            return;
        }

        for counter in [
            IntervalCounter::ExecutionTime,
            IntervalCounter::InstructionRet,
        ] {
            let wrapped_counter = Either::IntervalCounter(counter);

            if !wrapped_counter.is_enabled() {
                continue;
            }

            let mut bench = BENCH.lock();
            let value =
                wrapped_counter.reset_value() - bench.read_interval_counters(&counter, &scope);

            bench.update_inteval_counter_stats(&counter, &scope, value);
        }
    }

    fn update_inteval_counter_stats(
        &mut self,
        counter: &IntervalCounter,
        scope: &Scope,
        value: usize,
    ) {
        let index = Self::interval_counter_index(counter, scope);
        let stats = &mut self.interval_counters[index];
        stats.count += 1;
        stats.sum += value;
        stats.mean = stats.sum / stats.count;
        stats.min = core::cmp::min(value, stats.min);
        stats.max = core::cmp::max(value, stats.max);
    }

    /// Increment counter's value.
    pub fn increment_counter(counter: Counter) {
        if !config::BENCHMARK {
            return;
        }

        let index = counter as usize;

        let wrapped_counter = Either::Counter(counter);

        if !wrapped_counter.is_enabled() {
            return;
        }

        BENCH.lock().counters[index] += 1;
    }

    /// Print formated string with value of the counters
    pub fn record_counters() {
        if !config::BENCHMARK {
            return;
        }

        let bench = BENCH.lock();

        if config::BENCHMARK_CSV_FORMAT {
            benchmark_print!("START BENCHMARK\ncounter,min,max,sum,mean");
        } else {
            benchmark_print!("\nBenchmark results\n---");
        }

        // Regular counters
        for counter in [
            Counter::FirmwareExits,
            Counter::TotalExits,
            Counter::WorldSwitches,
        ] {
            let wrapped_counter = Either::Counter(counter);
            if !wrapped_counter.is_enabled() {
                continue;
            }
            let value = bench.counters[counter as usize];
            let name = wrapped_counter.name();
            if config::BENCHMARK_CSV_FORMAT {
                benchmark_print!("{},{},{},{},{}", name, value, value, value, value);
            } else {
                benchmark_print!("{:15}: {:>12}", name, value);
            }
        }

        // Interval counters
        for scope in [Scope::HandleTrap, Scope::RunVCPU] {
            if !config::BENCHMARK_CSV_FORMAT {
                benchmark_print!("╔{:─>30}╗", "");
                benchmark_print!("│{:^30}│", scope.name());
            }

            for counter in [
                IntervalCounter::ExecutionTime,
                IntervalCounter::InstructionRet,
            ] {
                let wrapped_counter = Either::IntervalCounter(counter);
                if !wrapped_counter.is_enabled() {
                    continue;
                }
                let index: usize = Self::interval_counter_index(&counter, &scope);
                let stats = bench.interval_counters[index];
                let name = wrapped_counter.name();
                if config::BENCHMARK_CSV_FORMAT {
                    benchmark_print!(
                        "{}::{},{},{},{},{}",
                        name.trim(),
                        scope.name(),
                        stats.min,
                        stats.max,
                        stats.sum,
                        stats.mean
                    );
                } else {
                    benchmark_print!("│╔{:─^28}╗│", name);
                    benchmark_print!("││  Min: {:>20} ││", stats.min);
                    benchmark_print!("││  Max: {:>20} ││", stats.max);
                    benchmark_print!("││  Sum: {:>20} ││", stats.sum);
                    benchmark_print!("││  Mean: {:>19} ││", stats.mean);
                    benchmark_print!("│╚{:─>28}╝│", "");
                }
            }
            if !config::BENCHMARK_CSV_FORMAT {
                benchmark_print!("╚{:─>30}╝", "");
            }
        }
    }
}
