use crate::kernel::structures::*;

pub fn derive_cap(cap: Capability) -> Capability {
    // match cap.get_info() {
    //     CapInfo::FrameCap { vptr, pptr }
    // }
    cap
}

pub fn cte_insert(new_cap: Capability, src_slot: &CapSlot, dest_slot: &mut CapSlot) {
    match dest_slot.cap.get_info() {
        CapInfo::NullCap => {}
        _ => panic!("cte_insert: dest_slot not null"),
    }
    assert!(dest_slot.mdb_node.is_empty());

    // todo: newcap mdb
    // todo: setUntypedCapAsFull
    dest_slot.cap = new_cap;
    // todo dest_slot.mdb
}
