use core::{cmp::min, fmt::Error};

use alloc::vec::Vec;
use riscv::register::sie;

use crate::{
    bit,
    common::*,
    drivers::plic_init_hart,
    kernel::{map_kernel_window, KERNEL_PT},
    machine::{Paddr, Pregion, Vaddr, Vregion},
    println,
};

use super::{activate_kernel_vspace, heap::init_heap, arch_get_n_paging};

#[link_section = ".boot.text"]
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
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

#[link_section = ".boot.text"]
fn init_freemem(
    reserved: &Vec<Pregion>,
    avail_reg: Pregion,
    it_v_reg: Vregion,
    extra_bi_size_bits: usize,
) -> bool {
    if !check_available_memory(avail_reg) {
        return false;
    }
    if !check_reserved_memory(reserved) {
        return false;
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
    println!("freemem = {:#x?}", freemem);

    /* now try to fit the root server objects into a region */
    let size = calculate_rootserver_size(it_v_reg, extra_bi_size_bits);
    // let max = rootserver_max_size_bits(extra_bi_size_bits);
    true
}

#[link_section = ".boot.text"]
fn arch_init_freemem(ui_reg: Pregion, it_v_reg: Vregion) -> Vec<Pregion> {
    let mut res_reg = Vec::new(); // reserved region
    extern "C" {
        fn ki_end();
    }
    let kernel_reg = Pregion::new(Paddr(KERNEL_ELF_BASE), Paddr(ki_end as u64));
    res_reg.push(kernel_reg);
    res_reg.push(ui_reg);

    let avail_reg = Pregion::new(Paddr(AVAIL_REGION_START), Paddr(AVAIL_REGION_END));
    init_freemem(&res_reg, avail_reg, it_v_reg, 0);
    res_reg
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

#[link_section = ".boot.text"]
fn try_init_kernel(
    ui_p_reg_start: Paddr,
    ui_p_reg_end: Paddr,
    pv_offset: Paddr,
    v_entry: Vaddr,
    dtb_addr_p: Paddr,
    dtb_size: u64,
) {
    extern "C" {
        fn ki_boot_end();
    }
    let boot_mem_reuse_reg = Pregion::new(Paddr(KERNEL_ELF_BASE), Paddr(ki_boot_end as u64));
    let ui_reg = Pregion::new(ui_p_reg_start, ui_p_reg_end);
    let ui_v_reg = Vregion::new(
        Vaddr(ui_p_reg_start.0 - pv_offset.0),
        Vaddr(ui_p_reg_end.0 - pv_offset.0),
    );

    let ipcbuf_vptr = ui_v_reg.end;
    let bi_frame_vptr = Vaddr(ipcbuf_vptr.0 + PAGE_SIZE as u64);
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

    arch_init_freemem(ui_reg, it_v_reg);
}

#[link_section = ".boot.text"]
#[no_mangle]
pub fn init_kernel(
    ui_p_reg_start: Paddr,
    ui_p_reg_end: Paddr,
    pv_offset: Paddr,
    v_entry: Vaddr,
    dtb_addr_p: Paddr,
    dtb_size: u64,
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
