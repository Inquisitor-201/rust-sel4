use sel4_common::structures_common::tcbCTable;

use super::{structures::CapSlot, thread::TCBInner};

pub fn lookup_slot(thread: &TCBInner, cptr: usize) -> &'static mut CapSlot {
    // cap_t threadRoot;
    // resolveAddressBits_ret_t res_ret;
    // lookupSlot_raw_ret_t ret;

    let thread_root = thread.tcb_cte_slot(tcbCTable).cap;
    thread_root.cnode_slot_at(cptr)
}
