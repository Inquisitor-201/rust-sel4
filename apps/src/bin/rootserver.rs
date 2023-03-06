#![no_std]
#![no_main]

use apps::{println, runtime::functions::sel4_debug_dump_scheduler};

extern crate apps;

#[no_mangle]
pub fn main() {
    println!("Hello, world!");
    sel4_debug_dump_scheduler(); 
}
