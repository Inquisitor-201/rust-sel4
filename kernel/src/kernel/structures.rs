use riscv::addr::BitField;

use crate::{common::CONFIG_ROOT_CNODE_SIZE_BITS, machine::Paddr, println};

// capability types
pub const CAP_NULL_CAP: usize = 0;
pub const cap_frame_cap: usize = 1;
pub const cap_untyped_cap: usize = 2;
pub const cap_page_table_cap: usize = 3;
pub const cap_endpoint_cap: usize = 4;
pub const cap_notification_cap: usize = 6;
pub const cap_reply_cap: usize = 8;
pub const CAP_CNODE_CAP: usize = 10;
pub const cap_asid_control_cap: usize = 11;
pub const cap_thread_cap: usize = 12;
pub const cap_asid_pool_cap: usize = 13;
pub const CAP_IRQ_CONTROL_CAP: usize = 14;
pub const cap_irq_handler_cap: usize = 16;
pub const cap_zombie_cap: usize = 18;
pub const CAP_DOMAIN_CAP: usize = 20;

// rootserver's cslot indexes
pub const seL4_CapNull: usize = 0; /* null cap */
pub const seL4_CapInitThreadTCB: usize = 1; /* initial thread's TCB cap */
pub const seL4_CapInitThreadCNode: usize = 2; /* initial thread's root CNode cap */
pub const seL4_CapInitThreadVSpace: usize = 3; /* initial thread's VSpace cap */
pub const seL4_CapIRQControl: usize = 4; /* global IRQ controller cap */
pub const seL4_CapASIDControl: usize = 5; /* global ASID controller cap */
pub const seL4_CapInitThreadASIDPool: usize = 6; /* initial thread's ASID pool cap */
pub const seL4_CapIOPortControl: usize = 7; /* global IO port control cap (null cap if not supported) */
pub const seL4_CapIOSpace: usize = 8; /* global IO space cap (null cap if no IOMMU support) */
pub const seL4_CapBootInfoFrame: usize = 9; /* bootinfo frame cap */
pub const seL4_CapInitThreadIPCBuffer: usize = 10; /* initial thread's IPC buffer frame cap */
pub const seL4_CapDomain: usize = 11; /* global domain controller cap */
pub const seL4_CapSMMUSIDControl: usize = 12; /*global SMMU SID controller cap, null cap if not supported*/
pub const seL4_CapSMMUCBControl: usize = 13; /*global SMMU CB controller cap, null cap if not supported*/
pub const seL4_NumInitialCaps: usize = 14;

#[derive(Debug)]
pub enum CapInfo {
    NullCap,
    CnodeCap { ptr: Paddr },
    IrqControlCap,
    DomainCap,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Capability {
    pub words: [usize; 2],
}

impl Capability {
    pub fn new_empty() -> Self {
        Self { words: [0; 2] }
    }

    fn get_type_raw(&self) -> usize {
        let ret = self.words[0].get_bits(59..=63);
        ret
    }

    pub fn get_info(&self) -> CapInfo {
        match self.get_type_raw() {
            CAP_NULL_CAP => CapInfo::NullCap,
            CAP_CNODE_CAP => CapInfo::CnodeCap {
                ptr: Paddr((self.words[0].get_bits(0..=34) << 1) as u64),
            },
            CAP_IRQ_CONTROL_CAP => CapInfo::IrqControlCap,
            CAP_DOMAIN_CAP => CapInfo::DomainCap,
            _ => unimplemented!("unknown capability type {}", self.get_type_raw()),
        }
    }

    // pub fn get_ptr(&self) -> Paddr {
    //     match self.get_info() {
    //         CapInfo::CnodeCap { ptr } => ptr,
    //         _ => panic!("This cnode has no valid ptr"),
    //     }
    // }

    /// 调试：打印该cap对应的cnode的全部内容
    pub fn debug_print_cnode(&self) {
        match self.get_info() {
            CapInfo::CnodeCap { ptr } => {
                println!("\n****** cnode cap info ******");
                let caps_num = 1 << CONFIG_ROOT_CNODE_SIZE_BITS;
                for i in 0..caps_num {
                    let capslot = CapSlot::slot_ref(ptr, i);
                    let info = capslot.cap.get_info();
                    match info {
                        CapInfo::NullCap => {}
                        _ => println!("cnode index {}: {:x?}", i, info),
                    }
                }
                println!("****************************\n");
            }
            _ => {
                panic!("Error: Not a cnode cap!");
            }
        }
    }

    pub fn cnode_write_slot_at(&self, index: usize, cap: Capability) {
        match self.get_info() {
            CapInfo::CnodeCap { ptr } => {
                CapSlot::slot_ref(ptr, index).write(cap);
            }
            _ => {
                panic!("Error: Not a cnode cap!");
            }
        }
    }

    pub fn cap_cnode_cap_new(
        cap_cnode_radix: usize,
        cap_cnode_guard_size: usize,
        cap_cnode_guard: usize,
        cap_cnode_ptr: usize,
    ) -> Self {
        let mut cap = Self::new_empty();

        /* fail if user has passed bits that we will override */
        assert_eq!(cap_cnode_radix & !0x3f, 0);
        assert_eq!(cap_cnode_guard_size & !0x3f, 0);
        assert_eq!(cap_cnode_ptr & !0x7ffffffffe, 0);

        cap.words[0] = (cap_cnode_radix & 0x3f) << 47
            | (cap_cnode_guard_size & 0x3f) << 53
            | (cap_cnode_ptr & 0x7ffffffff) >> 1
            | CAP_CNODE_CAP << 59;
        cap.words[1] = cap_cnode_guard;

        cap
    }

    pub fn cap_domain_cap_new() -> Self {
        let mut cap = Self::new_empty();
        cap.words[0] = CAP_DOMAIN_CAP << 59;
        cap.words[1] = 0;
        cap
    }

    pub fn cap_irq_control_cap_new() -> Self {
        let mut cap = Self::new_empty();
        cap.words[0] = CAP_IRQ_CONTROL_CAP << 59;
        cap.words[1] = 0;
        cap
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MDBNode {
    pub words: [usize; 2],
}

impl MDBNode {
    pub fn new_empty() -> Self {
        Self { words: [0; 2] }
    }

    /// 设置mdb是否可调用(bit 1)
    pub fn set_mdb_revocable(&mut self, v: bool) {
        self.words[1].set_bit(1, v);
    }

    /// 设置mdb是否可调用(bit 0)
    pub fn set_mdb_first_badged(&mut self, v: bool) {
        self.words[1].set_bit(0, v);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CapSlot {
    pub cap: Capability,
    pub mdb_node: MDBNode,
}

impl CapSlot {
    pub fn slot_ref(base: Paddr, index: usize) -> &'static mut Self {
        if index >= 1 << CONFIG_ROOT_CNODE_SIZE_BITS {
            panic!("Error: slot index exceeds max cnode size");
        }
        unsafe { (base.0 as *mut Self).add(index).as_mut().unwrap() }
    }

    pub fn write(&mut self, cap: Capability) {
        self.cap = cap;
        self.mdb_node = MDBNode::new_empty();
        self.mdb_node.set_mdb_revocable(true);
        self.mdb_node.set_mdb_first_badged(true);
    }
}

#[repr(C)]
pub struct CNode {
    pub slots: [CapSlot; 1 << CONFIG_ROOT_CNODE_SIZE_BITS],
}