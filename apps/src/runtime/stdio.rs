use core::fmt::{self, Write};

use crate::syscall::sel4_debug_putchar;

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            sel4_debug_putchar(c);
        }
        Ok(())
    }
}

pub fn console_print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::runtime::console_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}
