//! QEMU Virt board

use core::{fmt, hint};

use log::Level;
use spin::Mutex;

use crate::arch::{Arch, Architecture};
use crate::config::{TARGET_FIRMWARE_ADDRESS, TARGET_START_ADDRESS};
use crate::device::clint::{VirtClint, CLINT_SIZE};
use crate::device::tester::{VirtTestDevice, TEST_DEVICE_SIZE};
use crate::device::VirtDevice;
use crate::driver::clint::ClintDriver;
use crate::driver::uart::UartDriver;
use crate::Platform;

// —————————————————————————— Platform Parameters ——————————————————————————— //

const EIC770X_UART0_ADDR: usize = 0x50900000;
const EIC770X_REGISTER_SIZE: usize = 0x4;

// TODO: Check about that if it is the case or not
const MIRALIS_START_ADDR: usize = TARGET_START_ADDRESS;
const FIRMWARE_START_ADDR: usize = TARGET_FIRMWARE_ADDRESS;

// TODO: What is the base of the clint device?
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

pub static WRITER: Mutex<UartDriver> =
    Mutex::new(UartDriver::new(EIC770X_UART0_ADDR, EIC770X_REGISTER_SIZE));

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

pub struct PremierP550Platform {}

impl Platform for PremierP550Platform {
    // TODO: Change this number
    const NB_HARTS: usize = 5;
    // TODO: Check this number
    const NB_VIRT_DEVICES: usize = 2;

    fn name() -> &'static str {
        "Premier P550 board"
    }

    fn init() {
        let mut writer = WRITER.lock();
        // NOTE: for now we assume the uart has been initialized by the previous boot stage (U-boot
        // SPL)
        writer.write_char('\n')
    }

    fn debug_print(_level: Level, args: fmt::Arguments) {
        /*let mut writer = WRITER.lock();
        writer.write_fmt(args).unwrap();
        writer.write_str("\r\n").unwrap();*/
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
