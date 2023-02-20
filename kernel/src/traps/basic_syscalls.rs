use sel4_common::{
    shared_types::{IPCBuffer, MessageInfo},
    syscall_ids::*, invocation::LABEL_CNODE_COPY,
};

use crate::{
    kernel::{
        cspace::lookup_slot,
        statedata::ksCurThread,
        structures::{CapInfo, CapSlot, Capability},
        vspace::lookup_ipc_buffer,
    },
    println,
};

fn decode_cnode_invocation(
    inv_label: usize,
    length: usize,
    cap: Capability,
    buffer: &mut IPCBuffer,
) {
    match inv_label {
        LABEL_CNODE_COPY => {
            let cap_rights = rights_from_word(get_syscall_arg(4, buffer));
            
        }
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
) -> bool {
    let cap = slot.cap;
    match cap.get_info() {
        CapInfo::NullCap => {
            println!("Attempted to invoke a null cap {:#x?}", cap_index);
            return false;
        }
        CapInfo::CnodeCap { ptr } => {
            return decode_cnode_invocation(inv_label, length, cap, buffer);
        }
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

    decode_invocation(
        info.label(),
        info.length(),
        cptr,
        lu_ret,
        isBlocking,
        isCall,
        buffer,
    )
}

pub fn handle_basic_syscall(cptr: usize, msg_info: usize, syscall: usize) {
    match syscall {
        seL4_SysCall => handle_invocation(cptr, msg_info, syscall, true, true),
        _ => todo!("handle_basic_syscall"),
    }
}
