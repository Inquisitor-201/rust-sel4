mod cap_ops;
mod syscall_ids;

use core::arch::asm;

use self::syscall_ids::seL4_SysDebugPutChar;

fn syscall(
    id: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> (usize, usize, usize, usize, usize) {
    let mut ret: usize;
    let (mut out_mr0, mut out_mr1, mut out_mr2, mut out_mr3): (usize, usize, usize, usize);
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") a0 => ret,
            in("x11") a1,
            inlateout("x12") a2 => out_mr0,
            inlateout("x13") a3 => out_mr1,
            inlateout("x14") a4 => out_mr2,
            inlateout("x15") a5 => out_mr3,
            in("x17") id
        );
    }
    (ret, out_mr0, out_mr1, out_mr2, out_mr3)
}

pub fn sys_send_recv(
    sys: usize,
    dest: usize,
    info_arg: usize,
    mr0: &mut usize,
    mr1: &mut usize,
    mr2: &mut usize,
    mr3: &mut usize,
) {
    let r = syscall(
        sys, dest, info_arg, *mr0 as _, *mr1 as _, *mr2 as _, *mr3 as _,
    );
    *mr0 = r.1;
    *mr1 = r.2;
    *mr2 = r.3;
    *mr3 = r.4;
}

pub fn sel4_debug_putchar(c: char) {
    let mut unused0: usize = 0;
    let mut unused1: usize = 0;
    let mut unused2: usize = 0;
    let mut unused3: usize = 0;

    sys_send_recv(
        seL4_SysDebugPutChar,
        c as _,
        0,
        &mut unused0,
        &mut unused1,
        &mut unused2,
        &mut unused3,
    );
}
