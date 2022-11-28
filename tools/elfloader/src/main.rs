#![no_std]
#![no_main]
mod common;
mod config;
mod machine;

use common::load_images;
use config::*;
use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("crt0.S"));

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

extern "C" {
    pub fn _text();
    pub fn _end();
}

pub fn run_elfloader(_hart_id: u64, bootloader_dtb: *mut u64) -> ! {
    let num_apps = 0usize;
    let kernel_info = load_images(1, bootloader_dtb);
    println!("Jumping to kernel-image entry point...\n");
    let kernel_entry =
        unsafe { core::mem::transmute::<_, fn()>(kernel_info.virt_entry) };
    kernel_entry();
    panic!("Shouldn't reach here!");
}

#[no_mangle]
pub fn main(hart_id: u64, bootloader_dtb: *mut u64) -> ! {
    println!(
        "ELF-loader started on (HART {}) (NODES {})",
        hart_id, CONFIG_MAX_NUM_NODES
    );
    println!(
        "  paddr=[{:#x?}..{:#x?}]",
        _text as usize,
        _end as usize - 1
    );
    run_elfloader(hart_id, bootloader_dtb);
}
