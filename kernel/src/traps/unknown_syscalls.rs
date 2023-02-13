use crate::machine::console_putchar;
use sel4_common::syscall_ids::*;

pub fn handle_unknown_syscall(cptr: usize, msg_info: usize, syscall: usize) {
    match syscall {
        seL4_SysDebugPutChar => console_putchar(cptr),
        _ => todo!("unsupported unknown syscall {}", cptr as isize),
    }
}
