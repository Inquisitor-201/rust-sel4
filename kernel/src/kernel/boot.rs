use core::cmp::max;

use alloc::vec::Vec;
use riscv::register::{sie, stvec};
use sel4_common::{
    bit,
    bootinfo_common::{BootInfo, SlotRegion, UntypedDesc},
    constants::{
        seL4_ASIDPoolBits, seL4_PageBits, seL4_PageTableBits, seL4_SlotBits, seL4_TCBBits,
        seL4_VSpaceBits, BI_FRAME_SIZE_BITS, CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS,
    },
    round_down,
    structures_common::*,
};
use spin::{Lazy, Mutex};

use crate::{
    common::*,
    drivers::plic_init_hart,
    get_level_pgbits, get_level_pgsize, is_aligned,
    kernel::bootinfo::debug_print_bi_info,
    machine::{clear_memory, registerset::Rv64Reg, Paddr, Pregion, Vaddr, Vregion},
    max_free_index,
    object::cnode::{cte_insert, derive_cap},
    println,
};

use super::{
    heap::init_heap,
    statedata::{ksCurThread, ksIdleThread, ksSchedulerAction, SchedulerAction},
    structures::{CapSlot, Capability},
    thread::{activate_thread, schedule, ThreadPointer, ThreadState_Running, IDLE_THREAD_TCB, TCB},
    vspace::*,
};

struct BootState<'a> {
    slot_pos_cur: usize,
    reserved: Vec<Pregion>,
    freemem: Vec<Pregion>,
    bi_frame: Option<&'a mut BootInfo>,
}

static BOOT_STATE: Lazy<Mutex<BootState>> = Lazy::new(|| {
    Mutex::new(BootState {
        slot_pos_cur: 0,
        reserved: Vec::new(),
        freemem: Vec::new(),
        bi_frame: None,
    })
});

#[link_section = ".boot.text"]
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    clear_memory(Paddr(sbss as usize), ebss as usize - sbss as usize);
}

#[link_section = ".boot.text"]
fn check_available_memory(avail_reg: Pregion) -> bool {
    println!("available phys memory regions: {}", 1);
    println!("  {:#x?}", avail_reg);
    true
}

#[link_section = ".boot.text"]
fn check_reserved_memory(reserved: &Vec<Pregion>) -> bool {
    println!("reserved virt address space regions: {}", reserved.len());
    for i in 0..reserved.len() {
        println!("  {:#x?}", reserved[i]);
        if reserved[i].start.0 > reserved[i].end.0 {
            println!("ERROR: reserved region {:#x?} has start > end\n", i + 1);
            return false;
        }
        if i > 0 && reserved[i - 1].end.0 > reserved[i].start.0 {
            println!("ERROR: reserved region {:#x?} in wrong order\n", i + 1);
            return false;
        }
    }
    true
}

#[link_section = ".boot.text"]
fn merge_regions(regs: &Vec<Pregion>) -> Vec<Pregion> {
    let mut merged = Vec::<Pregion>::new();
    for r in regs {
        if merged.is_empty() || merged.last().unwrap().end.0 != r.start.0 {
            merged.push(*r);
        } else {
            merged.last_mut().unwrap().end = r.end;
        }
    }
    merged
}

#[link_section = ".boot.text"]
fn remove_empty_regions(regs: &Vec<Pregion>) -> Vec<Pregion> {
    let mut ret = Vec::<Pregion>::new();
    for r in regs {
        if !r.is_empty() {
            ret.push(*r);
        }
    }
    ret
}

#[link_section = ".boot.text"]
fn rootserver_max_size_bits(extra_bi_size_bits: usize) -> usize {
    max(
        max(seL4_VSpaceBits, CONFIG_ROOT_CNODE_SIZE_BITS + seL4_SlotBits),
        extra_bi_size_bits,
    )
}

