use super::{structures::seL4_CapInitThreadTCB, BootInfo};
pub struct Env {
    bootinfo: usize,
    it_ipc_buffer: usize,
    it_tcb_cptr: usize,
}

impl Env {
    pub fn new(bootinfo: *const BootInfo) -> Self {
        let it_ipc_buffer = unsafe { (*bootinfo).ipc_buffer };
        Self {
            bootinfo: bootinfo as _,
            it_ipc_buffer,
            it_tcb_cptr: seL4_CapInitThreadTCB,
        }
    }
}
