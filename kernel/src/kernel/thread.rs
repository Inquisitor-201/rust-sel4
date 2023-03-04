use core::{mem::size_of, ptr};

use sel4_common::{bit, constants::seL4_TCBBits, round_down};
use spin::{mutex::Mutex, Lazy};

use crate::{
    common::{seL4_MinPrio, TCB_OFFSET},
    kernel::statedata::ksIdleThread,
    machine::{
        registerset::{Rv64Reg, SSTATUS_SPIE},
        Paddr, Vaddr,
    }, println,
};

use super::{
    statedata::{ksCurThread, ksSchedulerAction, SchedulerAction},
    structures::CapSlot,
    vspace::set_vm_root,
};

pub const ThreadState_Inactive: u8 = 0;
pub const ThreadState_Running: u8 = 1;
pub const ThreadState_Restart: u8 = 2;
pub const ThreadState_BlockedOnReceive: u8 = 3;
pub const ThreadState_BlockedOnSend: u8 = 3;
pub const ThreadState_BlockedOnReply: u8 = 4;
pub const ThreadState_BlockedOnNotification: u8 = 5;

#[repr(C)]
#[derive(Debug)]
pub struct ThreadState {
    ts_type: u8,
}

#[repr(C)]
#[derive(Debug)]
pub struct TCBInner {
    pub registers: [usize; Rv64Reg::n_contextRegisters as _],
    pub tcb_state: ThreadState,
    pub tcb_priority: usize,
    pub tcb_ipc_buffer: Vaddr
}

impl TCBInner {
    pub fn new_empty() -> Self {
        Self {
            registers: [0; Rv64Reg::n_contextRegisters as _],
            tcb_state: ThreadState {
                ts_type: ThreadState_Inactive,
            },
            tcb_priority: seL4_MinPrio,
            tcb_ipc_buffer: Vaddr(0)
        }
    }

    pub fn init_context(&mut self) {
        /* Enable supervisor interrupts (when going to user-mode) */
        self.registers[Rv64Reg::SSTATUS as usize] = SSTATUS_SPIE;
    }

    pub fn is_runnable(&self) -> bool {
        // self.tcb_state.ts_type
        self.tcb_state.ts_type == ThreadState_Running
            || self.tcb_state.ts_type == ThreadState_Restart
    }

    pub fn ptr_eq(&self, another: &TCBInner) -> bool {
        ptr::eq(self as *const _, another as *const _)
    }

    pub fn schedule_tcb(&self) {
        let cur_thread = *(ksCurThread.lock());
        if cur_thread.is_none() {
            return;
        }
        let action = *(ksSchedulerAction.lock());
        if self.ptr_eq(cur_thread.unwrap()) && !self.is_runnable() {
            if let SchedulerAction::ResumeCurrentThread = action {
                panic!("reschedule required");
            }
        }
    }

    pub fn set_thread_state(&mut self, ts: u8) {
        self.tcb_state.ts_type = ts;
        self.schedule_tcb();
    }

    pub fn tcb_cte_slot(&self, index: usize) -> &mut CapSlot {
        let ctable = round_down!(self as *const _ as usize, seL4_TCBBits) as *mut CapSlot;
        unsafe { ctable.add(index).as_mut().unwrap() }
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

pub fn schedule() {
    let action = *(ksSchedulerAction.lock());
    match action {
        SchedulerAction::ResumeCurrentThread => {}
        _ => {
            // let was_runnable;
            let cur_thread = ksCurThread.lock().unwrap();
            let was_runnable = if cur_thread.is_runnable() {
                todo!("SCHED_ENQUEUE_CURRENT_TCB");
                true
            } else {
                false
            };
            if let SchedulerAction::ChooseNewThread = action {
                todo!("scheduleChooseNewThread");
            } else if let SchedulerAction::SwitchToThread(candidate) = action {
                assert!(candidate.is_runnable());
                /* Avoid checking bitmap when ksCurThread is higher prio, to
                 * match fast path.
                 * Don't look at ksCurThread prio when it's idle, to respect
                 * information flow in non-fastpath cases. */
                let fastfail = cur_thread.ptr_eq(ksIdleThread.lock().unwrap())
                    && candidate.tcb_priority < cur_thread.tcb_priority;
                if fastfail {
                    todo!("scheduleChooseNewThread");
                } else {
                    assert!(!candidate.ptr_eq(cur_thread));
                    switch_to_thread(candidate);
                }
            } else {
                panic!("schedule(): Should not reach here");
            }
        }
    }
    *(ksSchedulerAction.lock()) = SchedulerAction::ResumeCurrentThread;
}

pub fn switch_to_thread(tcb: &'static TCBInner) {
    set_vm_root(tcb);
    *(ksCurThread.lock()) = Some(tcb);
}

pub fn activate_thread() {
    let cur_thread = ksCurThread.lock().unwrap();
    match cur_thread.tcb_state.ts_type {
        ThreadState_Running => {}
        ThreadState_Restart => todo!("thread restart"),
        _ => todo!("activate_thread, type = {}", cur_thread.tcb_state.ts_type),
    }
}
