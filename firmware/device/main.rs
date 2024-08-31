#![no_std]
#![no_main]

use miralis_abi::{setup_binary, success};

setup_binary!(main);

const TEST_DEVICE_MAGIC_REGISTER: usize = 0x3000000;
const TEST_DEVICE_REMOTE_REGISTER: usize = 0x3000004;

fn main() -> ! {
    log::info!("Hello from driver tester firmware!");

    unsafe {
        // Test read
        assert_eq!(
            (TEST_DEVICE_MAGIC_REGISTER as *const u32).read_volatile(),
            0xbeef
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
