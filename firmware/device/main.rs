#![no_std]
#![no_main]

use miralis_abi::{setup_binary, success};

setup_binary!(main);

const TEST_DEVICE_BASE: usize = 0x2020000;
const TEST_DEVICE_MAGIC_REGISTER: usize = TEST_DEVICE_BASE;
const TEST_DEVICE_REMOTE_REGISTER: usize = TEST_DEVICE_BASE + 0x4;

fn main() -> ! {
    log::info!("Hello from driver tester firmware!");

    unsafe {
        // Test read
        assert_eq!(
            (TEST_DEVICE_MAGIC_REGISTER as *const u32).read_volatile(),
            0xdeadbeef
        );
        assert_eq!(
            (TEST_DEVICE_REMOTE_REGISTER as *const u32).read_volatile(),
            0x0
        );

        // Test write
        (TEST_DEVICE_REMOTE_REGISTER as *mut u32).write_volatile(0x43);
        assert_eq!(
            (TEST_DEVICE_REMOTE_REGISTER as *const u32).read_volatile(),
            0x43
        );
    }

    success();
}
