use riscv::addr::BitField;

use crate::{constants::{seL4_MsgMaxLength, seL4_MsgMaxExtraCaps}};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MessageInfo(pub usize);

impl MessageInfo {
    pub fn new(label: usize, caps_unwrapped: usize, extra_caps: usize, length: usize) -> Self {
        MessageInfo(label << 12 | caps_unwrapped << 9 | extra_caps << 7 | length)
    }
    pub fn label(&self) -> usize {
        self.0.get_bits(12..64)
    }
    pub fn caps_unwrapped(&self) -> usize {
        self.0.get_bits(9..12)
    }
    pub fn extra_caps(&self) -> usize {
        self.0.get_bits(7..9)
    }
    pub fn length(&self) -> usize {
        self.0.get_bits(0..7)
    }
}

#[repr(C)]
pub struct IPCBuffer {
    pub tag: MessageInfo,
    pub msg: [usize; seL4_MsgMaxLength],
    pub user_data: usize,
    pub caps_or_badges: [usize; seL4_MsgMaxExtraCaps],
    pub receive_cnode: usize,
    pub receive_index: usize,
    pub receive_depth: usize
}