#[link_section = ".boot.text"]
fn calculate_rootserver_size(it_v_reg: Vregion, extra_bi_size_bits: usize) -> usize {
    /* work out how much memory we need for root server objects */
    let mut size = bit!(CONFIG_ROOT_CNODE_SIZE_BITS + seL4_SlotBits);

    size += bit!(seL4_TCBBits); // root thread tcb
    size += bit!(seL4_PageBits); // ipc buf
    size += bit!(BI_FRAME_SIZE_BITS); // boot info
    size += bit!(seL4_ASIDPoolBits);

    size += if extra_bi_size_bits > 0 {
        bit!(extra_bi_size_bits)
    } else {
        0
    };
    size += bit!(seL4_VSpaceBits); // root vspace
                                   /* for all archs, seL4_PageTable Bits is the size of all non top-level paging structures */
    size + riscv_get_n_paging(it_v_reg) * bit!(seL4_PageTableBits)
}

/// 分配大小为extra_bi_size_bits的extra_bootinfo
#[link_section = ".boot.text"]
fn alloc_extra_bi(extra_bi_size_bits: usize) {
    if extra_bi_size_bits != 0 {
        unimplemented!();
    }
}

#[derive(Debug)]
pub struct RootServer {
    pub cnode: Paddr,
    pub vspace: Paddr,
    pub asid_pool: Paddr,
    pub ipc_buf: Paddr,
    pub boot_info: Paddr,
    pub extra_bi: Paddr,
    pub tcb: Paddr,
    pub paging: Pregion,
}

/// 分配n个rootserver对象，每个对象的大小为(1<<size_bits)
///
/// 返回
#[link_section = ".boot.text"]
fn alloc_rootserver_obj(rootserver_mem: &mut Pregion, size_bits: usize, n: usize) -> Paddr {
    assert!(is_aligned!(rootserver_mem.start.0, size_bits));
    let allocated = rootserver_mem.start;
    rootserver_mem.start.0 += n * bit!(size_bits);
    unsafe {
        core::slice::from_raw_parts_mut(allocated.as_raw_ptr_mut::<u8>(), n * bit!(size_bits))
            .fill(0);
    }
    allocated
}

/// 创建若干个rootserver对象，这些对象按从大到小的顺序分配
#[link_section = ".boot.text"]
fn create_rootserver_objects(
    start: usize,
    it_v_reg: Vregion,
    extra_bi_size_bits: usize,
    max: usize,
    size: usize,
) -> RootServer {
    let end = start + size;
    let mut rootserver_mem = Pregion::new(Paddr(start), Paddr(start + size));
    alloc_extra_bi(extra_bi_size_bits);

    // /* the root cnode is at least 4k, so it could be larger or smaller than a pd. */
    let cnode_size_bits = CONFIG_ROOT_CNODE_SIZE_BITS + seL4_SlotBits;
    let cnode = alloc_rootserver_obj(&mut rootserver_mem, cnode_size_bits, 1);
    let vspace = alloc_rootserver_obj(&mut rootserver_mem, seL4_VSpaceBits, 1);

    assert_eq!(seL4_ASIDPoolBits, seL4_PageBits);
    let asid_pool = alloc_rootserver_obj(&mut rootserver_mem, seL4_ASIDPoolBits, 1);
    let ipc_buf = alloc_rootserver_obj(&mut rootserver_mem, seL4_PageBits, 1);
    let boot_info = alloc_rootserver_obj(&mut rootserver_mem, BI_FRAME_SIZE_BITS, 1);

    // /* paging structures are 4k on every arch except aarch32 (1k) */
    let n = riscv_get_n_paging(it_v_reg);
    let paging_start = alloc_rootserver_obj(&mut rootserver_mem, seL4_PageTableBits, n);
    let paging_end = Paddr(paging_start.0 + n * bit!(seL4_PageTableBits));

    assert!(seL4_TCBBits <= seL4_PageTableBits);

    let tcb = alloc_rootserver_obj(&mut rootserver_mem, seL4_TCBBits, 1);

    assert_eq!(rootserver_mem.start.0, rootserver_mem.end.0);

    RootServer {
        cnode,
        vspace,
        asid_pool,
        ipc_buf,
        boot_info,
        extra_bi: Paddr(0),
        tcb,
        paging: Pregion::new(paging_start, paging_end),
    }
}

