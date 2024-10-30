//! Clint utilities
//!
//! The functions exposed in this modules assumes the CLINT is located at the same addresses as on
//! the QEMU virt platform, which we use for out tests.

/// Clint base, assuming a QEMU virt-like layout.
const CLINT_BASE: usize = 0x2000000;
const MTIME_OFFSET: usize = 0xBFF8;
const MTIMECMP_OFFSET: usize = 0x4000;

/// Get the current mtime value
pub fn read_mtime() -> usize {
    let mtime_ptr = (CLINT_BASE + MTIME_OFFSET) as *const usize;
    unsafe { mtime_ptr.read_volatile() }
}

/// Set mtimecmp deadline in the future
pub fn set_mtimecmp_deadline(delta: usize, hart: usize) {
    let current_mtime = read_mtime();
    let future_time = current_mtime.saturating_add(delta);

    let mtimecmp_ptr = (CLINT_BASE + MTIMECMP_OFFSET + 8 * hart) as *mut usize;
    unsafe {
        mtimecmp_ptr.write_volatile(future_time);
    }
}

/// Send an MSI to the given hart.
pub fn send_msi(hart: usize) {
    let msip_ptr = (CLINT_BASE + 4 * hart) as *mut u32;
    unsafe {
        msip_ptr.write_volatile(1);
    }
}

/// Clear MSI for the given hart.
pub fn clear_msi(hart: usize) {
    let msip_ptr = (CLINT_BASE + 4 * hart) as *mut u32;
    unsafe {
        msip_ptr.write_volatile(0);
    }
}
