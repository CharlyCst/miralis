//! QEMU Virt board

use core::arch::asm;
use core::fmt;
use core::fmt::Write;

use spin::Mutex;
use uart_16550::MmioSerialPort;

use super::Platform;

// —————————————————————————— Platform parameters ——————————————————————————— //

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
const TEST_MMIO_ADDRESS: usize = 0x100000;
const PAYLOAD_ADDR: usize = 0x80100000;

static SERIAL_PORT: Mutex<Option<MmioSerialPort>> = Mutex::new(None);

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VirtPlatform {}

impl Platform for VirtPlatform {
    fn init() {
        // Serial
        let mut uart = SERIAL_PORT.lock();
        let mut mmio = unsafe { MmioSerialPort::new(SERIAL_PORT_BASE_ADDRESS) };
        mmio.init();
        *uart = Some(mmio);
    }

    fn debug_print(args: fmt::Arguments) {
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

    fn load_payload() -> usize {
        // We directly load the payload from QEMU, nothing to do here.
        PAYLOAD_ADDR
    }

    fn get_nb_pmp() -> usize {
        16
    }

    fn get_max_valid_address() -> usize {
        usize::MAX
    }
}

fn exit_qemu(success: bool) -> ! {
    let code = if success { 0x5555 } else { (1 << 16) | 0x3333 };

    unsafe {
        asm! {
            "sw {code}, 0({address})",
            code = in(reg) code,
            address = in(reg) TEST_MMIO_ADDRESS,
        }
    }

    // Loop forever if shutdown failed
    loop {
        core::hint::spin_loop();
    }
}
