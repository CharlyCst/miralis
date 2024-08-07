#![no_std]
#![no_main]

use core::arch::global_asm;

use miralis_abi::firmware_panic;

// This ecall Miralis with benchmark ecall number that results in exits.
global_asm!(
    r#"
.text
.align 4
.global _start
_start:
    csrw mie, 0x1      // Dummy instruction to exit firmware
    li a6, 3           // Miralis ABI FID: benchmark
    li a7, 0x08475bcd  // Miralis ABI EID
    ecall
"#,
);

firmware_panic!();
