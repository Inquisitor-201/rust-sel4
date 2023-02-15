use sel4_common::syscall_ids::*;

fn handle_invocation(
    cptr: usize,
    msg_info: usize,
    syscall: usize,
    is_call: bool,
    is_blocking: bool,
) {
}

pub fn handle_basic_syscall(cptr: usize, msg_info: usize, syscall: usize) {
    match syscall {
        seL4_SysCall => handle_invocation(cptr, msg_info, syscall, true, true),
        _ => todo!("handle_basic_syscall"),
    }
}
