#![no_std]
#![no_main]

use apps::{println, runtime::{get_env, get_bootinfo}, bit, common::seL4_SlotBits};

extern crate apps;

#[no_mangle]
pub fn main() -> i64 {
    let info = get_bootinfo();

    let initial_cnode_object_size = bit!(info.it_cnode_size_bits);
    println!("Initial CNode is {} slots in size\n", initial_cnode_object_size);

    println!("The CNode is {} bytes in size\n", initial_cnode_object_size * bit!(seL4_SlotBits));

    let first_free_slot = info.empty.start;
    // seL4_Error error = seL4_CNode_Copy(seL4_CapInitThreadCNode, first_free_slot, seL4_WordBits,
    //                                    seL4_CapInitThreadCNode, seL4_CapInitThreadTCB, seL4_WordBits,
    //                                    seL4_AllRights);
    0
}
