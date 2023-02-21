use alloc::vec::Vec;
use sel4_common::shared_types::{IPCBuffer, MessageInfo};
use spin::{Lazy, Mutex};

use crate::kernel::{cspace::lookup_slot, structures::CapSlot, thread::TCBInner};

pub static CUR_EXTRA_CAPS: Lazy<Mutex<Vec<&'static mut CapSlot>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn lookup_extra_caps(thread: &TCBInner, buffer: &IPCBuffer, info: MessageInfo) {
    let n_extra_caps = info.extra_caps();
    let mut cur_extra_caps = CUR_EXTRA_CAPS.lock();
    cur_extra_caps.clear();
    for i in 0..n_extra_caps {
        let cptr = buffer.caps_or_badges[i];
        let slot = lookup_slot(thread, cptr);
        cur_extra_caps.push(slot);
    }
}
