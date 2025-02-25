//! SiFive HiFive P550 board

use core::fmt;
use core::fmt::Write;

use log::Level;
use spin::Mutex;

use crate::arch::{read_custom_csr, write_custom_csr};
use crate::device::clint::CLINT_SIZE;
use crate::device::VirtDevice;
use crate::driver::uart::UartDriver;
use crate::platform::{Plat, VIRT_CLINT};
use crate::Platform;

// ———————————————————————————— Platform Devices ———————————————————————————— //

pub static WRITER: Mutex<UartDriver> = Mutex::new(UartDriver::new(
    EIC770X_UART0_ADDR,
    (1 << EIC770X_UART_REG_SHIFT) as usize,
));

/// The list of virtual devices exposed on the platform.
static VIRT_DEVICES: &[VirtDevice; 1] = &[VirtDevice {
    start_addr: Plat::CLINT_BASE,
    size: CLINT_SIZE,
    name: "CLINT",
    device_interface: &VIRT_CLINT,
}];

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct PremierP550Platform {}

impl Platform for PremierP550Platform {
    const NB_HARTS: usize = 4;
    const NB_VIRT_DEVICES: usize = VIRT_DEVICES.len();

    fn name() -> &'static str {
        "Premier P550 board"
    }

    fn init() {
        eic770x_console_init();
        let mut writer = WRITER.lock();
        // NOTE: for now we assume the uart has been initialized by the previous boot stage (U-boot
        // SPL)
        writer.write_char('\n')
    }

    fn debug_print(_level: Level, args: fmt::Arguments) {
        let mut writer = WRITER.lock();
        writer.write_fmt(args).unwrap();
        writer.write_str("\r").unwrap();
    }

    fn get_virtual_devices() -> &'static [VirtDevice] {
        VIRT_DEVICES
    }

    fn is_valid_custom_csr(csr: usize) -> bool {
        // For the list and usage of custom CSRs refer to the EIC7700X SoC Technical Reference
        // Manual.
        (0x7C0..=0x7C4).contains(&csr) || csr == 0x7C8
    }

    fn write_custom_csr(csr: usize, value: usize) {
        // On the P550 we simply pass the write through
        match csr {
            0x7C0 => write_custom_csr!(0x7C0, value),
            0x7C1 => write_custom_csr!(0x7C1, value),
            0x7C2 => write_custom_csr!(0x7C2, value),
            0x7C3 => write_custom_csr!(0x7C3, value),
            0x7C4 => write_custom_csr!(0x7C4, value),
            0x7C8 => write_custom_csr!(0x7C8, value),
            _ => panic!("Invalid custom CSR: 0x{:x}", csr),
        }
    }

    fn read_custom_csr(csr: usize) -> usize {
        // On the P550 we simply pass the read through
        match csr {
            0x7C0 => read_custom_csr!(0x7C0),
            0x7C1 => read_custom_csr!(0x7C1),
            0x7C2 => read_custom_csr!(0x7C2),
            0x7C3 => read_custom_csr!(0x7C3),
            0x7C4 => read_custom_csr!(0x7C4),
            0x7C8 => read_custom_csr!(0x7C8),
            _ => panic!("Invalid custom CSR: 0x{:x}", csr),
        }
    }
}

// ——————————————————————————— Uart code ported from OpenSBI ———————————————————————————— //
// This code was ported from the version 1.4 of OpenSBI                                   //
// The uart8250_init function comes from opensbi/lib/utils/serial/uart8250.c The          //
// eic770x_console_init function comes from platform/eswin/eic770x/platform.c To generate //
// the file, you need the sifive opensbi patches from the meta-sifive repo present on the //
// branch rel/meta-sifive/hifive-premier-p550 available at the following location:        //
// meta-sifive/recipes-bsp/opensbi/opensbi-sifive-hf-prem                                 //
// —————————————————————————————————————————————————————————————————————————————————————— //

/// Base address for the UART on the Premier p550 board
const EIC770X_UART0_ADDR: usize = 0x50900000;
/// Clock rate for the UART on the Premier p550 board
const EIC770X_UART_CLK: usize = 200000000;
/// Baud rate for the UART on the Premier p550 board
const EIC770X_UART_BAUDRATE: usize = 115200;
/// Stride between the registers on the Premier p550 board
const EIC770X_UART_REG_SHIFT: u32 = 2;
/// For the Premier p550 board
const UART_DLF_OFFSET: usize = 48;

/// In:  Recieve Buffer Register
const UART_RBR_OFFSET: u32 = 0;
/// Out: Divisor Latch Low
const UART_DLL_OFFSET: u32 = 0;
/// I/O: Interrupt Enable Register
const UART_IER_OFFSET: u32 = 1;
/// Out: Divisor Latch High
const UART_DLM_OFFSET: u32 = 1;
/// Out: FIFO Control Register
const UART_FCR_OFFSET: u32 = 2;
/// Out: Line Control Register
const UART_LCR_OFFSET: u32 = 3;
/// Out: Modem Control Register
const UART_MCR_OFFSET: u32 = 4;
/// In:  Line Status Register
const UART_LSR_OFFSET: u32 = 5;
/// I/O: Scratch Register
const UART_SCR_OFFSET: u32 = 7;

fn div_round_closest(x: usize, divisor: usize) -> usize {
    (x + (divisor / 2)) / divisor
}

fn set_reg(num: u32, val: u32) {
    let offset: u32 = num << EIC770X_UART_REG_SHIFT;

    let address: usize = EIC770X_UART0_ADDR + offset as usize;

    let ptr = address as *const u16 as *mut u16;

    unsafe { ptr.write_volatile(val as u16) }
}

fn get_reg(num: u32) -> u32 {
    let offset: u32 = num << EIC770X_UART_REG_SHIFT;

    let address: usize = EIC770X_UART0_ADDR + offset as usize;

    let ptr = address as *const u16 as *mut u16;

    unsafe { ptr.read_volatile() as u32 }
}

fn eic770x_console_init() {
    uart8250_init();

    let base_baud = EIC770X_UART_BAUDRATE * 16;
    let mut bdiv_f = EIC770X_UART_CLK % base_baud;
    bdiv_f = div_round_closest(bdiv_f << 0x4, base_baud);

    unsafe {
        let addr = EIC770X_UART0_ADDR + (UART_DLF_OFFSET << 2);
        let ptr = addr as *mut u16;
        ptr.write_volatile(bdiv_f as u16);
    }
}

fn uart8250_init() {
    // Build divisor
    let bdiv =
        ((EIC770X_UART_CLK + 8 * EIC770X_UART_BAUDRATE) / (16 * EIC770X_UART_BAUDRATE)) as u16;

    // Disable all interrupts
    set_reg(UART_IER_OFFSET, 0x00);
    // Enable DLAB
    set_reg(UART_LCR_OFFSET, 0x80);

    if bdiv != 0 {
        // Set divisor low byte
        set_reg(UART_DLL_OFFSET, (bdiv & 0xff) as u32);
        // Set divisor high byte
        set_reg(UART_DLM_OFFSET, ((bdiv >> 8) & 0xff) as u32);
    }

    // 8 bits, no parity, one stop bit
    set_reg(UART_LCR_OFFSET, 0x03);
    // Enable FIFO
    set_reg(UART_FCR_OFFSET, 0x01);
    // No modem control DTR RTS
    set_reg(UART_MCR_OFFSET, 0x00);
    // Clear line status
    get_reg(UART_LSR_OFFSET);
    // Read receive buffer
    get_reg(UART_RBR_OFFSET);
    // Set scratchpad
    set_reg(UART_SCR_OFFSET, 0x00);
}
