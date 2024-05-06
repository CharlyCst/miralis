//! Structured logging implementation

use core::sync::atomic::{AtomicBool, Ordering};

use log::{Level, LevelFilter, Metadata, Record};

use crate::config;
use crate::platform::{Plat, Platform};

// ————————————————————————————————— Logger ————————————————————————————————— //

pub struct Logger {
    log_level: LevelFilter,
    max_level: LevelFilter,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.log_level >= metadata.level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            Plat::debug_print(core::format_args!(
                "[{} | {}] {}\n",
                level_display(record.level()),
                record.target(),
                record.args()
            ))
        }
    }

    fn flush(&self) {}
}

impl Logger {
    const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

    const fn new_from_env() -> Self {
        let log_level = match config::LOG_LEVEL {
            Some(s) => match s.as_bytes() {
                b"trace" => LevelFilter::Trace,
                b"debug" => LevelFilter::Debug,
                b"info" => LevelFilter::Info,
                b"warn" => LevelFilter::Warn,
                b"error" => LevelFilter::Error,
                b"off" => LevelFilter::Off,
                _ => Self::DEFAULT_LOG_LEVEL,
            },
            _ => Self::DEFAULT_LOG_LEVEL,
        };

        Logger {
            log_level,
            max_level: log_level,
        }
    }
}

pub fn init() {
    static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
    static LOGGER: Logger = Logger::new_from_env();

    match IS_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => {
            log::set_logger(&LOGGER).unwrap();
            log::set_max_level(LOGGER.max_level);
        }
        Err(_) => {
            log::warn!("Logger is already initialized, skipping init");
        }
    };
}

// ————————————————————————————————— Utils —————————————————————————————————— //

fn level_display(level: Level) -> &'static str {
    match level {
        Level::Error => "Error",
        Level::Warn => "Warn ",
        Level::Info => "Info ",
        Level::Debug => "Debug",
        Level::Trace => "Trace",
    }
}
