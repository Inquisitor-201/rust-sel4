#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::{asm, global_asm};

use runtime::{Env, ENV};
use sel4_common::bootinfo_common::BootInfo;

// extern crate alloc;

mod lang_items;
pub mod runtime;
pub mod syscalls;
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
    *(ENV.lock()) = Some(Env::new(bootinfo));
    let mut ret;
    unsafe {
        asm!("jal main", 
        out("x10") ret);
    }
    ret
}

global_asm!(include_str!("sel4_crt0.S"));
