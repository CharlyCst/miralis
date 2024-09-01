use spin::Mutex;

use crate::config::{PLATFORM_NB_HARTS, TARGET_STACK_SIZE};
use crate::device::clint::CLINT_SIZE;
use crate::device::tester::TEST_DEVICE_SIZE;
use crate::device::VirtDevice;
use crate::driver::ClintDriver;
use crate::platform::parameters::{
    CLINT_BASE, CLINT_MUTEX, FIRMWARE_START_ADDR, PASSTHROUGH_BASE, PASSTHROUGH_SIZE,
    PASS_THROUGH_MODULE, TEST_DEVICE_BASE, VIRT_CLINT, VIRT_TEST_DEVICE,
};
use crate::{_stack_start, _start_address, device};

pub fn load_firmware_default() -> usize {
    FIRMWARE_START_ADDR
}

pub fn compute_miralis_memory_start_and_size(miralis_start: usize) -> (usize, usize) {
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

    (miralis_start, size.next_power_of_two())
}

pub fn get_max_valid_address_default() -> usize {
    usize::MAX
}

pub fn create_virtual_devices_default() -> [VirtDevice; 2] {
    let virtual_clint: device::VirtDevice = VirtDevice {
        start_addr: CLINT_BASE,
        size: CLINT_SIZE,
        name: "CLINT",
        device_interface: &VIRT_CLINT,
    };

    let virtual_test_device: device::VirtDevice = VirtDevice {
        start_addr: TEST_DEVICE_BASE,
        size: TEST_DEVICE_SIZE,
        name: "TEST",
        device_interface: &VIRT_TEST_DEVICE,
    };

    [virtual_clint, virtual_test_device]
}

pub fn get_clint_default() -> &'static Mutex<ClintDriver> {
    &CLINT_MUTEX
}

pub fn create_passthrough_device_default() -> VirtDevice {
    unsafe {
        PASS_THROUGH_MODULE.attach_devices();

        VirtDevice {
            start_addr: PASSTHROUGH_BASE,
            size: PASSTHROUGH_SIZE,
            name: "passthrough_module",
            device_interface: &PASS_THROUGH_MODULE,
        }
    }
}
