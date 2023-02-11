#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::global_asm;

// extern crate alloc;

mod lang_items;
// use buddy_system_allocator::LockedHeap;

// const USER_HEAP_SIZE: usize = 32768;

// #[global_allocator]
// static HEAP: LockedHeap<32> = LockedHeap::empty();

// #[alloc_error_handler]
// pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
//     panic!("Heap allocation error, layout = {:?}", layout);
// }

// static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[no_mangle]
pub fn sel4_start_root(bootinfo: usize) -> usize {
    return bootinfo;
}

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    panic!("Cannot find main!");
}

global_asm!(include_str!("sel4_crt0.S"));