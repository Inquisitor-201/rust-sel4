use spin::{Lazy, Mutex};

use super::thread::TCBInner;

#[derive(Clone, Copy, Debug)]
pub enum SchedulerAction {
    ResumeCurrentThread,
    ChooseNewThread,
    SwitchToThread(&'static TCBInner),
}

pub static ksSchedulerAction: Lazy<Mutex<SchedulerAction>> =
    Lazy::new(|| Mutex::new(SchedulerAction::ResumeCurrentThread));
pub static ksCurThread: Lazy<Mutex<Option<&'static TCBInner>>> = Lazy::new(|| Mutex::new(None));
pub static ksIdleThread: Lazy<Mutex<Option<&'static TCBInner>>> = Lazy::new(|| Mutex::new(None));
