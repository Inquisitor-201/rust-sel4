use crate::{
    common::{Pregion, Vregion, KERNEL_ELF_BASE},
    machine::{Paddr, Vaddr},
    println,
};

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
    let boot_mem_reuse_reg = Pregion::new(KERNEL_ELF_BASE as Paddr, ki_boot_end as Paddr);
    let ui_reg = Pregion::new(ui_p_reg_start, ui_p_reg_end);
    let ui_v_reg = Vregion::new(ui_p_reg_start - pv_offset, ui_p_reg_end - pv_offset);
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