/// 初始化freemem，如果成功返回一个rootserver结构体 + freemem vector
///
/// rootserver struct记录rootserver各个对象的起始地址，freemem vector记录哪些空闲内存可用
#[link_section = ".boot.text"]
fn init_freemem(
    reserved: &Vec<Pregion>,
    avail_reg: Pregion,
    it_v_reg: Vregion,
    extra_bi_size_bits: usize,
) -> Option<(RootServer, Vec<Pregion>)> {
    if !check_available_memory(avail_reg) {
        return None;
    }
    if !check_reserved_memory(reserved) {
        return None;
    }
    let reserved = merge_regions(reserved);
    let mut freemem = Vec::<Pregion>::new();

    let mut a = avail_reg;
    for r in reserved.iter() {
        assert!(r.start.0 >= a.start.0 && r.end.0 <= a.end.0);
        freemem.push(Pregion::new(a.start, r.start));
        a.start = r.end;
    }
    freemem.push(a);
    freemem = remove_empty_regions(&freemem);

    // /* now try to fit the root server objects into a region */
    let size = calculate_rootserver_size(it_v_reg, extra_bi_size_bits);
    let max = rootserver_max_size_bits(extra_bi_size_bits);

    for i in (0..freemem.len()).rev() {
        /* Invariant: all non-empty regions are ordered, disjoint and unallocated. */

        /* Try to take the top-most suitably sized and aligned chunk. */
        let unaligned_start = freemem[i].end.0 - size;
        let start = round_down!(unaligned_start, max);

        /* if unaligned_start didn't underflow, and start fits in the region,
         * then we've found a region that fits the root server objects. */
        if unaligned_start <= freemem[i].end.0 && start >= freemem[i].start.0 {
            let rootserver =
                create_rootserver_objects(start, it_v_reg, extra_bi_size_bits, max, size);
            freemem.push(Pregion::new(Paddr(start + size), freemem[i].end));
            /* Leave the before leftover in current slot i. */
            freemem[i].end = Paddr(start);
            println!("final freemem = {:#x?}", freemem);
            /* Regions i and (i + 1) are now well defined, ordered, disjoint,
             * and unallocated, so we can return successfully. */
            return Some((rootserver, freemem));
        }
    }

    // /* We didn't find a big enough region. */
    println!(
        "ERROR: no free memory region is big enough for root server objects, need size/alignment of 2^{}",
        max
    );
    None
}

#[link_section = ".boot.text"]
fn riscv_init_freemem(ui_reg: Pregion, it_v_reg: Vregion) -> RootServer {
    let mut res_reg = Vec::new(); // reserved region
    extern "C" {
        fn ki_end();
    }
    let kernel_reg = Pregion::new(Paddr(KERNEL_ELF_BASE), Paddr(ki_end as _));
    res_reg.push(kernel_reg);
    res_reg.push(ui_reg);

    let avail_reg = Pregion::new(Paddr(AVAIL_REGION_START), Paddr(AVAIL_REGION_END));
    let (rootserver, freemem) = init_freemem(&res_reg, avail_reg, it_v_reg, 0).unwrap();

    let mut bs = BOOT_STATE.lock();
    bs.freemem = freemem;
    bs.reserved = Vec::from([avail_reg]);

    rootserver
}

#[link_section = ".boot.text"]
fn init_cpu() {
    activate_kernel_vspace();
    extern "C" {
        fn trap_entry();
    }
    unsafe { stvec::write(trap_entry as _, stvec::TrapMode::Direct) };
    init_local_irq_controller();
}

