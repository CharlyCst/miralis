#![no_std]
#![no_main]
#![feature(start)]
// ———————————————————————————————— Guest OS ———————————————————————————————— //

use core::arch::{asm, global_asm};

use miralis_abi::{ecall3, failure, log, setup_binary, success};

setup_binary!(main);

static mut EID: usize = 0; // Enclave ID

pub const ILLEGAL_ARGUMENT: usize = 100008;
pub const MIRALIS_KEYSTONE_EID: usize = 0x08424b45;
pub const CREATE_ENCLAVE_FID: usize = 2001;
pub const DESTROY_ENCLAVE_FID: usize = 2002;
pub const RUN_ENCLAVE_FID: usize = 2003;
pub const RESUME_ENCLAVE_FID: usize = 2005;
pub const ERR_ENCLAVE_INTERRUPTED: usize = 100002;
pub const ERR_ENCLAVE_EDGE_CALL_HOST: usize = 100011;

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

fn main() -> ! {
    log::info!("Hello from test keystone payload");
    // Test create enclave
    unsafe {
        // Miralis should not crash if given an invalid argument
        let err = ecall3(
            MIRALIS_KEYSTONE_EID,
            CREATE_ENCLAVE_FID,
            0xDEADBEEFDEADBEEF,
            0,
            0,
        )
        .unwrap_err();
        assert_eq!(err, ILLEGAL_ARGUMENT);
        log::info!("Illegal argument test passed");

        let shared_memory: [usize; 64] = [0; 64];
        let valid_args = CreateArgs {
            epm_paddr: _enclave as usize,
            epm_size: 0x256,
            utm_paddr: shared_memory.as_ptr() as usize,
            utm_size: shared_memory.len(),
            runtime_paddr: _enclave as usize + 0x128,
            user_paddr: _enclave as usize + 0x128,
            free_paddr: _enclave as usize + 0x128,
            free_requested: 0x40000,
        };

        // Keystone should return SUCCESS if given valid arguments
        EID = ecall3(
            MIRALIS_KEYSTONE_EID,
            CREATE_ENCLAVE_FID,
            &valid_args as *const CreateArgs as usize,
            0,
            0,
        )
        .expect("Failed to create enclave");
        log::info!("Enclave created successfully");

        // Run the enclave
        let mut result = ecall3(MIRALIS_KEYSTONE_EID, RUN_ENCLAVE_FID, EID, 0, 0);
        let mut max_exits = 100;
        log::info!("Enclave ran successfully");
        while result.is_err() {
            max_exits -= 1;
            assert!(
                result.unwrap_err() == ERR_ENCLAVE_INTERRUPTED
                    || result.unwrap_err() == ERR_ENCLAVE_EDGE_CALL_HOST
            );
            assert!(max_exits > 0, "Enclave exited too many times");
            result = ecall3(MIRALIS_KEYSTONE_EID, RESUME_ENCLAVE_FID, EID, 0, 0);
        }

        assert_eq!(result.unwrap(), 0xBEEF);
        log::info!("Enclave exited successfully");

        // Set up a trap handler to catch load access faults
        asm!(
            "csrw stvec, {trap_handler}",
            trap_handler = in(reg) trap_handler as usize & !0b11);

        // Try to access the enclave memory. This should trigger a trap.
        let y = *(_enclave as *const usize);

        log::info!("The enclave memory is not protected: {:x}.", y);
        failure();
    }
}

unsafe fn trap_handler() {
    log::info!("The enclave memory is protected.");

    // Destroy the enclave
    ecall3(MIRALIS_KEYSTONE_EID, DESTROY_ENCLAVE_FID, EID, 0, 0)
        .expect("Failed to destroy enclave");
    log::info!("Enclave destroyed successfully");

    let y = *(_enclave as *const usize);
    log::info!("The enclave is no longer protected: {:x}.", y);
    success()
}

global_asm!(
    r#"
.rodata
_enclave_message:
    .asciz "Hello from enclave!"

.text
.align 4
.global _enclave
_enclave:
    li a0, 3                 # Log level info
    la a1, _enclave_message  # Message
    li a2, 19                # Message length
    li a7, 138894285         # Miralis eid
    li a6, 2                 # Miralis log fid
    ecall

    li a7, 0x08424b45  # Keystone eid
    li a6, 3001        # Keystone random fid
    ecall

    li a7, 0x08424b45  # Keystone eid
    li a6, 3004        # Keystone stop enclave fid
    ecall

    li a7, 0x08424b45  # Keystone eid
    li a6, 3006        # Keystone exit fid
    li a0, 0xBEEF       # Exit code
    ecall
"#,
);

extern "C" {
    fn _enclave();
}
