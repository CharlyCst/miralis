//! Structured logging implementation

use core::sync::atomic::{AtomicBool, Ordering};

use log::{Level, LevelFilter, Metadata, Record};

use crate::config;
use crate::platform::{Plat, Platform};
use crate::utils::const_str_eq;

// ————————————————————————————————— Logger ————————————————————————————————— //

pub struct Logger {}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        enabled(metadata.target(), metadata.level())
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Writes the log
            if Plat::name() == "Miralis" {
                // No need for formatting, the host Miralis will handle it
                Plat::debug_print(record.level(), format_args!("{}", record.args()))
            } else {
                // Otherwise we format the logs properly
                Plat::debug_print(
                    record.level(),
                    format_args!(
                        "[{} | {}] {}\n",
                        level_display(record.level()),
                        record.target(),
                        record.args()
                    ),
                )
            }
        }
    }

    fn flush(&self) {}
}
pub fn init() {
    static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

    match IS_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => {
            log::set_logger(&Logger {}).unwrap();
            log::set_max_level(LevelFilter::Trace);
        }
        Err(_) => {
            log::warn!("Logger is already initialized, skipping init");
        }
    };
}

// —————————————————————————— Const Log Filtering ——————————————————————————— //
// We want to enable the filtering of logs at compile time on the critical
// path.
//
// We could achieve a similar outcome with the `max_level_*` features from the
// log crate, however it doesn't work well with incremental compilation: cargo
// will not re-compile a if a version with a superset of features is already
// compiled and can be re-used.
//
// Therefore we implement some `const` helpers to filter out the logs on the
// critical path.
// —————————————————————————————————————————————————————————————————————————— //

/// The global log level, defined at compile time by the configuration.
///
/// Higher log level can still be enabled on a per-module basis.
const GLOBAL_LOG_LEVEL: LevelFilter = match config::LOG_LEVEL {
    Some(s) => match s.as_bytes() {
        b"trace" => LevelFilter::Trace,
        b"debug" => LevelFilter::Debug,
        b"info" => LevelFilter::Info,
        b"warn" => LevelFilter::Warn,
        b"error" => LevelFilter::Error,
        b"off" => LevelFilter::Off,
        _ => LevelFilter::Info,
    },
    _ => LevelFilter::Info,
};

/// Returns true if the list of module names contains the target
const fn contains_target<const N: usize>(log_modules: &[&str; N], target: &str) -> bool {
    // Here we use a while loop because for loops are not yet stable in const contexts
    let mut i = 0;
    while i < log_modules.len() {
        if const_str_eq(log_modules[i], target) {
            return true;
        }
        i += 1
    }

    false
}

/// Returns true if the level is less verbose (or as much as) the level filter.
///
/// For instance:
/// ```
/// assert!(is_less_verbose(Level::Info, LevelFilter::Debug));
/// assert!(is_less_verbose(Level::Info, LevelFilter::Info));
/// assert!(!is_less_verbose(Level::Debug, LevelFilter::Info));
/// ```
const fn is_less_verbose(level: Level, filter: LevelFilter) -> bool {
    level as usize <= filter as usize
}

/// Returns true if the target module is enabled for the current log level.
const fn is_target_enabled(target: &str, level: Level) -> bool {
    let mut specific_module_enabled: bool = false;

    specific_module_enabled |=
        is_less_verbose(level, LevelFilter::Trace) && contains_target(config::LOG_TRACE, target);
    specific_module_enabled |=
        is_less_verbose(level, LevelFilter::Debug) && contains_target(config::LOG_DEBUG, target);
    specific_module_enabled |=
        is_less_verbose(level, LevelFilter::Info) && contains_target(config::LOG_INFO, target);
    specific_module_enabled |=
        is_less_verbose(level, LevelFilter::Warn) && contains_target(config::LOG_WARN, target);
    specific_module_enabled |=
        is_less_verbose(level, LevelFilter::Error) && contains_target(config::LOG_ERROR, target);

    specific_module_enabled
}

/// Returns true if the current level is enabled by the global configuration.
const fn is_globaly_enabled(level: Level) -> bool {
    GLOBAL_LOG_LEVEL as usize >= level as usize
}

/// Returns true if the target is enabled for the provided log level.
pub const fn enabled(target: &str, level: Level) -> bool {
    is_globaly_enabled(level) || is_target_enabled(target, level)
}

/// Returns true if the Trace logging level is enabled for the current module.
///
/// This macro is guaranteed to be executed at compile time, thus dead-code elimination is
/// guaranteed by any reasonable compiler.
macro_rules! trace_enabled {
    () => {
        // We wrap the computation in a const block to guarantee execution at compile time
        const { crate::logger::enabled(core::module_path!(), log::Level::Trace) }
    };
}

pub(crate) use trace_enabled;

// ————————————————————————————————— Utils —————————————————————————————————— //

fn level_display(level: Level) -> &'static str {
    if config::LOG_COLOR {
        // We log with colors, using ANSI escape sequences
        match level {
            Level::Error => "\x1b[31;1mError\x1b[0m",
            Level::Warn => "\x1b[33;1mWarn\x1b[0m ",
            Level::Info => "\x1b[32;1mInfo\x1b[0m ",
            Level::Debug => "\x1b[34;1mDebug\x1b[0m",
            Level::Trace => "\x1b[35;1mTrace\x1b[0m",
        }
    } else {
        match level {
            Level::Error => "Error",
            Level::Warn => "Warn ",
            Level::Info => "Info ",
            Level::Debug => "Debug",
            Level::Trace => "Trace",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_list() {
        assert!(contains_target(&["test", "test"], "test"));
        assert!(!contains_target(&["test"], "test2"));
        assert!(contains_target(&[""], ""));
        assert!(!contains_target(&["test", "test"], "test-test"));
        assert!(contains_target(&["car", "train", "boat"], "car"));
        assert!(contains_target(&["car", "train", "boat"], "train"));
        assert!(contains_target(&["car", "train", "boat"], "boat"));
    }
}
