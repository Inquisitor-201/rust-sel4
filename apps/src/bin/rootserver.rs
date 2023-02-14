#![no_std]
#![no_main]

use apps::{
    println,
    runtime::{functions::sel4_cnode_copy, get_bootinfo},
};
use sel4_common::{
    bit,
    constants::seL4_SlotBits,
    structures_common::{seL4_AllRights, seL4_CapInitThreadCNode, seL4_CapInitThreadTCB},
};

extern crate apps;

#[no_mangle]
pub fn main() -> i64 {
    let info = get_bootinfo();

    let initial_cnode_object_size = bit!(info.it_cnode_size_bits);
    println!(
        "Initial CNode is {} slots in size\n",
        initial_cnode_object_size
    );

    println!(
        "The CNode is {} bytes in size\n",
        initial_cnode_object_size * bit!(seL4_SlotBits)
    );

    let first_free_slot = info.empty.start;
    let error = sel4_cnode_copy(
        seL4_CapInitThreadCNode,
        first_free_slot,
        usize::BITS as _,
        seL4_CapInitThreadCNode,
        seL4_CapInitThreadTCB,
        usize::BITS as _,
        seL4_AllRights,
    );
    error as _
}
