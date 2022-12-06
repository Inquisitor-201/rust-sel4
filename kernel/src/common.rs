use crate::machine::{Paddr, Vaddr};

pub const KERNEL_ELF_BASE: usize = 0x80400000;

#[derive(Debug)]
pub struct Pregion {
    pub start: Paddr,
    pub end: Paddr
}

#[derive(Debug)]
pub struct Vregion {
    pub start: Vaddr,
    pub end: Vaddr
}

impl Pregion {
    pub fn new(start: Paddr, end: Paddr) -> Self {
        Self { start, end }
    }
}

impl Vregion {
    pub fn new(start: Vaddr, end: Vaddr) -> Self {
        Self { start, end }
    }
}