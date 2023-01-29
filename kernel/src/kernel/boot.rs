use core::cmp::max;

use alloc::vec::Vec;
use riscv::register::sie;

use crate::{
    bit,
    common::*,
    drivers::plic_init_hart,
    get_level_pgbits, get_level_pgsize, is_aligned,
    kernel::map_kernel_window,
    machine::{clear_memory, Paddr, Pregion, Vaddr, Vregion},
    println, round_down,
};

use super::{
    activate_kernel_vspace, arch_get_n_paging, create_it_pt_cap, create_mapped_it_frame_cap,
    create_unmapped_it_frame_cap,
    heap::init_heap,
    structures::{
        seL4_CapBootInfoFrame, seL4_CapDomain, seL4_CapIRQControl, seL4_CapInitThreadCNode,
        seL4_CapInitThreadIPCBuffer, seL4_CapInitThreadVSpace, seL4_NumInitialCaps, Capability,
    },
    IT_ASID,
};

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
    size + arch_get_n_paging(it_v_reg) * bit!(seL4_PageTableBits)
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
        core::slice::from_raw_parts_mut(allocated.0 as *mut u8, n * bit!(size_bits)).fill(0);
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
    let n = arch_get_n_paging(it_v_reg);
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
fn arch_init_freemem(
    ui_reg: Pregion,
    it_v_reg: Vregion,
) -> (Vec<Pregion>, Vec<Pregion>, RootServer) {
    let mut res_reg = Vec::new(); // reserved region
    extern "C" {
        fn ki_end();
    }
    let kernel_reg = Pregion::new(Paddr(KERNEL_ELF_BASE), Paddr(ki_end as _));
    res_reg.push(kernel_reg);
    res_reg.push(ui_reg);

    let avail_reg = Pregion::new(Paddr(AVAIL_REGION_START), Paddr(AVAIL_REGION_END));
    let (rootserver, freemem) = init_freemem(&res_reg, avail_reg, it_v_reg, 0).unwrap();
    (res_reg, freemem, rootserver)
}

#[link_section = ".boot.text"]
fn init_cpu() {
    activate_kernel_vspace();
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
        slot_pos_cur: &mut usize,
    ) -> Capability {
        //  cap_t      lvl1pt_cap;
        //  vptr_t     pt_vptr;

        //  copyGlobalMappings(PTE_PTR(rootserver.vspace));

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
                println!("i = {}, pt_vptr = {:#x?}", i, pt_vptr);
                provide_cap(
                    root_cnode_cap,
                    create_it_pt_cap(root_pt_cap, self.it_alloc_paging().0, pt_vptr, IT_ASID),
                    slot_pos_cur,
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
        // bi->nodeID = node_id;
        // bi->numNodes = num_nodes;
        // bi->numIOPTLevels = 0;
        // bi->ipcBuffer = (seL4_IPCBuffer *)ipcbuf_vptr;
        // bi->initThreadCNodeSizeBits = CONFIG_ROOT_CNODE_SIZE_BITS;
        // bi->initThreadDomain = ksDomSchedule[ksDomScheduleIdx].domain;
        // bi->extraLen = extra_bi_size;
    }

    #[link_section = ".boot.text"]
    fn create_bi_frame_cap(&self, root_cnode_cap: Capability, pd_cap: Capability, vptr: Vaddr) {
        /* create a cap of it and write it into the root CNode */
        let cap = create_mapped_it_frame_cap(pd_cap, self.boot_info, vptr, IT_ASID, false);
        root_cnode_cap.cnode_write_slot_at(seL4_CapBootInfoFrame, cap);
    }

    #[link_section = ".boot.text"]
    fn create_ipcbuf_frame_cap(&self, root_cnode_cap: Capability, pd_cap: Capability, vptr: Vaddr) {
        clear_memory(self.ipc_buf, PAGE_SIZE);

        /* create a cap of it and write it into the root CNode */
        let cap = create_mapped_it_frame_cap(pd_cap, self.ipc_buf, vptr, IT_ASID, false);
        root_cnode_cap.cnode_write_slot_at(seL4_CapInitThreadIPCBuffer, cap);
    }

    #[link_section = ".boot.text"]
    fn create_frames_of_region(
        &self,
        root_cnode_cap: Capability,
        pd_cap: Capability,
        reg: Pregion,
        do_map: bool,
        pv_offset: usize,
        slot_pos_cur: &mut usize,
    ) {
        // pptr_t     f;
        // cap_t      frame_cap;
        // seL4_SlotPos slot_pos_before;
        // seL4_SlotPos slot_pos_after;

        // slot_pos_before = ndks_boot.slot_pos_cur;

        let mut pa = reg.start;
        while pa.0 < reg.end.0 {
            let frame_cap = if do_map {
                create_mapped_it_frame_cap(pd_cap, pa, pa.to_pa(pv_offset), IT_ASID, true)
            } else {
                create_unmapped_it_frame_cap(pa)
            };
            provide_cap(root_cnode_cap, frame_cap, slot_pos_cur);
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
}

#[link_section = ".boot.text"]
/// 向root_cnode_cap指向的cnode的空闲cap区域中填写一个cap
fn provide_cap(root_cnode_cap: Capability, cap: Capability, slot_pos_cur: &mut usize) {
    if *slot_pos_cur >= bit!(CONFIG_ROOT_CNODE_SIZE_BITS) {
        panic!(
            "ERROR: can't add another cap, all {} slots used\n",
            bit!(CONFIG_ROOT_CNODE_SIZE_BITS)
        );
    }
    root_cnode_cap.cnode_write_slot_at(*slot_pos_cur, cap);
    *slot_pos_cur += 1;
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
        ui_p_reg_start.to_pa(pv_offset),
        ui_p_reg_end.to_pa(pv_offset),
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

    let (res_reg, freemem, mut rootserver) = arch_init_freemem(ui_reg, it_v_reg);
    println!("rootserver = {:#x?}", rootserver);

    /* create the root cnode */
    let root_cnode_cap = rootserver.create_root_cnode();

    create_domain_cap(root_cnode_cap);
    init_irqs(root_cnode_cap);

    /* create the bootinfo frame */
    rootserver.populate_bi_frame(0, 1, ipcbuf_vptr, 0);

    let mut slot_pos_cur = seL4_NumInitialCaps;
    let root_pt_cap =
        rootserver.create_it_address_space(root_cnode_cap, it_v_reg, &mut slot_pos_cur);

    /* Create and map bootinfo frame cap */
    rootserver.create_bi_frame_cap(root_cnode_cap, root_pt_cap, bi_frame_vptr);
    rootserver.create_ipcbuf_frame_cap(root_cnode_cap, root_pt_cap, ipcbuf_vptr);
    rootserver.create_frames_of_region(
        root_cnode_cap,
        root_pt_cap,
        ui_reg,
        true,
        pv_offset,
        &mut slot_pos_cur,
    );

    root_cnode_cap.debug_print_cnode();
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
) -> ! {
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
    panic!()
}
