use core::arch::asm;

use sel4_common::{shared_types::MessageInfo, syscall_ids::seL4_SysCall};

fn syscall(
    id: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> (usize, MessageInfo, usize, usize, usize, usize) {
    let mut ret: usize;
    let mut out_msginfo: usize;
    let (mut out_mr0, mut out_mr1, mut out_mr2, mut out_mr3): (usize, usize, usize, usize);
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") a0 => ret,
            inlateout("x11") a1 => out_msginfo,
            inlateout("x12") a2 => out_mr0,
            inlateout("x13") a3 => out_mr1,
            inlateout("x14") a4 => out_mr2,
            inlateout("x15") a5 => out_mr3,
            in("x17") id
        );
    }
    (
        ret,
        MessageInfo(out_msginfo),
        out_mr0,
        out_mr1,
        out_mr2,
        out_mr3,
    )
}

pub fn sys_send_recv(
    sys: usize,
    dest: usize,
    info_arg: usize,
    mr0: &mut usize,
    mr1: &mut usize,
    mr2: &mut usize,
    mr3: &mut usize,
    out_info: &mut MessageInfo,
) {
    let r = syscall(
        sys, dest, info_arg, *mr0 as _, *mr1 as _, *mr2 as _, *mr3 as _,
    );
    *mr0 = r.2;
    *mr1 = r.3;
    *mr2 = r.4;
    *mr3 = r.5;
    *out_info = r.1;
}

pub fn call_with_mrs(
    _service: usize,
    tag: MessageInfo,
    mr0: &mut usize,
    mr1: &mut usize,
    mr2: &mut usize,
    mr3: &mut usize,
) -> MessageInfo {
    let mut info = MessageInfo(0);
    sys_send_recv(seL4_SysCall, _service, tag.0, mr0, mr1, mr2, mr3, &mut info);
    info
}
