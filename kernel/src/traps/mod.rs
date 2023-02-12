mod syscalls;
mod exceptions;
mod interrupts;
mod basic_syscalls;
mod unknown_syscalls;
mod syscall_ids;

use core::arch::global_asm;

use crate::kernel::ksCurThread;


global_asm!(include_str!("trap.S"));

#[no_mangle]
pub fn restore_user_context() -> ! {
    let cur_thread = ksCurThread.lock().unwrap();
    let cur_thread_reg = (&cur_thread.registers) as *const _ as usize;
    extern "C" {
        fn __restore(_: usize);
    }
    unsafe { __restore(cur_thread_reg) };
    panic!("restore_user_context: Should not reach here");
}
