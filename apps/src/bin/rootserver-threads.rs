#![no_std]
#![no_main]

use apps::{println, runtime::functions::sel4_debug_dump_scheduler};

extern crate apps;

// const ROOT_CNODE: usize = 1;
// const ROOT_VSPACE: usize = 2;
// const ROOT_TCB: usize = 3;
// const TCB_UNTYPED: usize = 4;
// const BUF2_FRAME_CAP: usize = 5;
// const TCB_CAP_SLOT: usize = 6;
// const TCB_IPC_FRAME: usize = 7;

#[no_mangle]
pub fn main() {
    // println!("Hello, world!");
    // sel4_debug_dump_scheduler();
    // let result = sel4_untyped_retype(tcb_untyped, seL4_TCBObject, seL4_TCBBits, root_cnode, 0, 0, tcb_cap_slot, 1);
    // panic!("main exit");
}
