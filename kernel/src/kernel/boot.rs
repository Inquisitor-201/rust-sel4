use crate::{machine::{Paddr, Vptr}, println};

#[no_mangle]
pub fn init_kernel(
    ui_p_reg_start: Paddr,
    ui_p_reg_end: Paddr,
    pv_offset: Paddr,
    v_entry: Vptr,
    dtb_addr_p: Paddr,
    dtb_size: u32,
) -> ! {
    println!("ui_p_reg_start = {:#x?}", ui_p_reg_start);
    // result = try_init_kernel(ui_p_reg_start,
    //     ui_p_reg_end,
    //     pv_offset,
    //     v_entry,
    //     dtb_addr_p, dtb_size);
    panic!()
}
