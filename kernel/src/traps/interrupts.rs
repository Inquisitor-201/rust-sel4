use crate::println;

#[no_mangle]
pub fn handle_interrupt() {
    println!("interrupt 0");
    loop {}
}