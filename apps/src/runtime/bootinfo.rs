use super::CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS;

#[repr(C)]
pub struct BootInfo {
    pub extra_len: usize,
    pub node_id: usize,
    pub num_nodes: usize,
    pub num_io_pt_levels: usize,
    pub ipc_buffer: usize,
    pub empty: SlotRegion,
    pub it_cnode_size_bits: usize,
    pub untyped: SlotRegion,
    pub untyped_list: [UntypedDesc; CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS],
}

#[repr(C)]
pub struct UntypedDesc {
    pub paddr: usize,  /* physical address of untyped cap  */
    pub size_bits: u8, /* size (2^n) bytes of each untyped */
    pub is_device: u8, /* whether the untyped is a device  */
    pub _padding: [u8; 6],
}

#[repr(C)]
#[derive(Debug)]
pub struct SlotRegion {
    pub start: usize, /* first CNode slot position OF region */
    pub end: usize,   /* first CNode slot position AFTER region */
}
