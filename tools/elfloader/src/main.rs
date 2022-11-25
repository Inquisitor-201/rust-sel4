#![no_std]
#![no_main]
use core::arch::global_asm;

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop { }
}