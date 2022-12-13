use crate::println;

fn plic_get_current_hart_id() -> usize {
    0
}

pub fn plic_init_hart()
{
    println!("no PLIC present, skip platform specific initialisation");
    // let hart_id = plic_get_current_hart_id();
    // let mut i = 1;
    // let PLIC_NUM_INTERRUPTS = 0;
    
    // while i <= PLIC_NUM_INTERRUPTS {
    //     /* Disable interrupts */
    //     plic_mask_irq(true, i);
    //     i += 1;
    // }

    // /* Set threshold to zero */
    // writel(0, (PLIC_PPTR_BASE + plic_thres_offset(hart_id, PLIC_SVC_CONTEXT)));
}