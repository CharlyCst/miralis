//! Structured logging implementation

use log::{Level, LevelFilter, Metadata, Record};
use spin::Once;

use crate::config;
use crate::platform::{Plat, Platform};

// ————————————————————————————————— Logger ————————————————————————————————— //

pub struct Logger {}

impl Logger {
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

    fn contains_target<const N: usize>(log_modules: &[&str; N], target: &str) -> bool {
        for element in log_modules.iter() {
            if *element == target {
                return true;
            }
        }

        false
    }

    fn filter_by_module(&self, metadata: &Metadata) -> bool {
        let mut specific_module_enabled: bool = false;

        specific_module_enabled |= metadata.level() <= LevelFilter::Trace
            && Self::contains_target(config::LOG_TRACE, metadata.target());
        specific_module_enabled |= metadata.level() <= LevelFilter::Debug
            && Self::contains_target(config::LOG_DEBUG, metadata.target());
        specific_module_enabled |= metadata.level() <= LevelFilter::Info
            && Self::contains_target(config::LOG_INFO, metadata.target());
        specific_module_enabled |= metadata.level() <= LevelFilter::Warn
            && Self::contains_target(config::LOG_WARN, metadata.target());
        specific_module_enabled |= metadata.level() <= LevelFilter::Error
            && Self::contains_target(config::LOG_ERROR, metadata.target());

        specific_module_enabled
    }

    fn filter_by_global_level(&self, metadata: &Metadata) -> bool {
        Self::GLOBAL_LOG_LEVEL >= metadata.level()
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // We filter depending on the current log level.
        self.filter_by_global_level(metadata) || self.filter_by_module(metadata)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Writes the log
            if Plat::name() == "Miralis" {
                // No need for formatting, the host Miralis will handle it
                Plat::debug_print(record.level(), format_args!("{}", record.args()))
            } else {
                // Otherwise we format the logs proprely
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

static LOGGER_INIT: Once = Once::new();

pub fn init() {
    LOGGER_INIT.call_once(|| {
        if let Err(err) = log::set_logger(&Logger {}) {
            panic!("Failed to set logger: {}", err);
        }
        log::set_max_level(LevelFilter::Trace);
    });
}

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
    use crate::logger::Logger;

    #[test]
    fn test_in_list() {
        assert!(Logger::contains_target(&["test", "test"], "test"));
        assert!(!Logger::contains_target(&["test"], "test2"));
        assert!(Logger::contains_target(&[""], ""));
        assert!(!Logger::contains_target(&["test", "test"], "test-test"));
        assert!(Logger::contains_target(&["car", "train", "boat"], "car"));
        assert!(Logger::contains_target(&["car", "train", "boat"], "train"));
        assert!(Logger::contains_target(&["car", "train", "boat"], "boat"));
    }
}
