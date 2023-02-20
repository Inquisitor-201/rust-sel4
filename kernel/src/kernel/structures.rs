use riscv::addr::BitField;
use sel4_common::structures_common::{
    CAP_ASID_CONTROL_CAP, CAP_ASID_POOL_CAP, CAP_CNODE_CAP, CAP_DOMAIN_CAP, CAP_FRAME_CAP,
    CAP_IRQ_CONTROL_CAP, CAP_NULL_CAP, CAP_PAGE_TABLE_CAP, CAP_THREAD_CAP, CAP_UNTYPED_CAP,
};

use crate::{
    common::CONFIG_ROOT_CNODE_SIZE_BITS,
    machine::{Paddr, Vaddr},
    println,
};

#[derive(Debug)]
pub enum CapInfo {
    NullCap,
    FrameCap {
        vptr: Vaddr,
        pptr: Paddr,
        is_device: bool,
    },
    UntypedCap {
        pptr: Paddr,
        is_device: bool,
        size_bits: usize,
    },
    CnodeCap {
        ptr: Paddr,
    },
    ThreadCap {
        ptr: Paddr,
    },
    AsidControlCap,
    AsidPoolCap,
    PageTableCap {
        vptr: Vaddr,
        pptr: Paddr,
        asid: usize,
    },
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
        let ret = self.words[0].get_bits(59..64);
        ret
    }

    pub fn get_info(&self) -> CapInfo {
        match self.get_type_raw() {
            CAP_NULL_CAP => CapInfo::NullCap,
            CAP_FRAME_CAP => CapInfo::FrameCap {
                vptr: Vaddr(self.words[0].get_bits(0..39)),
                pptr: Paddr(self.words[1].get_bits(9..48)),
                is_device: self.words[0].get_bit(54),
            },
            CAP_UNTYPED_CAP => CapInfo::UntypedCap {
                pptr: Paddr(self.words[0].get_bits(0..39)),
                is_device: self.words[1].get_bit(6) as _,
                size_bits: self.words[1].get_bits(0..6),
            },
            CAP_CNODE_CAP => CapInfo::CnodeCap {
                ptr: Paddr(self.words[0].get_bits(0..35) << 1),
            },
            CAP_ASID_CONTROL_CAP => CapInfo::AsidControlCap,
            CAP_THREAD_CAP => CapInfo::ThreadCap {
                ptr: Paddr(self.words[0].get_bits(0..39)),
            },
            CAP_ASID_POOL_CAP => CapInfo::AsidPoolCap,
            CAP_IRQ_CONTROL_CAP => CapInfo::IrqControlCap,
            CAP_DOMAIN_CAP => CapInfo::DomainCap,
            CAP_PAGE_TABLE_CAP => CapInfo::PageTableCap {
                vptr: Vaddr(self.words[0].get_bits(0..39)),
                pptr: Paddr(self.words[1].get_bits(9..48)),
                asid: self.words[1].get_bits(48..64),
            },
            _ => unimplemented!("unknown capability type {}", self.get_type_raw()),
        }
    }

    pub fn get_pptr(&self) -> Paddr {
        match self.get_info() {
            CapInfo::CnodeCap { ptr } => ptr,
            CapInfo::PageTableCap { pptr, .. } => pptr,
            _ => panic!("This cnode has no ptr field"),
        }
    }

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

    pub fn cnode_slot_at(&self, index: usize) -> &'static mut CapSlot {
        match self.get_info() {
            CapInfo::CnodeCap { ptr } => CapSlot::slot_ref(ptr, index),
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

    /// 创建新的指向页表页面的cap
    /// 参数: capPTMappedASID：进程标识号asid,
    ///       capPTBasePtr：页表页面的物理地址,
    ///       capPTIsMapped：pt是否mapped，即cap是否有效,
    ///       capPTMappedAddress：映射的虚拟地址
    pub fn cap_page_table_cap_new(
        capPTMappedASID: usize,
        capPTBasePtr: usize,
        capPTIsMapped: bool,
        capPTMappedAddress: usize,
    ) -> Capability {
        let mut cap = Self::new_empty();

        cap.words[0] =
            CAP_PAGE_TABLE_CAP << 59 | (capPTIsMapped as usize) << 39 | capPTMappedAddress;
        cap.words[1] = capPTMappedASID << 48 | capPTBasePtr << 9;

        cap
    }

    /// 创建新的指向页面frame的cap
    /// 参数: capFMappedASID：进程标识号asid,
    ///       capPTBasePtr：映射的物理地址,
    ///       capFSize：页面大小,
    ///       capFVMRights：访问权限,
    ///       capFIsDevice：是否为设备,
    ///       capPTMappedAddress：映射的虚拟地址
    pub fn cap_frame_cap_new(
        capFMappedASID: usize,
        capFBasePtr: usize,
        capFSize: usize,
        capFVMRights: usize,
        capFIsDevice: bool,
        capFMappedAddress: usize,
    ) -> Capability {
        let mut cap = Self::new_empty();

        cap.words[0] = CAP_FRAME_CAP << 59
            | capFSize << 57
            | capFVMRights << 55
            | (capFIsDevice as usize) << 54
            | capFMappedAddress;
        cap.words[1] = capFMappedASID << 48 | capFBasePtr << 9;

        cap
    }

    pub fn cap_asid_pool_cap_new(capASIDBase: usize, capASIDPool: usize) -> Capability {
        let mut cap = Self::new_empty();
        cap.words[0] = CAP_ASID_POOL_CAP << 59 | capASIDBase << 43 | capASIDPool >> 2;
        cap
    }

    pub fn cap_asid_control_cap_new() -> Capability {
        let mut cap = Self::new_empty();
        cap.words[0] = CAP_ASID_CONTROL_CAP << 59;
        cap
    }

    pub fn cap_thread_cap_new(cap_tcb_ptr: usize) -> Capability {
        let mut cap = Self::new_empty();
        cap.words[0] = CAP_THREAD_CAP << 59 | cap_tcb_ptr;
        cap
    }

    pub fn cap_untyped_cap_new(
        capFreeIndex: usize,
        capIsDevice: bool,
        capBlockSize: usize,
        capPtr: usize,
    ) -> Capability {
        let mut cap = Self::new_empty();
        cap.words[0] = CAP_UNTYPED_CAP << 59 | capPtr;
        cap.words[1] = capFreeIndex << 25 | (capIsDevice as usize) << 6 | capBlockSize;
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

    pub fn is_empty(&self) -> bool {
        self.words[0] == 0 && self.words[1] == 0
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
        unsafe { base.as_raw_ptr_mut::<Self>().add(index).as_mut().unwrap() }
    }

    pub fn write(&mut self, cap: Capability) {
        self.cap = cap;
        self.mdb_node = MDBNode::new_empty();
        self.mdb_node.set_mdb_revocable(true);
        self.mdb_node.set_mdb_first_badged(true);
    }
}
