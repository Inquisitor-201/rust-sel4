use sel4_common::syscall_ids::*;

use crate::{machine::sbi::console_putchar, println, kernel::thread::THREAD_LIST};

fn debug_dump_scheduler() {
    println!("Dumping all tcbs!");
    println!("Name                                    \tState          \tIP                  \t Prio \t Core");
    println!("------------------------------------------------------------------------------------------------------");
    let list = THREAD_LIST.lock();
    for tcb in list.iter() {
        println!("{}", tcb.get().unwrap());
    }
}

pub fn handle_unknown_syscall(cptr: usize, msg_info: usize, syscall: usize) {
    match syscall {
        seL4_SysDebugPutChar => console_putchar(cptr),
        seL4_SysDebugDumpScheduler => debug_dump_scheduler(),
        _ => todo!("unsupported unknown syscall {}", syscall as isize),
    }
}
