//! Trap handling

use core::fmt;

use crate::arch::{Arch, Architecture};

// —————————————————————————————— Trap Handler —————————————————————————————— //

/// Trap handler entry point
#[no_mangle]
pub(crate) extern "C" fn trap_handler() {
    log::info!("Trapped!");
    log::info!("  mcause: {:?}", Arch::read_mcause());
}

// ————————————————————————————————— mcause ————————————————————————————————— //

#[derive(Clone, Copy)]
pub struct MCause(usize);

impl MCause {
    pub fn new(cause: usize) -> Self {
        Self(cause)
    }

    pub fn value(self) -> usize {
        self.0
    }
}

// ———————————————————————————————— Display ————————————————————————————————— //

impl fmt::Debug for MCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.value();
        if (val as isize) < 0 {
            // Interrupt
            match val {
                0 => write!(f, "user software interrupt"),
                1 => write!(f, "supervisor software interrupt"),
                3 => write!(f, "machine software interrupt"),
                4 => write!(f, "user timer interrupt"),
                5 => write!(f, "supervisor timer interrupt"),
                7 => write!(f, "machine timer interrupt"),
                8 => write!(f, "user external interrupt"),
                9 => write!(f, "supervisor external interrupt"),
                11 => write!(f, "machine external interrupt"),
                _ => write!(f, "unknown interrupt: 0x{:x}", val),
            }
        } else {
            // Trap
            match val {
                0 => write!(f, "instruction address misaligned"),
                1 => write!(f, "instruction addess fault"),
                2 => write!(f, "illegal instruction"),
                3 => write!(f, "breakpoint"),
                4 => write!(f, "load address misaligned"),
                5 => write!(f, "load access fault"),
                6 => write!(f, "store/amo misaligned"),
                7 => write!(f, "store/amo access fault"),
                8 => write!(f, "ecall from u-mode"),
                9 => write!(f, "ecall from s-mode"),
                11 => write!(f, "ecall from m-mode"),
                12 => write!(f, "instruction page fault"),
                13 => write!(f, "load page fault"),
                15 => write!(f, "store/amo page fault"),
                _ => write!(f, "unknown exception: 0x{:x}", val),
            }
        }
    }
}
