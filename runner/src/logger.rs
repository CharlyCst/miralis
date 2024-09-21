use std::sync::Mutex;

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

/// The runner logger
pub struct RunnerLogger {
    state: Mutex<LoggerState>,
}

/// The inner state of the logger
struct LoggerState {
    level: LevelFilter,
}

/// The global logger
///
/// We use a static here and lock mutable state, this allows sharing the logger between multiple
/// threads as needed.
static LOGGER: RunnerLogger = RunnerLogger {
    state: Mutex::new(LoggerState {
        level: LevelFilter::Info,
    }),
};

impl RunnerLogger {
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        // We first set the global log level, and install the the static logger to be used by all
        // the threads.
        LOGGER.state.lock().unwrap().level = level;
        log::set_logger(&LOGGER).expect("Failed to set logger");
        log::set_max_level(level);
        Ok(())
    }
}

impl log::Log for RunnerLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.state.lock().unwrap().level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => {
                    eprintln!("\x1b[31m{}\x1b[0m", record.args());
                }
                Level::Warn => {
                    println!("\x1b[33m{}\x1b[0m", record.args());
                }
                Level::Info => {
                    println!("{}", record.args());
                }
                _ => {}
            }
        }
    }

    fn flush(&self) {}
}
