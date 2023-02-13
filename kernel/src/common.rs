use sel4_common::{
    bit,
    constants::{seL4_PageBits, seL4_TCBBits},
};

#[macro_export]
macro_rules! mask {
    ($b: expr) => {
        (1 << $b) - 1
    };
}

#[macro_export]
macro_rules! is_aligned {
    ($n: expr, $b: expr) => {
        ($n & $crate::mask!($b)) == 0
    };
}

pub const KERNEL_HEAP_SIZE: usize = 0x4000;

pub const KERNEL_ELF_BASE: usize = 0x84000000;
pub const PAGE_PTES: usize = PAGE_SIZE / 8;
pub const PTE_FLAG_BITS: usize = 10;

pub const USER_TOP: usize = 0x3ffffff000;
pub const CONFIG_PADDR_USER_DEVICE_TOP: usize = 0x8000000000;

pub const AVAIL_REGION_START: usize = 0x80200000;
pub const AVAIL_REGION_END: usize = 0x90000000;

pub const CONFIG_ROOT_CNODE_SIZE_BITS: usize = 13;

pub const seL4_MinUntypedBits: usize = 4;
pub const seL4_MaxUntypedBits: usize = 38;
pub const WORD_BITS: usize = 64;

pub const PAGE_SIZE: usize = bit!(seL4_PageBits);
pub const PT_INDEX_BITS: usize = 9;

pub const TCB_SIZE_BITS: usize = seL4_TCBBits - 1;
pub const TCB_OFFSET: usize = bit!(TCB_SIZE_BITS);

// threads
pub const CONFIG_NUM_PRIORITIES: usize = 256;
pub const seL4_MinPrio: usize = 0;
pub const seL4_MaxPrio: usize = CONFIG_NUM_PRIORITIES - 1;
