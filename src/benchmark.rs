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
        Plat::debug_print(core::format_args!($($arg)*))
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
#[derive(PartialEq, Eq)]
pub enum Counter {
    TotalExits = 0,
    FirmwareExits = 1,
    WorldSwitches = 2,
}

const NB_INTERVAL_COUNTER: usize = 2;

/// Benchmark interval counters.
/// This kind of counter aims to measure difference beetween two events.
#[derive(PartialEq, Eq)]
pub enum IntervalCounter {
    ExecutionTime = 0,
    InstructionRet = 1,
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
            Either::Counter(_) => "counters",
            Either::IntervalCounter(c) => match c {
                IntervalCounter::ExecutionTime => "time",
                IntervalCounter::InstructionRet => "instruction ret",
            },
        }
    }
}

pub struct Benchmark {
    // Temporary value to store previous state (e.g. state when the benchmark started to compare).
    interval_counters: [usize; NB_INTERVAL_COUNTER],

    // Counters that could be incremented and reset to 0.
    counters: [usize; NB_COUNTER],
}

impl Benchmark {
    pub const fn new() -> Benchmark {
        Benchmark {
            interval_counters: [0; NB_INTERVAL_COUNTER],
            counters: [0; NB_COUNTER],
        }
    }

    /// Reset counter value to default and return previous one.
    fn reset(&mut self, counter: Either) -> usize {
        let value = counter.reset_value();
        self.set(counter, value)
    }

    /// Increment counter by one. Mainly useful for occurence counters.
    fn increment(&mut self, counter: Either) {
        match counter {
            Either::Counter(c) => self.counters[c as usize] += 1,
            Either::IntervalCounter(c) => self.interval_counters[c as usize] += 1,
        }
    }

    /// Set value of a counter and return previous value.
    fn set(&mut self, counter: Either, value: usize) -> usize {
        match counter {
            Either::Counter(c) => {
                let index = c as usize;
                let previous = self.counters[index];
                self.counters[index] = value;
                previous
            }
            Either::IntervalCounter(c) => {
                let index = c as usize;
                let previous = self.interval_counters[index];
                self.interval_counters[index] = value;
                previous
            }
        }
    }

    /// Read value of a counter.
    fn read(&self, counter: Either) -> usize {
        match counter {
            Either::Counter(c) => self.counters[c as usize],
            Either::IntervalCounter(c) => self.interval_counters[c as usize],
        }
    }

    /// Reset interval counters.
    pub fn start_interval_counters() {
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
                return;
            }

            BENCH.lock().reset(counter);
        }
    }

    /// Stop and record interval counter.
    pub fn stop_interval_counters(tag: &str) {
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
                return;
            }

            let bench = counter.name();
            let value = counter.reset_value() - BENCH.lock().reset(counter);

            Self::record_entry(bench, tag, value);
        }
    }

    /// Increment counter's value.
    pub fn increment_counter(counter: Counter) {
        if !config::BENCHMARK {
            return;
        }

        let wrapped_counter = Either::Counter(counter);

        if !wrapped_counter.is_enabled() {
            return;
        }

        BENCH.lock().increment(wrapped_counter)
    }

    #[allow(dead_code)] // TODO: remove this when eventually used
    pub fn reset_counter(counter: Counter) {
        if !config::BENCHMARK {
            return;
        }

        BENCH.lock().reset(Either::Counter(counter));
    }

    /// Log counter value. Tag allows to link different recordings together for analysis.
    #[allow(dead_code)]
    pub fn record_counter(counter: Counter, tag: &str) {
        if !config::BENCHMARK {
            return;
        }
        let wrapped_counter = Either::Counter(counter);

        if !wrapped_counter.is_enabled() {
            return;
        }
        Benchmark::record_entry(
            wrapped_counter.name(),
            tag,
            BENCH.lock().read(wrapped_counter),
        )
    }

    /// Print formated string with value of the entry and a tag for identification.
    /// "benchmark" section allows to filter lines of benchmarking in the output.
    ///
    /// bench: what kind of benchmark it is
    /// (e.g. execution time, instruction count, number of exits...)
    ///
    /// tag: unique identifier of a measure that can be used to add more context
    /// (e.g. time when we measured, in which part of the code...)
    pub fn record_entry(bench: &str, tag: &str, entry: usize) {
        if !config::BENCHMARK {
            return;
        }
        benchmark_print!("benchmark || {:>15} || {:25} || {}", bench, tag, entry);
    }
}
