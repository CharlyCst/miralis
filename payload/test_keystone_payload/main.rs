#![no_std]
#![no_main]
#![feature(start)]
// ———————————————————————————————— Guest OS ———————————————————————————————— //

use miralis_abi::{ecall3, log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::info!("Hello from test keystone payload");

    pub const ILLEGAL_ARGUMENT: usize = 100008;
    pub const MIRALIS_KEYSTONE_EID: usize = 0x08424b45;
    pub const CREATE_ENCLAVE_FID: usize = 2001;

    // Test create enclave
    #[repr(C)]
    struct CreateArgs {
        epm_paddr: usize,
        epm_size: usize,
        utm_paddr: usize,
        utm_size: usize,
        runtime_paddr: usize,
        user_paddr: usize,
        free_paddr: usize,
        free_requested: usize,
    }

    let valid_args = CreateArgs {
        // Values copied from keystone
        epm_paddr: 0x83800000,
        epm_size: 0x200000,
        utm_paddr: 0x83240000,
        utm_size: 0x40000,
        runtime_paddr: 0x8380C000,
        user_paddr: 0x83834000,
        free_paddr: 0x838D6000,
        free_requested: 0x40000,
    };

    // Keystone should return SUCCESS if given valid arguments
    unsafe {
        ecall3(
            MIRALIS_KEYSTONE_EID,
            CREATE_ENCLAVE_FID,
            &valid_args as *const CreateArgs as usize,
            0,
            0,
        )
    }
    .expect("Failed to create enclave");

    // Miralis should not crash if given an invalid argument
    let err = unsafe {
        ecall3(
            MIRALIS_KEYSTONE_EID,
            CREATE_ENCLAVE_FID,
            0xDEADBEEFDEADBEEF,
            0,
            0,
        )
    }
    .unwrap_err();
    assert_eq!(err, ILLEGAL_ARGUMENT);

    success()
}
