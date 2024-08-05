//! Benchmark features to implement different specific benchmarks.
//!
//! This is useful for creating different benchmark on time of execution or
//! the number of instruction for example.

use core::arch::asm;

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

pub struct Benchmark {
    // Does the benchmark started.
    pub is_running: bool,

    // Temporary value to store previous state (e.g. state when the benchmark started to compare).
    pub previous_instr_count: usize,

    pub previous_timer: usize,
}

impl Benchmark {
    pub fn new() -> Benchmark {
        Benchmark {
            is_running: false,
            previous_instr_count: 0,
            previous_timer: 0,
        }
    }

    /// Start benchmarking.
    pub fn start(&mut self) {
        if self.is_running {
            log::error!("already counting instr");
        } else {
            self.is_running = true;

            // instruction
            if config::BENCHMARK_INSTRUCTION {
                self.previous_instr_count = Arch::read_csr(Csr::Minstret);
            }

            // time
            if config::BENCHMARK_TIME {
                unsafe {
                    asm!(
                        "rdtime {value}",
                        value = out(reg) self.previous_timer
                    );
                }
            }
        }
    }

    /// Stop benchmarking.
    pub fn stop(&mut self, tag: &str) {
        self.is_running = false;

        // instruction
        if config::BENCHMARK_INSTRUCTION {
            let stop_value = Arch::read_csr(Csr::Minstret);
            Self::record_entry("instr", tag, stop_value - self.previous_instr_count);
        }

        // time
        if config::BENCHMARK_TIME {
            let stop_time: usize;
            unsafe {
                asm!(
                    "rdtime {value}", 
                    value = out(reg) stop_time);
            }
            Self::record_entry("time", tag, stop_time - self.previous_timer);
        }
    }

    /// Print formated string with value of the entry and a tag for identification.
    pub fn record_entry(bench: &str, tag: &str, entry: usize) {
        benchmark_print!("benchmark || {:>15} || {:25} || {}", bench, tag, entry);
    }
}
