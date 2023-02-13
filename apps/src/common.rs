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

pub const seL4_PageBits: usize = 12;
pub const seL4_SlotBits: usize = 5;
pub const seL4_TCBBits: usize = 10;
pub const seL4_ASIDPoolBits: usize = 12;
pub const seL4_VSpaceBits: usize = 12;
pub const seL4_PageTableBits: usize = 12;
pub const BI_FRAME_SIZE_BITS: usize = seL4_PageBits;