use sel4_common::shared_types::MessageInfo;

use crate::{kernel::thread::ThreadPointer, machine::registerset::Rv64Reg};

pub fn reply_from_kernel_susccess_empty(thread: ThreadPointer) {
    let t = thread.get().unwrap();
    t.registers[Rv64Reg::a0 as usize] = 0;
    t.registers[Rv64Reg::a1 as usize] = MessageInfo::new(0, 0, 0, 0).0;
}