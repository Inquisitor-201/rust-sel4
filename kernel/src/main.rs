#![no_std]
#![no_main]

mod kernel;
mod machine;
mod panic_handler;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
