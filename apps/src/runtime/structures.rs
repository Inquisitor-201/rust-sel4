pub const CAP_NULL_CAP: usize = 0;
pub const CAP_FRAME_CAP: usize = 1;
pub const CAP_UNTYPED_CAP: usize = 2;
pub const CAP_PAGE_TABLE_CAP: usize = 3;
pub const cap_endpoint_cap: usize = 4;
pub const cap_notification_cap: usize = 6;
pub const cap_reply_cap: usize = 8;
pub const CAP_CNODE_CAP: usize = 10;
pub const CAP_ASID_CONTROL_CAP: usize = 11;
pub const CAP_THREAD_CAP: usize = 12;
pub const CAP_ASID_POOL_CAP: usize = 13;
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

pub const tcbCTable: usize = 0; /* A TCB CNode and a TCB are always allocated together, and adjacently. The CNode comes first. */
pub const tcbVTable: usize = 1; /* VSpace root */
pub const tcbReply: usize = 2; /* Reply cap slot */
pub const tcbCaller: usize = 3; /* TCB of most recent IPC sender */
pub const tcbBuffer: usize = 4; /* IPC buffer cap slot */