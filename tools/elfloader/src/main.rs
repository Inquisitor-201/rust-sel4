#![no_std]
#![no_main]
#![feature(panic_info_message)]
mod common;
mod config;
mod lang_items;
mod machine;

use common::load_images;
use config::*;
use core::arch::global_asm;

global_asm!(include_str!("crt0.S"));

type InitRiscvKernelFn = fn(usize, usize, usize, usize, usize, usize);

extern "C" {
    pub fn _text();
    pub fn _end();
}

pub fn run_elfloader(_hart_id: usize, bootloader_dtb: *mut usize) -> ! {
    let num_apps = 0usize;
    let (kernel_info, user_info) = load_images(1, bootloader_dtb);
    println!("Jumping to kernel-image entry point...\n");
    let kernel_entry =
        unsafe { core::mem::transmute::<_, InitRiscvKernelFn>(kernel_info.virt_entry) };
    kernel_entry(
        user_info.phys_region_start,
        user_info.phys_region_end,
        user_info.phys_virt_offset,
        user_info.virt_entry,
        0,
        0,
    );
    panic!("Shouldn't reach here!");
}

#[no_mangle]
pub fn main(hart_id: usize, bootloader_dtb: *mut usize) -> ! {
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
