use sel4_common::syscall_ids::*;

use crate::machine::sbi::console_putchar;

pub fn handle_unknown_syscall(cptr: usize, msg_info: usize, syscall: usize) {
    match syscall {
        seL4_SysDebugPutChar => console_putchar(cptr),
        _ => todo!("unsupported unknown syscall {}", cptr as isize),
    }
}
