use core::fmt::{self, Formatter};

use crate::{bit, common::CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS, machine::Vaddr, println};
use core::fmt::Debug;

#[repr(C)]
#[derive(Debug)]
pub struct SlotRegion {
    pub start: usize, /* first CNode slot position OF region */
    pub end: usize,   /* first CNode slot position AFTER region */
}

impl SlotRegion {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[repr(C)]
pub struct UntypedDesc {
    pub paddr: usize,  /* physical address of untyped cap  */
    pub size_bits: u8, /* size (2^n) bytes of each untyped */
    pub is_device: u8, /* whether the untyped is a device  */
    pub _padding: [u8; 6],
}

impl Debug for UntypedDesc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "UntypedDesc {{ PA range: [{:#x?}..{:#x?}), size_bits: {}, is_device: {} }}",
            self.paddr,
            self.paddr + bit!(self.size_bits),
            self.size_bits,
            self.is_device != 0
        ))
    }
}

#[repr(C)]
pub struct BootInfo {
    pub extra_len: usize,
    pub node_id: usize,
    pub num_nodes: usize,
    pub num_io_pt_levels: usize,
    pub ipc_buffer: Vaddr,
    pub empty: SlotRegion,
    pub untyped: SlotRegion,
    pub untyped_list: [UntypedDesc; CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS],
}

impl BootInfo {
    pub fn debug_print_info(&self) {
        println!("\n****** bootinfo ******");
        println!("extra_len = {:#x?}", self.extra_len);
        println!("node_id = {}", self.node_id);
        println!("num_nodes = {}", self.num_nodes);
        println!("num_io_pt_levels = {}", self.num_io_pt_levels);
        println!("ipc_buffer = {:#x?}", self.ipc_buffer);
        println!("empty = {:?}", self.empty);
        println!("untyped = {:?}", self.untyped);
        let mut i = 0;
        for desc in self.untyped_list.iter() {
            if desc.size_bits == 0 {
                break;
            }
            println!("untyped_list({}) = {:x?}", i, desc);
            i += 1;
        }
    }
}
