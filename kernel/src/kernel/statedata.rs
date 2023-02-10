use spin::{Lazy, Mutex};

use super::thread::TCBInner;

#[derive(Clone, Copy, Debug)]
pub enum SchedulerAction<'a> {
    ResumeCurrentThread,
    ChooseNewThread,
    SwitchToThread(&'a TCBInner),
}

pub static ksSchedulerAction: Lazy<Mutex<SchedulerAction>> =
    Lazy::new(|| Mutex::new(SchedulerAction::ResumeCurrentThread));
pub static ksCurThread: Lazy<Mutex<Option<&TCBInner>>> = Lazy::new(|| Mutex::new(None));
pub static ksIdleThread: Lazy<Mutex<Option<&TCBInner>>> = Lazy::new(|| Mutex::new(None));
