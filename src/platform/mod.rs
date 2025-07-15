mod miralis;
mod premierp550;
mod virt;
mod visionfive2;

use core::{fmt, hint};

use config_select::select_env;
use log::Level;
pub use miralis::MiralisPlatform;
pub use premierp550::PremierP550Platform;
pub use virt::VirtPlatform;
pub use visionfive2::VisionFive2Platform;

// Re-export virt platform by default for now
use crate::arch::{Arch, Architecture};
use crate::config::{TARGET_FIRMWARE_ADDRESS, TARGET_START_ADDRESS};
use crate::device::clint::VirtClint;
use crate::driver::clint::ClintDriver;
use crate::{debug, device, logger};

// ——————————————————————————— Platform Constants ——————————————————————————— //

/// The expected number of harts.
pub const PLATFORM_NB_HARTS: usize = {
    use miralis_config::PLATFORM_NB_HARTS;

    if PLATFORM_NB_HARTS < Plat::NB_HARTS {
        PLATFORM_NB_HARTS
    } else {
        Plat::NB_HARTS
    }
};

/// A mask containing all harts
pub const ALL_HARTS_MASK: usize = (1 << PLATFORM_NB_HARTS) - 1;

// ———————————————————————————— Platform Choice ————————————————————————————— //

/// Export the current platform.
///
/// We use a custom proc macro that checks the value of an environment variable and select the
/// appropriate platform accordingly. This makes it possible to avoid adding an ever increasing set
/// of features and `#[cfg]` guards to select a platform.
pub type Plat = select_env!["MIRALIS_PLATFORM_NAME":
    "miralis"     => miralis::MiralisPlatform
    "visionfive2" => visionfive2::VisionFive2Platform
    "premierp550" => premierp550::PremierP550Platform
    _             => virt::VirtPlatform

];

// ————————————————————————————— Platform Trait ————————————————————————————— //

pub trait Platform {
    fn name() -> &'static str;
    fn debug_print(level: Level, args: fmt::Arguments);
    fn get_virtual_devices() -> &'static [device::VirtDevice];
    fn get_clint() -> &'static ClintDriver;
    fn get_vclint() -> &'static VirtClint;

    // Platform specific initialization.
    fn init() {}

    /// Halt Miralis and signal a success.
    ///
    /// The exact behavior is platform dependant.
    fn exit_success() -> ! {
        loop {
            Arch::wfi();
            hint::spin_loop();
        }
    }

    /// Halt Miralis and signal a failure.
    ///
    /// The exact behavior is platform dependant.
    fn exit_failure() -> ! {
        loop {
            Arch::wfi();
            hint::spin_loop();
        }
    }

    /// Signal a pending policy interrupt on all cores and trigger an MSI.
    ///
    /// As a result the policy interrupt callback will be called into on each cores.
    fn broadcast_policy_interrupt(mask: usize) {
        // Mark values in virtual clint
        Self::get_vclint().set_all_policy_msi(mask);

        // Fire physical clint
        Self::get_clint().trigger_msi_on_all_harts(mask);
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
}

// ————————————————————————————— Platform Utils ————————————————————————————— //

/// Initializes the platform.
///
/// Mut be called as the first action when booting Miralis.
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
