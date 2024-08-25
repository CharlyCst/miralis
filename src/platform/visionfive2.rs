//! QEMU Virt board

use core::arch::asm;
use core::fmt::Write;
use core::{fmt, hint, ptr};

use log::Level;
use spin::Mutex;

use crate::arch::{Arch, Architecture};
use crate::config::{
    PLATFORM_NB_HARTS, TARGET_FIRMWARE_ADDRESS, TARGET_STACK_SIZE, TARGET_START_ADDRESS,
};
use crate::device::{self, VirtClint};
use crate::driver::ClintDriver;
use crate::{Platform, _stack_start, _start_address};
// —————————————————————————— Platform Parameters ——————————————————————————— //

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
const MIRALIS_START_ADDR: usize = TARGET_START_ADDRESS;
const FIRMWARE_START_ADDR: usize = TARGET_FIRMWARE_ADDRESS;

const CLINT_BASE: usize = 0x2000000;

// ———————————————————————————— Platform Devices ———————————————————————————— //

/// The physical CLINT driver.
///
/// SAFETY: this is the only CLINT device driver that we create, and the platform code does not
/// otherwise access the CLINT.
static CLINT_MUTEX: Mutex<ClintDriver> = unsafe { Mutex::new(ClintDriver::new(CLINT_BASE)) };

/// The virtual CLINT device.
static VIRT_CLINT: VirtClint = VirtClint::new(&CLINT_MUTEX);
pub static WRITER: Mutex<Writer> = Mutex::new(Writer::new(SERIAL_PORT_BASE_ADDRESS));

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VisionFive2Platform {}

impl Platform for VisionFive2Platform {
    const NB_HARTS: usize = 5;

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
        writer.write_str("\r\n").unwrap();
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

    fn get_miralis_memory_start_and_size() -> (usize, usize) {
        let size: usize;
        // SAFETY: The unsafe block is required to get the address of the stack and start of
        // Miralis, which are external values defined by the linker.
        // We also ensure that `size` is non-negative and within reasonable bounds
        unsafe {
            size = (_stack_start as usize)
                .checked_sub(_start_address as usize)
                .and_then(|diff| diff.checked_add(TARGET_STACK_SIZE * PLATFORM_NB_HARTS))
                .unwrap();
        }

        (MIRALIS_START_ADDR, size.next_power_of_two())
    }

    fn get_max_valid_address() -> usize {
        usize::MAX
    }

    fn create_clint_device() -> device::VirtDevice {
        device::VirtDevice {
            start_addr: CLINT_BASE,
            size: device::CLINT_SIZE,
            name: "CLINT",
            device_interface: &VIRT_CLINT,
        }
    }

    fn get_clint() -> &'static Mutex<ClintDriver> {
        &CLINT_MUTEX
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

    fn write_char(&mut self, c: char) {
        // const LSR_OFFSET: usize = 0x05;
        // const LSR_THRE: u8 = 0x20;

        unsafe {
            // Wait until THR (Transmitter Holding Register) is ready
            // For now that's disabled, on the board this bit of LSR always reads as 0
            // Which leads to an infinite wait cycle

            // while ptr::read_volatile((self.serial_port_base_addr + LSR_OFFSET) as *const u8)
            //     & LSR_THRE
            //     == 0
            // {}

            ptr::write_volatile(self.serial_port_base_addr as *mut char, c);
            for _n in 1..1000001 {
                asm!("nop");
            }
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
