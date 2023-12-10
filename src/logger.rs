//! Structured logging implementation

use core::sync::atomic::{AtomicBool, Ordering};
use log::{LevelFilter, Metadata, Record};

use crate::platform::debug_print;

pub struct Logger {}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            debug_print(core::format_args!(
                "[{} | {}] {}\n",
                record.level(),
                record.target(),
                record.args()
            ))
        }
    }

    fn flush(&self) {}
}

pub fn init(level: LevelFilter) {
    static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
    static LOGGER: Logger = Logger {};

    match IS_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => {
            log::set_logger(&LOGGER).unwrap();
            log::set_max_level(level);
        }
        Err(_) => {
            log::warn!("Logger is already initialized, skipping init");
        }
    };
}
