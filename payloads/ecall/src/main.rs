#![no_std]
#![no_main]

use core::arch::global_asm;

use mirage_abi::payload_panic;

global_asm!(
    r#"
.text
.align 4
.global _start
_start:
    li a6, 1           // Mirage ABI FID: success
    li a7, 0x08475bcd  // Mirage ABI EID
    ecall
"#,
);

payload_panic!();
