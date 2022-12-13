use core::arch::asm;

use crate::{
    common::{KERNEL_ELF_BASE, PAGE_BITS, PAGE_PTES, PTE_FLAG_BITS},
    get_level_pgbits, get_level_pgsize,
    machine::{Paddr, Vaddr},
    println, round_down,
};
use riscv::register::satp;
use spin::{Lazy, Mutex};

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PTE(pub u64);

impl PTE {
    pub fn from_pa(pa: Paddr, flags: PTEFlags) -> Self {
        Self((pa.0 >> PAGE_BITS) << PTE_FLAG_BITS | flags.bits() as u64)
    }

    pub fn pa(&self) -> Paddr {
        Paddr((self.0 >> PTE_FLAG_BITS) << PAGE_BITS)
    }
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct KernelPagetable {
    root: [PTE; PAGE_PTES],
}

pub static KERNEL_PT: Lazy<Mutex<KernelPagetable>> = Lazy::new(|| {
    Mutex::new({
        KernelPagetable {
            root: [PTE(0); PAGE_PTES],
        }
    })
});

impl KernelPagetable {
    fn map_kernel_window(&mut self) {
        let pa = Paddr(round_down!(KERNEL_ELF_BASE, get_level_pgbits!(0)));
        let va = Vaddr(pa.0);

        // insert identical mapping
        self.root[va.pt_level_index(0)] = PTE::from_pa(
            pa,
            PTEFlags::R
                | PTEFlags::X
                | PTEFlags::W
                | PTEFlags::G
                | PTEFlags::A
                | PTEFlags::D
                | PTEFlags::V,
        );
    }

    pub fn satp(&self) -> u64 {
        let root_pa = self.root.as_ptr() as u64;
        8 << 60 | (root_pa >> PAGE_BITS)
    }

    fn activate(&self) {
        unsafe {
            satp::write(self.satp() as usize);
            asm!("sfence.vma");
        }
    }
}

pub fn activate_kernel_vspace() {
    KERNEL_PT.lock().activate();
}

pub fn map_kernel_window() {
    KERNEL_PT.lock().map_kernel_window();
}