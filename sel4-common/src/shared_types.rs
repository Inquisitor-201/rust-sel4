use crate::invocation::Invocation;

#[repr(C)]
pub struct MessageInfo(usize);

impl MessageInfo {
    pub fn new(label: Invocation, caps_unwrapped: usize, extra_caps: usize, length: usize) -> Self {
        MessageInfo((label as usize) << 12 | caps_unwrapped << 9 | extra_caps << 7 | length)
    }
}
