#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

use mirage_abi::failure;

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

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    failure();
}
