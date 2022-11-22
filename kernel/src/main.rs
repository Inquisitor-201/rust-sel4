#![no_std]
#![no_main]
mod machine;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

// #[no_mangle]
pub fn main() {
    println!("Hello, world!");
}
