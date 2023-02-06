use spin::{Lazy, Mutex};

use super::thread::TCBInner;

#[allow(non_camel_case_types)]
pub static ksSchedulerAction: Lazy<Mutex<Option<&TCBInner>>> = Lazy::new(|| Mutex::new(None));
pub static ksCurThread: Lazy<Mutex<Option<&TCBInner>>> = Lazy::new(|| Mutex::new(None));
pub static ksIdleThread: Lazy<Mutex<Option<&TCBInner>>> = Lazy::new(|| Mutex::new(None));
