//! Mirage SBI logger
//!
//! This is a logger implementation that uses the Mirage SBI to log messages.

use core::arch::asm;
use core::fmt::Write;

use log::{LevelFilter, Metadata, Record};
use mirage_core::abi;

// ————————————————————————————————— Logger ————————————————————————————————— //

pub struct Logger {}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // Log is always enabled for all levels, filtering is done by Mirage depending on its
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

            // Prepare ecall arguments
            let eid = abi::MIRAGE_EID;
            let fid = abi::MIRAGE_LOG_FID;
            let level = match record.level() {
                log::Level::Error => abi::log::MIRAGE_ERROR,
                log::Level::Warn => abi::log::MIRAGE_WARN,
                log::Level::Info => abi::log::MIRAGE_INFO,
                log::Level::Debug => abi::log::MIRAGE_DEBUG,
                log::Level::Trace => abi::log::MIRAGE_TRACE,
            };
            let addr = buff.buff.as_ptr() as usize;
            let len = buff.cursor;

            // The actual call
            unsafe {
                asm!(
                    "ecall",
                    inout("a0") level => _,
                    inout("a1") addr => _,
                    inout("a2") len => _,
                    inout("a6") fid => _,
                    inout("a7") eid => _,
                );
            }
        }
    }

    fn flush(&self) {}
}

/// Initialize the firmware logger
///
/// This function is called automatically bu `setup_firmware!`.
pub fn init() {
    static LOGGER: Logger = Logger {};

    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);
}

// —————————————————————————————— Stack Buffer —————————————————————————————— //

/// A simple buffer than can be stask allocated and implement the Write trait.
struct StackBuffer<const N: usize> {
    buff: [u8; N],
    cursor: usize,
}

impl<const N: usize> StackBuffer<N> {
    const fn new() -> Self {
        StackBuffer {
            buff: [0u8; N],
            cursor: 0,
        }
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
