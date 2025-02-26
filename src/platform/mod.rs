mod miralis;
mod premierp550;
pub mod virt;
pub mod visionfive2;

use core::{fmt, hint};

use config_select::select_env;
use log::Level;

// Re-export virt platform by default for now
use crate::arch::{Arch, Architecture};
use crate::config::{TARGET_FIRMWARE_ADDRESS, TARGET_START_ADDRESS};
use crate::device::clint::VirtClint;
use crate::device::tester::VirtTestDevice;
use crate::device::VirtDevice;
use crate::driver::clint::clint_driver;
use crate::{debug, logger};

/// The virtual test device.
static VIRT_TEST_DEVICE: VirtTestDevice = VirtTestDevice::new();

/// The virtual CLINT device.
static VIRT_CLINT: VirtClint = VirtClint::default();

/// Export the current platform.
///
/// We use a custom proc macro that checks the value of an environment variable and select the
/// appropriate platform accordingly. This makes it possible to avoid adding an ever increasing set
/// of features and `#[cfg]` guards to select a platform.
pub type Plat = select_env!["MIRALIS_PLATFORM_NAME":
    "miralis"     => miralis::MiralisPlatform
    "visionfive2" => visionfive2::VisionFive2Platform
    "premierp550"             => premierp550::PremierP550Platform
    _             => virt::VirtPlatform

];
pub trait Platform {
    fn name() -> &'static str;
    fn init() {}
    fn debug_print(level: Level, args: fmt::Arguments);
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

    fn get_virtual_devices() -> &'static [VirtDevice];

    fn get_vclint() -> &'static VirtClint {
        &VIRT_CLINT
    }

    /// Signal a pending policy interrupt on all cores and trigger an MSI.
    ///
    /// As a result the policy interrupt callback will be called into on each cores.
    fn broadcast_policy_interrupt(mask: usize) {
        // Mark values in virtual clint
        Self::get_vclint().set_all_policy_msi(mask);

        // Fire physical clint
        clint_driver::trigger_msi_on_all_harts(mask);
    }

    /// Load the firmware (virtual M-mode software) and return its address.
    fn load_firmware() -> usize {
        TARGET_FIRMWARE_ADDRESS
    }
    /// Returns the start and size of Miralis's own memory.
    fn get_miralis_start() -> usize {
        TARGET_START_ADDRESS
    }

    /// Return maximum valid address
    fn get_max_valid_address() -> usize {
        usize::MAX
    }

    /// Returns true if the
    fn is_valid_custom_csr(csr: usize) -> bool {
        // Default implementation, no valid custom CSR
        let _ = csr;
        false
    }

    /// Writes to a platform-specific CSR.
    fn write_custom_csr(csr: usize, value: usize) {
        // By default drop the writes.
        debug::warn_once!(
            "Trying to write to custom CSR 0x{:x}, but custom CSR write is not implemented",
            csr
        );
        let _ = value; // Supress unused warning
    }

    /// Reads a platform-specific CSR.
    fn read_custom_csr(csr: usize) -> usize {
        unimplemented!(
            "Trying to read custom CSR 0x{:x}, but custom CSR read is not implemented",
            csr
        );
    }

    const NB_HARTS: usize;
    const NB_VIRT_DEVICES: usize;

    const CLINT_BASE: usize = 0x2000000;

    const TEST_DEVICE_BASE: usize = 0x3000000;
}

pub fn init() {
    Plat::init();
    logger::init();

    // Trap handler
    Arch::init();

    // Ideally we would like to check this statically, until we find a good solution we assert it
    // at runtime.
    assert_eq!(
        Plat::NB_VIRT_DEVICES,
        Plat::get_virtual_devices().len(),
        "Mismatch between advertised number of devices and returned value"
    );
}
