use crate::{
    kernel::PTE,
    machine::{Paddr, Vaddr, Pregion},
};

#[macro_export]
macro_rules! round_up {
    ($n: expr, $b: expr) => {
        ((($n - 1) >> $b) + 1) << $b
    };
}

#[macro_export]
macro_rules! round_down {
    ($n: expr, $b: expr) => {
        ($n >> $b) << $b
    };
}

#[macro_export]
macro_rules! bit {
    ($b: expr) => {
        1 << $b
    };
}

#[macro_export]
macro_rules! mask {
    ($b: expr) => {
        $crate::bit!($b) - 1
    };
}

#[macro_export]
macro_rules! is_aligned {
    ($n: expr, $b: expr) => {
        ($n & $crate::mask!($b)) == 0
    };
}

pub const KERNEL_HEAP_SIZE: usize = 0x4000;

pub const KERNEL_ELF_BASE: u64 = 0x84000000;
pub const PAGE_PTES: usize = PAGE_SIZE / 8;
pub const PTE_FLAG_BITS: usize = 10;

pub const USER_TOP: u64 = 0x0000003ffffff000;

pub const AVAIL_REGION_START: u64 = 0x80200000;
pub const AVAIL_REGION_END: u64 = 0x90000000;

pub const CONFIG_ROOT_CNODE_SIZE_BITS: usize = 13;
pub const seL4_PageBits: usize = 12;
pub const seL4_SlotBits: usize = 5;
pub const seL4_TCBBits: usize = 10;
pub const seL4_ASIDPoolBits: usize = 12;
pub const seL4_VSpaceBits: usize = 12;
pub const seL4_PageTableBits: usize = 12;
pub const BI_FRAME_SIZE_BITS: usize = seL4_PageBits;

pub const PAGE_SIZE: usize = bit!(seL4_PageBits);