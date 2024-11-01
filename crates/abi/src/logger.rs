//! Miralis SBI logger
//!
//! This is a logger implementation that uses the Miralis SBI to log messages.

use core::fmt::Write;

use log::{LevelFilter, Metadata, Record};
use spin::Once;

use crate::miralis_log;

// ————————————————————————————————— Logger ————————————————————————————————— //

pub struct Logger {}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // Log is always enabled for all levels, filtering is done by Miralis depending on its
        // configuration
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Write data into a stack-allocated buffer
            // For now we limit ourselves to 300 characters, that should be enough for printing the
            // panic error while not consuming too much stack space.
            let mut buff: StackBuffer<300> = StackBuffer::new();
            write!(&mut buff, "{}", record.args()).ok();

            miralis_log(record.level(), buff.as_str());
        }
    }

    fn flush(&self) {}
}

static LOGGER_INIT: Once = Once::new();

/// Initialize the firmware logger
///
/// This function is called automatically by `setup_binary!`.
pub fn init() {
    LOGGER_INIT.call_once(|| {
        if let Err(err) = log::set_logger(&Logger {}) {
            panic!("Failed to set logger: {}", err);
        }
        log::set_max_level(LevelFilter::Trace);
    });
}

// —————————————————————————————— Stack Buffer —————————————————————————————— //

/// A simple buffer than can be stask allocated and implement the Write trait.
pub(crate) struct StackBuffer<const N: usize> {
    buff: [u8; N],
    cursor: usize,
}

impl<const N: usize> StackBuffer<N> {
    pub const fn new() -> Self {
        StackBuffer {
            buff: [0u8; N],
            cursor: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        // NOTE: we only ever put valid strings in this buffer, so this will never panic
        core::str::from_utf8(&self.buff[..self.cursor]).unwrap()
    }
}

impl<const N: usize> core::fmt::Write for StackBuffer<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();

        // Check if the buffer has the required capacity
        // For now we just return an error if that is not the case, but we could also maybe just
        // silently drop the extra bytes.
        let n = bytes.len();
        if n > self.buff.len() - self.cursor {
            return Err(core::fmt::Error);
        }

        let new_cursor = self.cursor + n;
        self.buff[self.cursor..new_cursor].copy_from_slice(bytes);
        self.cursor = new_cursor;
        Ok(())
    }
}
