use sel4_common::shared_types::IPCBuffer;

use crate::machine::registerset::n_msgRegisters;

use super::{
    basic_syscalls::handle_basic_syscall, restore_user_context,
    unknown_syscalls::handle_unknown_syscall,
};

pub const BASIC_SYSCALL_MIN: isize = -8;
pub const BASIC_SYSCALL_MAX: isize = -1;

#[no_mangle]
pub fn handle_syscall(cptr: usize, msg_info: usize, syscall: usize) -> ! {
    // println!("syscall 0: {:#x?}, {:#x?}, {:#x?}", cptr, msg_info, syscall);
    slowpath(cptr, msg_info, syscall);
    restore_user_context();
}

pub fn slowpath(cptr: usize, msg_info: usize, syscall: usize) {
    if syscall as isize >= BASIC_SYSCALL_MIN && syscall as isize <= BASIC_SYSCALL_MAX {
        handle_basic_syscall(cptr, msg_info, syscall);
    } else {
        handle_unknown_syscall(cptr, msg_info, syscall);
    }
}

pub struct SyscallError {}

pub fn get_syscall_arg(i: usize, ipc_buffer: &mut IPCBuffer) -> usize {
    if i < n_msgRegisters {
        todo!()
    }
    ipc_buffer.msg[i]
}
