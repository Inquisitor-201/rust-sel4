mod io;
mod sbi;

use core::fmt::{self, Debug, Formatter};
pub use io::*;
pub use sbi::*;

use crate::{
    common::{seL4_PageBits, PAGE_SIZE},
    is_aligned,
    kernel::PTE,
    mask,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Paddr(pub usize);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vaddr(pub usize);

// paddr

impl Paddr {
    pub fn get_page_bytes(&self) -> &'static [u8] {
        assert!(is_aligned!(self.0, seL4_PageBits));
        unsafe { core::slice::from_raw_parts(self.0 as *const u8, PAGE_SIZE) }
    }

    pub fn get_page_pte_array(&self) -> &'static [PTE] {
        assert!(is_aligned!(self.0, seL4_PageBits));
        unsafe {
            core::slice::from_raw_parts(
                self.0 as *const PTE,
                PAGE_SIZE / core::mem::size_of::<PTE>(),
            )
        }
    }

    pub fn to_pa(&self, pv_offset: usize) -> Vaddr {
        Vaddr(self.0 - pv_offset)
    }
}

// vaddr

impl Vaddr {
    pub fn pt_level_index(&self, level: usize) -> usize {
        (self.0 >> (seL4_PageBits + (2 - level) * 9)) & mask!(9)
    }
    pub fn to_pa(&self, pv_offset: usize) -> Paddr {
        Paddr(self.0 + pv_offset)
    }
}

#[derive(Clone, Copy)]
pub struct Pregion {
    pub start: Paddr,
    pub end: Paddr,
}

#[derive(Clone, Copy)]
pub struct Vregion {
    pub start: Vaddr,
    pub end: Vaddr,
}

impl Pregion {
    pub fn new(start: Paddr, end: Paddr) -> Self {
        Self { start, end }
    }
    pub fn is_empty(&self) -> bool {
        self.start.0 == self.end.0
    }
}

impl Vregion {
    pub fn new(start: Vaddr, end: Vaddr) -> Self {
        Self { start, end }
    }
}

impl Debug for Vaddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Vaddr({:#x})", self.0))
    }
}

impl Debug for Paddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Paddr({:#x})", self.0))
    }
}

impl Debug for Vregion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Vregion[{:#x?}..{:#x?}]",
            self.start.0,
            self.end.0 - 1
        ))
    }
}

impl Debug for Pregion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Pregion[{:#x?}..{:#x?}]",
            self.start.0,
            self.end.0 - 1
        ))
    }
}

#[macro_export]
macro_rules! get_level_pgbits {
    ($lvl: expr) => {
        $crate::common::PT_INDEX_BITS * (2 - $lvl) + $crate::common::seL4_PageBits
    };
}

#[macro_export]
macro_rules! get_level_pgsize {
    ($lvl: expr) => {
        $crate::bit!($crate::get_level_pgbits!($lvl))
    };
}

pub fn clear_memory(pa: Paddr, len: usize) {
    unsafe {
        core::slice::from_raw_parts_mut(pa.0 as *mut u8, len).fill(0);
    }
}