use spin::{Lazy, Mutex};

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
    unsafe fn get_mut(&self) -> &'static mut Self {
        &mut *(self as *const _ as *mut Self)
    }
}

pub static ENV: Lazy<Mutex<Option<Env>>> = Lazy::new(|| Mutex::new(None));

pub fn get_env() -> &'static mut Env {
    match ENV.lock().as_ref() {
        Some(env) => unsafe { env.get_mut() },
        None => panic!(),
    }
}

pub fn get_bootinfo() -> &'static mut BootInfo {
    unsafe { &mut *(get_env().bootinfo as *mut BootInfo) }
}
