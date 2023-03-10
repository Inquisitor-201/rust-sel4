#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod drivers;
mod kernel;
mod machine;
mod object;
mod traps;

#[macro_use]
mod common;

#[macro_use]
extern crate bitflags;
extern crate alloc;

use core::arch::global_asm;

global_asm!(include_str!("head.S"));

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    loop {}
}
