#![no_std]
#![no_main]

use core::arch::global_asm;

use miralis_abi::firmware_panic;

global_asm!(
    r#"
.text
.align 4
.global _start
_start:
    li a6, 1           // Miralis ABI FID: success
    li a7, 0x08475bcd  // Miralis ABI EID
    ecall
"#,
);

firmware_panic!();
