use core::arch::asm;

use crate::{
    bit,
    common::{seL4_PageBits, KERNEL_ELF_BASE, PAGE_PTES, PAGE_SIZE, PTE_FLAG_BITS, PT_INDEX_BITS},
    get_level_pgbits,
    machine::{Paddr, Vaddr, Vregion},
    mask, round_down, round_up,
};
use riscv::register::satp;
use spin::{Lazy, Mutex};

use super::structures::{CapInfo, Capability};

pub const ASID_INVALID: usize = 0;
pub const IT_ASID: usize = 1;

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
pub struct PTE(pub usize);

impl PTE {
    pub fn new(pa: Paddr, flags: PTEFlags) -> Self {
        Self((pa.0 >> seL4_PageBits) << PTE_FLAG_BITS | flags.bits() as usize)
    }

    pub fn pa(&self) -> Paddr {
        Paddr((self.0 >> PTE_FLAG_BITS) << seL4_PageBits)
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.0 as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.flags() & PTEFlags::V != PTEFlags::empty()
    }

    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }

    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }

    pub fn is_pte_pagetalbe(&self) -> bool {
        self.is_valid() && !(self.readable() || self.writable() || self.executable())
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
        self.root[va.pt_level_index(0)] = PTE::new(
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

    pub fn satp(&self) -> usize {
        let root_pa = self.root.as_ptr();
        8 << 60 | (root_pa as usize >> seL4_PageBits)
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

#[link_section = ".boot.text"]
fn get_n_paging(v_reg: Vregion, bits: usize) -> usize {
    let start = round_down!(v_reg.start.0, bits);
    let end = round_up!(v_reg.end.0, bits);
    (end - start) as usize / bit!(bits)
}

#[link_section = ".boot.text"]
pub fn arch_get_n_paging(it_v_reg: Vregion) -> usize {
    let mut n = 0;
    for i in 0..2usize {
        n += get_n_paging(it_v_reg, get_level_pgbits!(i));
    }
    n
}

pub struct LookupPTSlotRet {
    ptSlot: *mut PTE,
    ptBitsLeft: usize,
}

fn lookupPTSlot(lvl1pt: *const PTE, vptr: Vaddr) -> LookupPTSlotRet {
    let mut level = 2;

    /* this is how many bits we potentially have left to decode. Initially we have the
     * full address space to decode, and every time we walk this will be reduced. The
     * final value of this after the walk is the size of the frame that can be inserted,
     * or already exists, in ret.ptSlot. The following formulation is an invariant of
     * the loop: */
    unsafe {
        let mut ptBitsLeft = PT_INDEX_BITS * level + seL4_PageBits;
        let mut ptSlot = lvl1pt.add((vptr.0 >> ptBitsLeft) & mask!(PT_INDEX_BITS)) as *mut PTE;

        while (*ptSlot).is_pte_pagetalbe() && level > 0 {
            level -= 1;
            ptBitsLeft -= PT_INDEX_BITS;
            ptSlot = (*ptSlot).pa().0 as *mut PTE;
            ptSlot = ptSlot.add((vptr.0 >> ptBitsLeft) & mask!(PT_INDEX_BITS));
        }
        LookupPTSlotRet { ptSlot, ptBitsLeft }
    }
}

/// 找到vspace_cap指向的pagetable，然后在页表中建立pt_cap保存的va->pa的映射
#[link_section = ".boot.text"]
fn map_it_pt_cap(vspace_cap: Capability, pt_cap: Capability) {
    let (pt_vptr, pt_pptr) = match pt_cap.get_info() {
        CapInfo::PageTableCap { vptr, pptr } => (vptr, pptr),
        _ => panic!("invalid pt_cap"),
    };

    let root_pt = vspace_cap.get_pptr().0 as *mut PTE;

    /* Get PT slot to install the address in */
    let pt_ret = lookupPTSlot(root_pt, pt_vptr);
    let target_slot = pt_ret.ptSlot;

    unsafe {
        *target_slot = PTE::new(pt_pptr, PTEFlags::V);
        asm!("sfence.vma");
    }
}

#[link_section = ".boot.text"]
fn map_it_frame_cap(vspace_cap: Capability, frame_cap: Capability) {
    let (pt_vptr, pt_pptr) = match frame_cap.get_info() {
        CapInfo::FrameCap { vptr, pptr } => (vptr, pptr),
        _ => panic!("invalid pt_cap"),
    };

    let root_pt = vspace_cap.get_pptr().0 as *mut PTE;

    /* Get PT slot to install the address in */
    let pt_ret = lookupPTSlot(root_pt, pt_vptr);
    assert!(pt_ret.ptBitsLeft == seL4_PageBits);
    let target_slot = pt_ret.ptSlot;

    unsafe {
        *target_slot = PTE::new(pt_pptr, PTEFlags::R | PTEFlags::W | PTEFlags::V);
        asm!("sfence.vma");
    }
}

#[link_section = ".boot.text"]
pub fn create_it_pt_cap(
    vspace_cap: Capability,
    pptr: usize,
    vptr: usize,
    asid: usize,
) -> Capability {
    let cap;
    cap = Capability::cap_page_table_cap_new(
        asid, /* capPTMappedASID      */
        pptr, /* capPTBasePtr         */
        true, /* capPTIsMapped        */
        vptr, /* capPTMappedAddress   */
    );

    map_it_pt_cap(vspace_cap, cap);
    cap
}

enum VmRights {
    VMKernelOnly = 1,
    VMReadOnly = 2,
    VMReadWrite = 3,
}

#[link_section = ".boot.text"]
pub fn create_mapped_it_frame_cap(
    pd_cap: Capability,
    pptr: Paddr,
    vptr: Vaddr,
    asid: usize,
    executable: bool,
) -> Capability {
    let cap = Capability::cap_frame_cap_new(
        asid,                       /* capFMappedASID    */
        pptr.0,                     /* capFBasePtr       */
        PAGE_SIZE,                  /* capFSize          */
        VmRights::VMReadWrite as _, /* capFVMRights      */
        false,                      /* capFIsDevice      */
        vptr.0,                     /* capFMappedAddress */
    );

    map_it_frame_cap(pd_cap, cap);
    cap
}

#[link_section = ".boot.text"]
pub fn create_unmapped_it_frame_cap(pptr: Paddr) -> Capability {
    Capability::cap_frame_cap_new(
        ASID_INVALID, /* capFMappedASID       */
        pptr.0,       /* capFBasePtr          */
        0,            /* capFSize             */
        0,            /* capFVMRights         */
        false,
        0, /* capFMappedAddress    */
    )
}
