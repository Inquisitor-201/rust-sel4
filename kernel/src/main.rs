global_asm!(include_str!("entry.asm"));

// #[no_mangle]
pub fn main() {
    println!("Hello, world!");
}
