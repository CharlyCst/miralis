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
    ecall
"#,
);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    failure();
}
