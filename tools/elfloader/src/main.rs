#![no_std]
#![no_main]
mod config;
mod machine;
mod common;

use config::*;
use core::arch::global_asm;
use core::panic::PanicInfo;
use common::load_images;

global_asm!(include_str!("crt0.S"));

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

extern "C" {
    pub fn _text();
    pub fn _end();
}

pub fn run_elfloader(_hart_id: u64, bootloader_dtb: *mut u64) {
    let num_apps = 0usize;
    load_images(1, bootloader_dtb);
}

#[no_mangle]
pub fn main(hart_id: u64, bootloader_dtb: *mut u64) -> ! {
    println!("ELF-loader started on (HART {}) (NODES {})", hart_id, CONFIG_MAX_NUM_NODES);
    println!("  paddr=[{:#x?}..{:#x?}]", _text as usize, _end as usize - 1);
    let ret = run_elfloader(hart_id, bootloader_dtb);
    panic!();
}
