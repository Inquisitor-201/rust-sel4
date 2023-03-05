use sel4_common::{
    invocation::{LABEL_CNODE_COPY, LABEL_NO_ERROR},
    shared_types::MessageInfo,
    syscall_ids::seL4_SysDebugPutChar, structures_common::CapRights,
};

use crate::syscalls::{call_with_mrs, sys_send_recv};
use sel4_common::shared_types::IPCBuffer;

use super::get_bootinfo;

pub fn sel4_debug_putchar(c: char) {
    let mut unused0: usize = 0;
    let mut unused1: usize = 0;
    let mut unused2: usize = 0;
    let mut unused3: usize = 0;
    let mut unused_info = MessageInfo(0);
    sys_send_recv(
        seL4_SysDebugPutChar,
        c as _,
        0,
        &mut unused0,
        &mut unused1,
        &mut unused2,
        &mut unused3,
        &mut unused_info,
    );
}

pub fn sel4_cnode_copy(
    dest_root: usize,
    dest_index: usize,
    dest_depth: usize,
    src_root: usize,
    src_index: usize,
    src_depth: usize,
    rights: CapRights,
) -> usize {
    // 	seL4_Error result;
    let tag = MessageInfo::new(LABEL_CNODE_COPY, 0, 1, 5);

    /* Setup input capabilities. */
    sel4_setcap(0, src_root);

    /* Marshal and initialise parameters. */
    let mut mr0 = dest_index;
    let mut mr1 = dest_depth & 0xff;
    let mut mr2 = src_index;
    let mut mr3 = src_depth & 0xff;
    sel4_setmr(4, rights.bits());

    /* Perform the call, passing in-register arguments directly. */
    let output_tag = call_with_mrs(dest_root, tag, &mut mr0, &mut mr1, &mut mr2, &mut mr3);
    let result = output_tag.label();

    /* Unmarshal registers into IPC buffer on error. */
    if result != LABEL_NO_ERROR {
        panic!("sel4_cnode_copy: error");
    }

    result
}

pub fn sel4_setcap(i: usize, cptr: usize) {
    sel4_get_ipcbuffer().caps_or_badges[i] = cptr;
}

pub fn sel4_setmr(i: usize, mr: usize) {
    sel4_get_ipcbuffer().msg[i] = mr;
}

pub fn sel4_get_ipcbuffer() -> &'static mut IPCBuffer {
    unsafe { &mut *(get_bootinfo().ipc_buffer as *mut IPCBuffer) }
}
