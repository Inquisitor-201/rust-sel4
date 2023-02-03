use crate::kernel::{Capability, CapSlot, CapInfo};

pub fn cte_insert(newCap: Capability, src_slot: &CapSlot, dest_slot: &mut CapSlot) {
    match dest_slot.cap.get_info() {
        CapInfo::NullCap => {},
        _ => panic!("cte_insert: dest_slot not null")
    }
    assert!(dest_slot.mdb_node.is_empty());

    // todo: newcap mdb
    // todo: setUntypedCapAsFull
    dest_slot.cap = newCap;
    // todo dest_slot.mdb
}