#[link_section = ".boot.text"]
fn init_local_irq_controller() {
    println!("Init local IRQ");

    /* Init per-hart PLIC */
    plic_init_hart();

    /* Enable timer and external interrupt. If SMP is enabled, then enable the
     * software interrupt also, it is used as IPI between cores. */
    unsafe {
        sie::set_stimer();
        sie::set_sext();
    }
}

// #[link_section = ".boot.text"]
// fn calculate_extra_bi_size_bits() -> usize {
//     0
// }

impl RootServer {
    #[link_section = ".boot.text"]
    fn create_root_cnode(&self) -> Capability {
        let cap = Capability::cap_cnode_cap_new(
            CONFIG_ROOT_CNODE_SIZE_BITS,             /* radix */
            WORD_BITS - CONFIG_ROOT_CNODE_SIZE_BITS, /* guard size */
            0,                                       /* guard */
            self.cnode.0 as _,
        ); /* pptr */

        /* write the root CNode cap into the root CNode */
        cap.cnode_write_slot_at(seL4_CapInitThreadCNode, cap);
        cap
    }

    #[link_section = ".boot.text"]
    /// 从rootserver的paging区域分配一个内存页
    fn it_alloc_paging(&mut self) -> Paddr {
        let allocated = self.paging.start;
        self.paging.start.0 += bit!(seL4_PageTableBits);
        assert!(self.paging.start.0 <= self.paging.end.0);
        return allocated;
    }

    #[link_section = ".boot.text"]
    /* Create an address space for the initial thread.
     * This includes page directory and page tables */
    pub fn create_it_address_space(
        &mut self,
        root_cnode_cap: Capability,
        it_v_reg: Vregion,
    ) -> Capability {
        //  cap_t      lvl1pt_cap;
        //  vptr_t     pt_vptr;

        unsafe { self.vspace.as_mut::<PageTable>().map_kernel_window() }

        let root_pt_cap = Capability::cap_page_table_cap_new(
            IT_ASID,            /* capPTMappedASID    */
            self.vspace.0 as _, /* capPTBasePtr       */
            true,               /* capPTIsMapped      */
            self.vspace.0 as _, /* capPTMappedAddress */
        );
        root_cnode_cap.cnode_write_slot_at(seL4_CapInitThreadVSpace, root_pt_cap);

        //  /* create all n level PT caps necessary to cover userland image in 4KiB pages */
        for i in 0..2usize {
            let mut pt_vptr = round_down!(it_v_reg.start.0, get_level_pgbits!(i));
            while pt_vptr < it_v_reg.end.0 {
                provide_cap(
                    root_cnode_cap,
                    create_it_pt_cap(root_pt_cap, self.it_alloc_paging().0, pt_vptr, IT_ASID),
                );
                pt_vptr += get_level_pgsize!(i);
            }
        }

        //  seL4_SlotPos slot_pos_after = ndks_boot.slot_pos_cur;
        //  ndks_boot.bi_frame->userImagePaging = (seL4_SlotRegion) {
        //      slot_pos_before, slot_pos_after
        //  };

        root_pt_cap
    }

    #[link_section = ".boot.text"]
    pub fn populate_bi_frame(
        &self,
        node_id: usize,
        num_nodes: usize,
        ipcbuf_vptr: Vaddr,
        extra_bi_size: usize,
    ) {
        /* clear boot info memory */
        clear_memory(self.boot_info, bit!(BI_FRAME_SIZE_BITS));
        if extra_bi_size != 0 {
            clear_memory(self.extra_bi, extra_bi_size);
        }

        /* initialise bootinfo-related global state */
        // seL4_BootInfo *bi = BI_PTR(rootserver.boot_info);
        let mut bs = BOOT_STATE.lock();
        let bi = unsafe { self.boot_info.as_mut::<BootInfo>() };
        bi.node_id = node_id;
        bi.num_nodes = num_nodes;
        bi.num_io_pt_levels = 0;
        bi.ipc_buffer = ipcbuf_vptr.0;
        bi.it_cnode_size_bits = CONFIG_ROOT_CNODE_SIZE_BITS;
        // bi->initThreadDomain = ksDomSchedule[ksDomScheduleIdx].domain;
        bi.extra_len = extra_bi_size;

        bs.slot_pos_cur = seL4_NumInitialCaps;
        bs.bi_frame = Some(bi);
    }

