use core::mem::size_of;

use spin::{mutex::Mutex, Lazy};

use crate::{
    bit,
    common::{seL4_TCBBits, TCB_OFFSET},
    machine::{Paddr, Rv64Reg, SSTATUS_SPIE},
};

#[repr(C)]
pub struct TCBInner {
    registers: [usize; Rv64Reg::n_contextRegisters as _],
}

impl TCBInner {
    pub fn new_empty() -> Self {
        Self {
            registers: [0; Rv64Reg::n_contextRegisters as _],
        }
    }

    pub fn init_context(&mut self) {
        /* Enable supervisor interrupts (when going to user-mode) */
        self.registers[Rv64Reg::SSTATUS as usize] = SSTATUS_SPIE;
    }
}

#[repr(C)]
#[repr(align(1024))]
pub struct TCB {
    data: [u8; bit!(seL4_TCBBits)],
}

impl TCB {
    pub fn new() -> Self {
        assert!(size_of::<TCBInner>() <= bit!(seL4_TCBBits) - TCB_OFFSET);
        Self {
            data: [0; bit!(seL4_TCBBits)],
        }
    }
    pub fn pptr(&self) -> Paddr {
        Paddr(self as *const TCB as usize)
    }
    pub fn inner_pptr(&self) -> Paddr {
        Paddr(self as *const TCB as usize + TCB_OFFSET)
    }
    pub unsafe fn inner(&self) -> &TCBInner {
        self.inner_pptr().as_ref()
    }
    pub unsafe fn inner_mut(&self) -> &mut TCBInner {
        self.inner_pptr().as_mut()
    }
}

pub static IDLE_THREAD_TCB: Lazy<Mutex<TCB>> = Lazy::new(|| Mutex::new(TCB::new()));
