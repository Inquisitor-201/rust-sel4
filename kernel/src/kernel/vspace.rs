use core::arch::asm;

use crate::{
    common::{PAGE_PTES, PAGE_SIZE, PTE_FLAG_BITS, PT_INDEX_BITS, KERNEL_ELF_BASE},
    get_level_pgbits,
    machine::{Paddr, Vaddr, Vregion},
    mask
};
use riscv::register::satp;
use sel4_common::{constants::seL4_PageBits, round_down, round_up, bit};
use spin::{Lazy, Mutex};

use super::{
    structures::{CapInfo, Capability},
    tcbVTable,
    thread::TCBInner,
};

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

    pub fn is_pte_pagetable(&self) -> bool {
        self.is_valid() && !(self.readable() || self.writable() || self.executable())
    }
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct PageTable {
    root: [PTE; PAGE_PTES],
}

pub static KERNEL_PT: Lazy<Mutex<PageTable>> = Lazy::new(|| {
    Mutex::new({
        PageTable {
            root: [PTE(0); PAGE_PTES],
        }
    })
});

impl PageTable {
    pub fn map_kernel_window(&mut self) {
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

    fn activate(&self, asid: usize) {
        assert!(asid <= 0xffff);
        unsafe {
            satp::write(asid << 44 | self.satp() as usize);
            asm!("sfence.vma");
        }
    }
}

pub fn activate_kernel_vspace() {
    KERNEL_PT.lock().activate(0);
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
pub fn riscv_get_n_paging(it_v_reg: Vregion) -> usize {
    let mut n = 0;
    for i in 0..2usize {
        n += get_n_paging(it_v_reg, get_level_pgbits!(i));
    }
    n
}

#[derive(Debug)]
pub struct LookupPTSlotRet {
    pt_slot: *mut PTE,
    pt_bits_left: usize,
}

fn lookup_ptslot(lvl1pt: *const PTE, vptr: Vaddr) -> LookupPTSlotRet {
    let mut level = 2;

    /* this is how many bits we potentially have left to decode. Initially we have the
     * full address space to decode, and every time we walk this will be reduced. The
     * final value of this after the walk is the size of the frame that can be inserted,
     * or already exists, in ret.ptSlot. The following formulation is an invariant of
     * the loop: */
    unsafe {
        let mut pt_bits_left = PT_INDEX_BITS * level + seL4_PageBits;
        let mut pt_slot = lvl1pt.add((vptr.0 >> pt_bits_left) & mask!(PT_INDEX_BITS)) as *mut PTE;
        while (*pt_slot).is_pte_pagetable() {
            level -= 1;
            pt_bits_left -= PT_INDEX_BITS;
            pt_slot = (*pt_slot).pa().as_raw_ptr_mut();
            pt_slot = pt_slot.add((vptr.0 >> pt_bits_left) & mask!(PT_INDEX_BITS));
        }
        LookupPTSlotRet {
            pt_slot,
            pt_bits_left,
        }
    }
}

/// 找到vspace_cap指向的pagetable，然后在页表中建立pt_cap保存的va->pa的映射
#[link_section = ".boot.text"]
fn map_it_pt_cap(vspace_cap: Capability, pt_cap: Capability) {
    let (pt_vptr, pt_pptr) = match pt_cap.get_info() {
        CapInfo::PageTableCap { vptr, pptr, .. } => (vptr, pptr),
        _ => panic!("invalid pt_cap"),
    };

    let root_pt = vspace_cap.get_pptr().0 as *mut PTE;

    /* Get PT slot to install the address in */
    let pt_ret = lookup_ptslot(root_pt, pt_vptr);
    let target_slot = pt_ret.pt_slot;

    unsafe {
        *target_slot = PTE::new(pt_pptr, PTEFlags::V | PTEFlags::U);
        asm!("sfence.vma");
    }
}

#[link_section = ".boot.text"]
fn map_it_frame_cap(vspace_cap: Capability, frame_cap: Capability) {
    let (pt_vptr, pt_pptr) = match frame_cap.get_info() {
        CapInfo::FrameCap { vptr, pptr } => (vptr, pptr),
        _ => panic!("invalid pt_cap"),
    };

    let root_pt = vspace_cap.get_pptr().as_raw_ptr_mut::<PTE>();

    /* Get PT slot to install the address in */
    let pt_ret = lookup_ptslot(root_pt, pt_vptr);
    assert!(pt_ret.pt_bits_left == seL4_PageBits, "{:#x?}", pt_ret);
    let target_slot = pt_ret.pt_slot;

    unsafe {
        *target_slot = PTE::new(
            pt_pptr,
            PTEFlags::R | PTEFlags::W | PTEFlags::X | PTEFlags::V | PTEFlags::U,
        );
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

pub const asidHighBits: usize = 7;
pub const asidLowBits: usize = 9;

#[link_section = ".boot.text"]
pub fn write_it_asid_pool(it_ap_cap: Capability, root_pt_cap: Capability) {
    // asid_pool_t *ap = ASID_POOL_PTR(pptr_of_cap(it_ap_cap));
    // ap->array[IT_ASID] = PTE_PTR(pptr_of_cap(root_pt_cap));
    // riscvKSASIDTable[IT_ASID >> asidLowBits] = ap;

    // todo: write it asid pool
}

pub fn set_vm_root(tcb: &TCBInner) {
    let thread_root_cap = tcb.tcb_cte_slot(tcbVTable).cap;
    match thread_root_cap.get_info() {
        CapInfo::PageTableCap { pptr, asid, .. } => unsafe {
            pptr.as_ref::<PageTable>().activate(asid)
        },
        _ => panic!("set_vm_root: thread_root_cap is not a PageTableCap"),
    }
}
