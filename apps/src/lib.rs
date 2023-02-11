#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::global_asm;

use runtime::{BootInfo, Env};

// extern crate alloc;

mod lang_items;
pub mod syscall;
pub mod runtime;
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
pub fn sel4runtime_start_main(bootinfo: *const BootInfo) -> i64 {
    let env = Env::new(bootinfo);
    main()
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i64 {
    panic!("Cannot find main!");
}

global_asm!(include_str!("sel4_crt0.S"));