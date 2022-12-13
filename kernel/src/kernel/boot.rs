use core::fmt::Error;

use alloc::vec::Vec;
use riscv::register::sie;

use crate::{
    bit,
    common::{BI_FRAME_SIZE_BITS, KERNEL_ELF_BASE, PAGE_SIZE, USER_TOP},
    drivers::plic_init_hart,
    kernel::{map_kernel_window, KERNEL_PT},
    machine::{Paddr, Pregion, Vaddr, Vregion},
    println,
};

use super::{activate_kernel_vspace, heap::init_heap};

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
fn arch_init_freemem(ui_reg: Pregion, it_v_reg: Vregion) {
    let mut res_reg = Vec::new();
    extern "C" {
        fn ki_end();
    }
    let kernel_reg = Pregion::new(Paddr(KERNEL_ELF_BASE), Paddr(ki_end as u64));
    res_reg.push(ui_reg);
    println!("res_reg = {:#x?}", res_reg);
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
