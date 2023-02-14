pub const seL4_PageBits: usize = 12;
pub const seL4_SlotBits: usize = 5;
pub const seL4_TCBBits: usize = 10;
pub const seL4_ASIDPoolBits: usize = 12;
pub const seL4_VSpaceBits: usize = 12;
pub const seL4_PageTableBits: usize = 12;
pub const BI_FRAME_SIZE_BITS: usize = seL4_PageBits;

pub const CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS: usize = 230;
pub const seL4_MsgMaxLength: usize = 120;
pub const seL4_MsgMaxExtraCaps: usize = 3;