    #[link_section = ".boot.text"]
    fn create_bi_frame_cap(&self, root_cnode_cap: Capability, pd_cap: Capability, vptr: Vaddr) {
        /* create a cap of it and write it into the root CNode */
        let cap = create_mapped_it_frame_cap(pd_cap, self.boot_info, vptr, IT_ASID, false);
        root_cnode_cap.cnode_write_slot_at(seL4_CapBootInfoFrame, cap);
    }

    #[link_section = ".boot.text"]
    fn create_ipcbuf_frame_cap(
        &self,
        root_cnode_cap: Capability,
        pd_cap: Capability,
        vptr: Vaddr,
    ) -> Capability {
        clear_memory(self.ipc_buf, PAGE_SIZE);
        /* create a cap of it and write it into the root CNode */
        let cap = create_mapped_it_frame_cap(pd_cap, self.ipc_buf, vptr, IT_ASID, false);
        root_cnode_cap.cnode_write_slot_at(seL4_CapInitThreadIPCBuffer, cap);
        cap
    }

    #[link_section = ".boot.text"]
    fn create_frames_of_region(
        &self,
        root_cnode_cap: Capability,
        pd_cap: Capability,
        reg: Pregion,
        do_map: bool,
        pv_offset: usize,
    ) {
        // pptr_t     f;
        // cap_t      frame_cap;
        // seL4_SlotPos slot_pos_before;
        // seL4_SlotPos slot_pos_after;

        let mut pa = reg.start;
        while pa.0 < reg.end.0 {
            let frame_cap = if do_map {
                create_mapped_it_frame_cap(pd_cap, pa, pa.to_va(pv_offset), IT_ASID, true)
            } else {
                create_unmapped_it_frame_cap(pa)
            };
            provide_cap(root_cnode_cap, frame_cap);
            pa.0 += PAGE_SIZE;
        }

        // slot_pos_after = ndks_boot.slot_pos_cur;

        // return (create_frames_of_region_ret_t) {
        //     .region = (seL4_SlotRegion) {
        //         .start = slot_pos_before,
        //         .end   = slot_pos_after
        //     },
        //     .success = true
        // };
    }

    #[link_section = ".boot.text"]
    pub fn create_it_asid_pool(&self, root_cnode_cap: Capability) -> Capability {
        let ap_cap = Capability::cap_asid_pool_cap_new(IT_ASID >> asidLowBits, self.asid_pool.0);
        root_cnode_cap.cnode_write_slot_at(seL4_CapInitThreadASIDPool, ap_cap);

        /* create ASID control cap */
        root_cnode_cap
            .cnode_write_slot_at(seL4_CapASIDControl, Capability::cap_asid_control_cap_new());
        ap_cap
    }

