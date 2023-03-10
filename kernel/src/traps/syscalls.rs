#![allow(unused)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use sel4_common::shared_types::IPCBuffer;

use crate::{
    kernel::statedata::ksCurThread,
    machine::registerset::{msg_registers, n_msgRegisters},
    println,
};

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
        println!("cptr = {:#x?}", cptr);
        handle_basic_syscall(cptr, msg_info, syscall);
    } else {
        handle_unknown_syscall(cptr, msg_info, syscall);
    }
}

pub const seL4_NoError: usize = 0;
pub const seL4_InvalidArgument: usize = 1;
pub const seL4_InvalidCapability: usize = 2;
pub const seL4_IllegalOperation: usize = 3;
pub const seL4_RangeError: usize = 4;
pub const seL4_AlignmentError: usize = 5;
pub const seL4_FailedLookup: usize = 6;
pub const seL4_TruncatedMessage: usize = 7;
pub const seL4_DeleteFirst: usize = 8;
pub const seL4_RevokeFirst: usize = 9;
pub const seL4_NotEnoughMemory: usize = 10;

pub struct SyscallError {
    pub error_type: usize,
}

impl SyscallError {
    pub fn new() -> Self {
        Self {
            error_type: seL4_NoError,
        }
    }
}
pub fn get_syscall_arg(i: usize, ipc_buffer: &IPCBuffer) -> usize {
    if i < n_msgRegisters {
        return ksCurThread.lock().get().unwrap().registers[msg_registers[i] as usize];
    }
    ipc_buffer.msg[i]
}
