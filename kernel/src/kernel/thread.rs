use spin::{mutex::Mutex, Lazy};

use crate::{bit, common::{seL4_TCBBits, TCB_OFFSET}, machine::Paddr};

#[repr(C)]
#[repr(align(1024))]
pub struct TCB {
    data: [u8; bit!(seL4_TCBBits)],
}

impl TCB {
    pub fn new() -> Self {
        Self {
            data: [0; bit!(seL4_TCBBits)],
        }
    }
    pub fn tcb_pptr(&self) -> Paddr {
        Paddr(self as *const TCB as usize + TCB_OFFSET)
    }
}
pub static IDLE_THREAD_TCB: Lazy<Mutex<TCB>> = Lazy::new(|| Mutex::new(TCB::new()));
