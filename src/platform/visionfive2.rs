//! QEMU Virt board

use core::fmt::Write;
use core::{fmt, hint, ptr};

use log::Level;
use spin::Mutex;

use crate::arch::{Arch, Architecture};
use crate::config::{TARGET_FIRMWARE_ADDRESS, TARGET_START_ADDRESS};
use crate::device::clint::{VirtClint, CLINT_SIZE};
use crate::device::tester::{VirtTestDevice, TEST_DEVICE_SIZE};
use crate::device::VirtDevice;
use crate::driver::clint::ClintDriver;
use crate::Platform;

// —————————————————————————— Platform Parameters ——————————————————————————— //

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
const MIRALIS_START_ADDR: usize = TARGET_START_ADDRESS;
const FIRMWARE_START_ADDR: usize = TARGET_FIRMWARE_ADDRESS;

const CLINT_BASE: usize = 0x2000000;
const TEST_DEVICE_BASE: usize = 0x3000000;

// ———————————————————————————— Platform Devices ———————————————————————————— //

/// The physical CLINT driver.
///
/// SAFETY: this is the only CLINT device driver that we create, and the platform code does not
/// otherwise access the CLINT.
static CLINT_MUTEX: Mutex<ClintDriver> = unsafe { Mutex::new(ClintDriver::new(CLINT_BASE)) };

/// The virtual CLINT device.
static VIRT_CLINT: VirtClint = VirtClint::new(&CLINT_MUTEX);

/// The virtual test device.
static VIRT_TEST_DEVICE: VirtTestDevice = VirtTestDevice::new();

pub static WRITER: Mutex<Writer> = Mutex::new(Writer::new(SERIAL_PORT_BASE_ADDRESS));

/// The list of virtual devices exposed on the platform.
static VIRT_DEVICES: &[VirtDevice; 2] = &[
    VirtDevice {
        start_addr: CLINT_BASE,
        size: CLINT_SIZE,
        name: "CLINT",
        device_interface: &VIRT_CLINT,
    },
    VirtDevice {
        start_addr: TEST_DEVICE_BASE,
        size: TEST_DEVICE_SIZE,
        name: "TEST",
        device_interface: &VIRT_TEST_DEVICE,
    },
];

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VisionFive2Platform {}

impl Platform for VisionFive2Platform {
    const NB_HARTS: usize = 5;
    const NB_VIRT_DEVICES: usize = 2;

    fn name() -> &'static str {
        "VisionFive 2 board"
    }

    fn init() {
        let mut writer = WRITER.lock();
        // NOTE: for now we assume the uart has been initialized by the previous boot stage (U-boot
        // SPL)
        // uart_init(SERIAL_PORT_BASE_ADDRESS);
        writer.write_char('\n');
    }

    fn debug_print(_level: Level, args: fmt::Arguments) {
        let mut writer = WRITER.lock();
        writer.write_fmt(args).unwrap();
        writer.write_str("\r").unwrap();
    }

    fn exit_success() -> ! {
        loop {
            Arch::wfi();
            hint::spin_loop();
        }
    }

    fn exit_failure() -> ! {
        loop {
            Arch::wfi();
            hint::spin_loop();
        }
    }

    fn load_firmware() -> usize {
        FIRMWARE_START_ADDR
    }

    fn get_miralis_start() -> usize {
        MIRALIS_START_ADDR
    }

    fn get_max_valid_address() -> usize {
        usize::MAX
    }

    fn get_virtual_devices() -> &'static [VirtDevice] {
        VIRT_DEVICES
    }

    fn get_clint() -> &'static Mutex<ClintDriver> {
        &CLINT_MUTEX
    }

    fn get_vclint() -> &'static VirtClint {
        &VIRT_CLINT
    }
}

pub struct Writer {
    serial_port_base_addr: usize,
}

impl Writer {
    pub const fn new(serial_port_base_addr: usize) -> Self {
        Writer {
            serial_port_base_addr,
        }
    }

    pub const fn get_register(&mut self, offset: usize) -> usize {
        // Registers are 32 bits wide on the board
        self.serial_port_base_addr + (offset << 2)
    }

    pub fn is_line_busy(&mut self) -> bool {
        // Line Status Register
        const LSR_OFFSET: usize = 0x05;
        // Transmit Holding Register Empty
        const LSR_THRE: u8 = 0x20;

        unsafe { ptr::read_volatile(self.get_register(LSR_OFFSET) as *const u8) & LSR_THRE == 0 }
    }

    fn write_char(&mut self, c: char) {
        unsafe {
            while self.is_line_busy() {}

            ptr::write_volatile(self.serial_port_base_addr as *mut char, c);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

// NOTE: for now this function is not used, as we assume the previous boot stage (U-boot SPL) will
// initialize the uart for us.
#[allow(dead_code)]
fn uart_init(serial_port_base_addr: usize) {
    let reg_lcr = 0x03;
    let reg_brdl = 0x00;
    let reg_brdh = 0x01;
    let reg_mdc = 0x04;
    let reg_fcr = 0x02;
    let reg_ier = 0x01;

    let lcr_dlab = 0x80;
    let lcr_cs8 = 0x03;
    let lcr_1_stb = 0x00;
    let lcr_pdis = 0x00;
    let fcr_fifo = 0x01;
    let fcr_mode1 = 0x08;
    let fcr_fifo_8 = 0x80;
    let fcr_rcvrclr = 0x02;
    let fcr_xmitclr = 0x04;

    let divisor = 0x01;

    // Read LCR and cache its value
    let lcr_cache = unsafe { ptr::read_volatile((serial_port_base_addr + reg_lcr) as *const u8) };

    // Enable DLAB (Divisor Latch Access Bit) to set the baud rate divisor
    unsafe {
        ptr::write_volatile(
            (serial_port_base_addr + reg_lcr) as *mut u8,
            lcr_dlab | lcr_cache,
        );
        ptr::write_volatile(
            (serial_port_base_addr + reg_brdl) as *mut u8,
            (divisor & 0xFF) as u8,
        );
        ptr::write_volatile(
            (serial_port_base_addr + reg_brdh) as *mut u8,
            ((divisor >> 8) & 0xFF) as u8,
        );
        ptr::write_volatile((serial_port_base_addr + reg_lcr) as *mut u8, lcr_cache);
        // Restore LCR
    }

    // Configure UART: 8 data bits, 1 stop bit, no parity
    unsafe {
        ptr::write_volatile(
            (serial_port_base_addr + reg_lcr) as *mut u8,
            lcr_cs8 | lcr_1_stb | lcr_pdis,
        );

        // Disable flow control
        ptr::write_volatile((serial_port_base_addr + reg_mdc) as *mut u8, 0);

        // Configure FIFO: enabled, mode 0, generate interrupt at 8th byte, clear receive and transmit buffers
        ptr::write_volatile(
            (serial_port_base_addr + reg_fcr) as *mut u8,
            fcr_fifo | fcr_mode1 | fcr_fifo_8 | fcr_rcvrclr | fcr_xmitclr,
        );

        // Disable UART interrupts
        ptr::write_volatile((serial_port_base_addr + reg_ier) as *mut u8, 0);
    }
}
