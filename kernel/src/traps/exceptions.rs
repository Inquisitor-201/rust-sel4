use crate::println;

#[no_mangle]
pub fn handle_exception() {
    println!("exception 0");
    loop {}
}