    #[link_section = ".boot.text"]
    pub fn create_initial_thread(
        &self,
        root_cnode_cap: Capability,
        it_pd_cap: Capability,
        ui_v_entry: Vaddr,
        bi_frame_vptr: Vaddr,
        ipcbuf_vptr: Vaddr,
        ipcbuf_cap: Capability,
    ) -> ThreadPointer {
        let tcb_inner = unsafe { self.tcb.as_ref::<TCB>().inner_mut() };

        tcb_inner.init_context();

        // todo: derive ipc buffer cap
        let dc_cap = derive_cap(ipcbuf_cap);

        cte_insert(
            root_cnode_cap,
            &root_cnode_cap.cnode_slot_at(seL4_CapInitThreadCNode),
            CapSlot::slot_ref(self.tcb, tcbCTable),
        );

        cte_insert(
            it_pd_cap,
            &root_cnode_cap.cnode_slot_at(seL4_CapInitThreadVSpace),
            CapSlot::slot_ref(self.tcb, tcbVTable),
        );

        cte_insert(
            dc_cap,
            &root_cnode_cap.cnode_slot_at(seL4_CapInitThreadIPCBuffer),
            CapSlot::slot_ref(self.tcb, tcbBuffer),
        );
        // todo: cte insert ipc_buf cap
        // todo: set tcbIPCBuffer, tcbMCP, tcbDomain
        tcb_inner.registers[Rv64Reg::a0 as usize] = bi_frame_vptr.0;
        tcb_inner.registers[Rv64Reg::NextIP as usize] = ui_v_entry.0;
        tcb_inner.tcb_priority = seL4_MaxPrio;
        tcb_inner.tcb_ipc_buffer = ipcbuf_vptr;
        tcb_inner.set_thread_state(ThreadState_Running);
        // todo: set Cur_domain

        /* create initial thread's TCB cap */
        let cap = Capability::cap_thread_cap_new(tcb_inner as *mut _ as _);
        root_cnode_cap.cnode_write_slot_at(seL4_CapInitThreadTCB, cap);

        tcb_inner.pointer()
    }

    #[link_section = ".boot.text"]
    fn create_untypeds_for_region(
        &self,
        root_cnode_cap: Capability,
        device_memory: bool,
        mut reg: Pregion,
        first_untyped_slot: usize,
    ) -> bool {
        while !reg.is_empty() {
            let mut size_bits =
                usize::BITS as usize - 1 - (reg.end.0 - reg.start.0).leading_zeros() as usize;
            size_bits = size_bits.min(seL4_MaxUntypedBits);
            if reg.start.0 != 0 {
                size_bits = size_bits.min(reg.start.0.trailing_zeros() as _);
            }
            if size_bits >= seL4_MinUntypedBits {
                if !provide_untyped_cap(
                    root_cnode_cap,
                    device_memory,
                    reg.start,
                    size_bits,
                    first_untyped_slot,
                ) {
                    return false;
                }
            }
            reg.start.0 += bit!(size_bits);
        }
        true
    }

    #[link_section = ".boot.text"]
    pub fn create_untypeds(&self, root_cnode_cap: Capability, boot_mem_reuse_reg: Pregion) -> bool {
        let bs = BOOT_STATE.lock();
        let first_untyped_slot = bs.slot_pos_cur;
        let mut start = 0;
        let reserved = bs.reserved.clone();
        let freemem = bs.freemem.clone();
        drop(bs);
        for res in reserved.iter() {
            if start < res.start.0 {
                let reg = Pregion::new(Paddr(start), res.start);
                if !self.create_untypeds_for_region(root_cnode_cap, true, reg, first_untyped_slot) {
                    println!(
                        "ERROR: creation of untypeds for device region {:#x?} failed\n",
                        reg
                    );
                    return false;
                }
            }
            start = res.end.0;
        }
        if start < CONFIG_PADDR_USER_DEVICE_TOP {
            let reg = Pregion::new(Paddr(start), Paddr(CONFIG_PADDR_USER_DEVICE_TOP));
            if !self.create_untypeds_for_region(root_cnode_cap, true, reg, first_untyped_slot) {
                println!(
                    "ERROR: creation of untypeds for top device region {:#x?} failed\n",
                    reg
                );
                return false;
            }
        }
        if !self.create_untypeds_for_region(
            root_cnode_cap,
            false,
            boot_mem_reuse_reg,
            first_untyped_slot,
        ) {
            println!(
                "ERROR: creation of untypeds for recycled boot memory {:#x?} failed\n",
                boot_mem_reuse_reg
            );
            return false;
        }
        /* convert remaining freemem into UT objects and provide the caps */
        for &reg in freemem.iter() {
            if !self.create_untypeds_for_region(root_cnode_cap, false, reg, first_untyped_slot) {
                println!(
                    "ERROR: creation of untypeds for free memory region {:#x?} failed\n",
                    reg
                );
                return false;
            }
        }
        let mut bs = BOOT_STATE.lock();
        bs.bi_frame.as_mut().unwrap().untyped = SlotRegion {
            start: first_untyped_slot,
            end: bs.slot_pos_cur,
        };
        true
    }
}

