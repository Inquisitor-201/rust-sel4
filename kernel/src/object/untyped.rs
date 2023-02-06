#[macro_export]
macro_rules! max_free_index {
    ($sizeBits: expr) => {
        bit!($sizeBits - $crate::common::seL4_MinUntypedBits)
    };
}
