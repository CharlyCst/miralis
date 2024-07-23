//! Structured logging implementation

use core::sync::atomic::{AtomicBool, Ordering};

use log::{Level, LevelFilter, Metadata, Record};

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

        return false;
    }

    fn filter_by_module(&self, record: &Record) -> bool {
        let mut specific_module_enabled: bool = false;

        specific_module_enabled |= record.metadata().level() <= LevelFilter::Trace
            && Self::contains_target(config::LOG_TRACE, record.target());
        specific_module_enabled |= record.metadata().level() <= LevelFilter::Debug
            && Self::contains_target(config::LOG_DEBUG, record.target());
        specific_module_enabled |= record.metadata().level() <= LevelFilter::Info
            && Self::contains_target(config::LOG_INFO, record.target());
        specific_module_enabled |= record.metadata().level() <= LevelFilter::Warn
            && Self::contains_target(config::LOG_WARN, record.target());
        specific_module_enabled |= record.metadata().level() <= LevelFilter::Error
            && Self::contains_target(config::LOG_ERROR, record.target());

        specific_module_enabled
    }

    fn filter_by_global_level(&self, record: &Record) -> bool {
        Self::GLOBAL_LOG_LEVEL >= record.metadata().level()
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        // We set to true such that each logs arrives in the log function and then we filter
        true
    }

    fn log(&self, record: &Record) {
        let global_level_mask: bool = self.filter_by_global_level(record);
        let module_level_mask: bool = self.filter_by_module(record);

        if global_level_mask || module_level_mask {
            // Writes the log
            Plat::debug_print(format_args!(
                "[{} | {}] {}\n",
                level_display(record.level()),
                record.target(),
                record.args()
            ))
        }
    }

    fn flush(&self) {}
}
pub fn init() {
    static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
    // A temp crutch: for some reason w/o those two strings logger won't print (working on it)
    Plat::debug_print(format_args!("Log init status: {:?}", IS_INITIALIZED));
    let address: *const AtomicBool = &IS_INITIALIZED;

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