#[link_section = ".boot.text"]
/// 向root_cnode_cap指向的cnode的空闲cap区域中填写一个cap
fn provide_cap(root_cnode_cap: Capability, cap: Capability) -> bool {
    let mut bs = BOOT_STATE.lock();
    if bs.slot_pos_cur >= bit!(CONFIG_ROOT_CNODE_SIZE_BITS) {
        println!(
            "ERROR: can't add another cap, all {} slots used\n",
            bit!(CONFIG_ROOT_CNODE_SIZE_BITS)
        );
        return false;
    }
    root_cnode_cap.cnode_write_slot_at(bs.slot_pos_cur, cap);
    bs.slot_pos_cur += 1;
    true
}

#[link_section = ".boot.text"]
/// 向root_cnode_cap指向的cnode的空闲cap区域中填写一个cap
fn provide_untyped_cap(
    root_cnode_cap: Capability,
    device_memory: bool,
    pptr: Paddr,
    size_bits: usize,
    first_untyped_slot: usize,
) -> bool {
    let mut bs = BOOT_STATE.lock();
    let i = bs.slot_pos_cur - first_untyped_slot;
    if i < CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS {
        let bi_frame = bs.bi_frame.as_mut().unwrap();
        bi_frame.untyped_list[i] = UntypedDesc {
            paddr: pptr.0,
            _padding: [0; 6],
            size_bits: size_bits as _,
            is_device: device_memory as _,
        };
        let ut_cap = Capability::cap_untyped_cap_new(
            max_free_index!(size_bits),
            device_memory,
            size_bits,
            pptr.0,
        );
        drop(bs);
        return provide_cap(root_cnode_cap, ut_cap);
    }
    println!("Kernel init: Too many untyped regions for boot info\n");
    true
}

#[link_section = ".boot.text"]
fn create_domain_cap(root_cnode_cap: Capability) {
    let cap = Capability::cap_domain_cap_new();
    root_cnode_cap.cnode_write_slot_at(seL4_CapDomain, cap);
}

#[link_section = ".boot.text"]
fn init_irqs(root_cnode_cap: Capability) {
    let cap = Capability::cap_irq_control_cap_new();
    root_cnode_cap.cnode_write_slot_at(seL4_CapIRQControl, cap);
}

#[link_section = ".boot.text"]
pub fn create_idle_thread() -> bool {
    let idle = IDLE_THREAD_TCB.lock();
    let pptr = idle.pptr();
    unsafe {
        *(ksIdleThread.lock()) = ThreadPointer(pptr);
    }
    //         configureIdleThread(NODE_STATE_ON_CORE(ksIdleThread, i));
    true
}

#[link_section = ".boot.text"]
pub fn init_core_state(scheduler_action: ThreadPointer) {
    *(ksSchedulerAction.lock()) = SchedulerAction::SwitchToThread(scheduler_action);
    *(ksCurThread.lock()) = *(ksIdleThread.lock());
}

#[link_section = ".boot.text"]
pub fn bi_finalise() {
    let mut bs = BOOT_STATE.lock();
    bs.bi_frame.as_mut().unwrap().empty =
        SlotRegion::new(bs.slot_pos_cur, bit!(CONFIG_ROOT_CNODE_SIZE_BITS));
}

