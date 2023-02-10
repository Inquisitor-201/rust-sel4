use core::arch::global_asm;

use super::statedata::ksCurThread;

global_asm!(include_str!("trap.S"));

#[no_mangle]
pub fn restore_user_context() {
    let cur_thread = ksCurThread.lock().unwrap();
    let cur_thread_reg = (&cur_thread.registers) as *const _ as usize;
    extern "C" {
        fn __restore(_: usize);
    }
    panic!("restore");
    unsafe { __restore(cur_thread_reg) };
}
