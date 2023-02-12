use crate::println;

#[no_mangle]
pub fn handle_syscall(cptr: usize, msg_info: usize, syscall: usize) {
    println!("syscall 0: {:#x?}, {:#x?}, {:#x?}", cptr, msg_info, syscall);
    loop {}
}

#[no_mangle]
pub fn handle_exception() {
    println!("exception 0");
    loop {}
}

#[no_mangle]
pub fn handle_interrupt() {
    println!("interrupt 0");
    loop {}
}