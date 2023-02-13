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
