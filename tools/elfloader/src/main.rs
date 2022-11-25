#![no_std]
#![no_main]
mod machine;

use core::arch::global_asm;
use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop { }
}

#[no_mangle]
pub fn main(hard_id: i64, bootloader_dtb: *mut u64) -> ! {
    println!("hard_id = {:#x?}, bootloader_dtb = {:#x?}", hard_id, bootloader_dtb);
    panic!();
}