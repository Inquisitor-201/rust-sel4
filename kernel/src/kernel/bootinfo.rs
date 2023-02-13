use crate::println;
use sel4_common::bootinfo_common::BootInfo;

pub fn debug_print_bi_info(bootinfo: &BootInfo) {
    println!("\n****** bootinfo ******");
    println!("extra_len = {:#x?}", bootinfo.extra_len);
    println!("node_id = {}", bootinfo.node_id);
    println!("num_nodes = {}", bootinfo.num_nodes);
    println!("num_io_pt_levels = {}", bootinfo.num_io_pt_levels);
    println!("ipc_buffer = {:#x?}", bootinfo.ipc_buffer);
    println!("empty = {:?}", bootinfo.empty);
    println!("untyped = {:?}", bootinfo.untyped);
    let mut i = 0;
    for desc in bootinfo.untyped_list.iter() {
        if desc.size_bits == 0 {
            break;
        }
        println!("untyped_list({}) = {:x?}", i, desc);
        i += 1;
    }
}
