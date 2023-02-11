#![no_std]
#![no_main]

use apps::println;

extern crate apps;

#[no_mangle]
pub fn main() -> i64 {
    println!("[user] Hello, world\n");
    return -3;
}
