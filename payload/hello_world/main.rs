#![no_std]
#![no_main]

// ———————————————————————————————— Guest OS ———————————————————————————————— //

use miralis_abi::{log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::info!("Payload Hello world!\n");
    success()
}