#[link_section = ".boot.text"]
fn try_init_kernel(
    ui_p_reg_start: Paddr,
    ui_p_reg_end: Paddr,
    pv_offset: usize,
    v_entry: Vaddr,
    dtb_addr_p: Paddr,
    dtb_size: usize,
) {
    extern "C" {
        fn ki_boot_end();
    }
    let boot_mem_reuse_reg = Pregion::new(Paddr(KERNEL_ELF_BASE), Paddr(ki_boot_end as _));
    let ui_reg = Pregion::new(ui_p_reg_start, ui_p_reg_end);
    let ui_v_reg = Vregion::new(
        ui_p_reg_start.to_va(pv_offset),
        ui_p_reg_end.to_va(pv_offset),
    );

    let ipcbuf_vptr = ui_v_reg.end;
    let bi_frame_vptr = Vaddr(ipcbuf_vptr.0 + PAGE_SIZE);
    let extra_bi_frame_vptr = Vaddr(bi_frame_vptr.0 + bit!(BI_FRAME_SIZE_BITS));

    map_kernel_window();
    init_cpu();
    println!("Bootstrapping kernel");
    // let extra_bi_size_bits = calculate_extra_bi_size_bits();

    /* init thread virt region */
    let it_v_reg = Vregion::new(ui_v_reg.start, extra_bi_frame_vptr);
    if it_v_reg.end.0 >= USER_TOP {
        panic!(
            "ERROR: userland image virt region exceeds USER_TOP({:#x?})",
            USER_TOP
        );
    }

    let mut rootserver = riscv_init_freemem(ui_reg, it_v_reg);
    println!("rootserver = {:#x?}", rootserver);

    /* create the root cnode */
    let root_cnode_cap = rootserver.create_root_cnode();

    create_domain_cap(root_cnode_cap);
    init_irqs(root_cnode_cap);

    /* create the bootinfo frame */
    rootserver.populate_bi_frame(0, 1, ipcbuf_vptr, 0);

    let root_pt_cap = rootserver.create_it_address_space(root_cnode_cap, it_v_reg);

    /* Create and map bootinfo frame cap */
    rootserver.create_bi_frame_cap(root_cnode_cap, root_pt_cap, bi_frame_vptr);
    let ipcbuf_cap = rootserver.create_ipcbuf_frame_cap(root_cnode_cap, root_pt_cap, ipcbuf_vptr);
    rootserver.create_frames_of_region(root_cnode_cap, root_pt_cap, ui_reg, true, pv_offset);
    let it_ap_cap = rootserver.create_it_asid_pool(root_cnode_cap);
    // write_it_asid_pool(it_ap_cap, root_pt_cap);
    create_idle_thread();
    let initial = rootserver.create_initial_thread(
        root_cnode_cap,
        root_pt_cap,
        v_entry,
        bi_frame_vptr,
        ipcbuf_vptr,
        ipcbuf_cap,
    );
    init_core_state(initial);
    if !rootserver.create_untypeds(root_cnode_cap, boot_mem_reuse_reg) {
        panic!("create_untypeds failed");
    }

    /* finalise the bootinfo frame */
    bi_finalise();

    debug_print_bi_info(BOOT_STATE.lock().bi_frame.as_ref().unwrap());
    root_cnode_cap.debug_print_cnode();
    println!("Booting all finished, dropped to user space");
}

#[link_section = ".boot.text"]
#[no_mangle]
pub fn init_kernel(
    ui_p_reg_start: Paddr,
    ui_p_reg_end: Paddr,
    pv_offset: usize,
    v_entry: Vaddr,
    dtb_addr_p: Paddr,
    dtb_size: usize,
) {
    clear_bss();
    init_heap();
    let result = try_init_kernel(
        ui_p_reg_start,
        ui_p_reg_end,
        pv_offset,
        v_entry,
        dtb_addr_p,
        dtb_size,
    );

    schedule();
    activate_thread();
}
