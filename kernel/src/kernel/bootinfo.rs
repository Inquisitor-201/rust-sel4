use crate::{machine::Vaddr, common::CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS};

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
    pub paddr: usize,   /* physical address of untyped cap  */
    pub size_bits: u8,   /* size (2^n) bytes of each untyped */
    pub is_device: u8,   /* whether the untyped is a device  */
    pub padding: [u8; 6]
}

#[repr(C)]
pub struct BootInfo {
    pub extra_len: usize,
    pub node_id: usize,
    pub num_nodes: usize,
    pub num_io_pt_levels: usize,
    pub ipc_buffer: Vaddr,
    pub untyped: SlotRegion,
    pub untyped_list: [UntypedDesc; CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS]
}
