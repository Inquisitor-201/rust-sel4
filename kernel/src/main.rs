#![no_std]
#![no_main]

mod kernel;
mod machine;
mod common;

use core::arch::global_asm;

global_asm!(include_str!("head.S"));

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop { }
}