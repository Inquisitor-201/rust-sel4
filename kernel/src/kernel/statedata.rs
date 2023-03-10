use spin::Mutex;

use crate::machine::Paddr;

use super::thread::ThreadPointer;

#[derive(Clone, Copy, Debug)]
pub enum SchedulerAction {
    ResumeCurrentThread,
    ChooseNewThread,
    SwitchToThread(ThreadPointer),
}

pub static ksSchedulerAction: Mutex<SchedulerAction> =
    Mutex::new(SchedulerAction::ResumeCurrentThread);
pub static ksCurThread: Mutex<ThreadPointer> = Mutex::new(ThreadPointer(Paddr(0)));
pub static ksIdleThread: Mutex<ThreadPointer> = Mutex::new(ThreadPointer(Paddr(0)));
