//! QEMU Virt board

use core::fmt::Write;
use core::{fmt, ptr};

use log::Level;
use spin::Mutex;
use uart_16550::MmioSerialPort;

use super::Platform;
use crate::device::VirtDevice;
use crate::driver::ClintDriver;
use crate::platform::parameters::{
    MIRALIS_START_ADDR, SERIAL_PORT, SERIAL_PORT_BASE_ADDRESS, TEST_MMIO_ADDRESS,
};
use crate::platform::utils::{
    compute_miralis_memory_start_and_size, create_passthrough_device_default,
    create_virtual_devices_default, get_clint_default, get_max_valid_address_default,
    load_firmware_default,
};

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VirtPlatform {}

impl Platform for VirtPlatform {
    const NB_HARTS: usize = usize::MAX;

    fn name() -> &'static str {
        "QEMU virt"
    }

    fn init() {
        // Serial
        let mut uart = SERIAL_PORT.lock();
        let mut mmio = unsafe { MmioSerialPort::new(SERIAL_PORT_BASE_ADDRESS) };
        mmio.init();
        *uart = Some(mmio);
    }

    fn debug_print(_level: Level, args: fmt::Arguments) {
        let mut serial_port = SERIAL_PORT.lock();
        if let Some(ref mut serial_port) = serial_port.as_mut() {
            serial_port
                .write_fmt(args)
                .expect("Printing to serial failed")
        };
    }

    fn exit_success() -> ! {
        exit_qemu(true)
    }

    fn exit_failure() -> ! {
        exit_qemu(false)
    }

    fn load_firmware() -> usize {
        // We directly load the firmware from QEMU, nothing to do here.
        load_firmware_default()
    }

    fn get_miralis_memory_start_and_size() -> (usize, usize) {
        compute_miralis_memory_start_and_size(MIRALIS_START_ADDR)
    }

    fn get_max_valid_address() -> usize {
        get_max_valid_address_default()
    }

    fn create_virtual_devices() -> [VirtDevice; 2] {
        create_virtual_devices_default()
    }

    fn get_clint() -> &'static Mutex<ClintDriver> {
        get_clint_default()
    }

    fn create_passthrough_device() -> VirtDevice {
        create_passthrough_device_default()
    }
}

fn exit_qemu(success: bool) -> ! {
    let code = if success { 0x5555 } else { (1 << 16) | 0x3333 };

    unsafe {
        let mmio_addr = TEST_MMIO_ADDRESS as *mut i32;
        ptr::write_volatile(mmio_addr, code);
    }

    // Loop forever if shutdown failed
    loop {
        core::hint::spin_loop();
    }
}
