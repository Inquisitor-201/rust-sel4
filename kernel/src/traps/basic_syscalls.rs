#![allow(non_upper_case_globals)]

use sel4_common::{
    invocation::LABEL_CNODE_COPY,
    shared_types::{IPCBuffer, MessageInfo},
    syscall_ids::*,
};

use crate::{
    kernel::{
        cspace::lookup_slot,
        statedata::ksCurThread,
        structures::{CapInfo, CapSlot, Capability},
        vspace::lookup_ipc_buffer, thread::ThreadState_Restart,
    },
    object::{
        cnode::{cte_insert, derive_cap},
        tcb::{lookup_extra_caps, CUR_EXTRA_CAPS},
    },
    println,
    traps::syscalls::{seL4_TruncatedMessage, SyscallError, seL4_InvalidCapability},
};

use crate::traps::syscalls::get_syscall_arg;

use super::syscalls::seL4_NoError;

fn decode_cnode_invocation(
    inv_label: usize,
    length: usize,
    dest_cap: Capability,
    buffer: &mut IPCBuffer,
) -> SyscallError {
    let mut ret = SyscallError::new();

    let dest_index = get_syscall_arg(0, buffer);
    let dest_depth = get_syscall_arg(1, buffer);
    let dest_slot = dest_cap.cnode_slot_at(dest_index);

    let src_index = get_syscall_arg(2, buffer);
    let src_depth = get_syscall_arg(3, buffer);

    let src_root = CUR_EXTRA_CAPS.lock()[0].cap;
    let src_slot = src_root.cnode_slot_at(src_index);

    let new_cap: Capability;
    let is_move: bool;

    match inv_label {
        LABEL_CNODE_COPY => {
            if length < 5 {
                println!("Truncated message for CNode Copy operation.");
                ret.error_type = seL4_TruncatedMessage;
                return ret;
            }
            let cap_rights = get_syscall_arg(4, buffer);
            let src_cap = src_slot.cap;
            new_cap = derive_cap(src_cap);
            is_move = false;
        }
        _ => todo!(),
    }

    let cur_thread = ksCurThread.lock();
    // (*cur_thread).unwrap().set_thread_state(ThreadState_Restart);

    if is_move {
        todo!("cnode invocation: is move");
    } else {
        cte_insert(new_cap, src_slot, dest_slot);
        return ret;
    }
}

fn decode_invocation(
    inv_label: usize,
    length: usize,
    cap_index: usize,
    slot: &CapSlot,
    is_blocking: bool,
    is_call: bool,
    buffer: &mut IPCBuffer,
) -> SyscallError {
    let cap = slot.cap;
    let mut ret = SyscallError::new();
    match cap.get_info() {
        CapInfo::NullCap => {
            println!("Attempted to invoke a null cap {:#x?}", cap_index);
            ret.error_type = seL4_InvalidCapability;
            return ret;
        }
        CapInfo::CnodeCap { .. } => {
            return decode_cnode_invocation(inv_label, length, cap, buffer);
        }
        _ => todo!(),
    }
}

fn handle_invocation(
    cptr: usize,
    msg_info: usize,
    syscall: usize,
    is_call: bool,
    is_blocking: bool,
) {
    let cur_thread = ksCurThread.lock().unwrap();
    let lu_ret = lookup_slot(cur_thread, cptr);
    let buffer = unsafe { lookup_ipc_buffer(false, cur_thread).as_mut::<IPCBuffer>() };
    let info = MessageInfo(msg_info);

    lookup_extra_caps(cur_thread, buffer, info);

    let status = decode_invocation(
        info.label(),
        info.length(),
        cptr,
        lu_ret,
        is_blocking,
        is_call,
        buffer,
    );
    
    match status.error_type {
        seL4_NoError => { }
        _ => todo!()
    }

    // todo: reschedule??
}

pub fn handle_basic_syscall(cptr: usize, msg_info: usize, syscall: usize) {
    match syscall {
        seL4_SysCall => handle_invocation(cptr, msg_info, syscall, true, true),
        _ => todo!("handle_basic_syscall"),
    }
}
