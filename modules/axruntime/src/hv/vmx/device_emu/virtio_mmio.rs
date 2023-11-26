#![no_std]

pub const VIRTIO_F_VERSION_1: usize = 1 << 32;
pub const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
pub const VIRTIO_MMIO_VERSION: usize = 0x004;
pub const VIRTIO_MMIO_DEVICE_ID: usize = 0x008;
pub const VIRTIO_MMIO_VENDOR_ID: usize = 0x00c;
pub const VIRTIO_MMIO_HOST_FEATURES: usize = 0x010;
pub const VIRTIO_MMIO_HOST_FEATURES_SEL: usize = 0x014;
pub const VIRTIO_MMIO_GUEST_FEATURES: usize = 0x020;
pub const VIRTIO_MMIO_GUEST_FEATURES_SEL: usize = 0x024;
pub const VIRTIO_MMIO_GUEST_PAGE_SIZE: usize = 0x28;
pub const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
pub const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
pub const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
pub const VIRTIO_MMIO_QUEUE_ALIGN: usize = 0x03C;
pub const VIRTIO_MMIO_QUEUE_PFN: usize = 0x040;
pub const VIRTIO_MMIO_QUEUE_READY: usize = 0x044;
pub const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050;
pub const VIRTIO_MMIO_INTERRUPT_STATUS: usize = 0x060;
pub const VIRTIO_MMIO_INTERRUPT_ACK: usize = 0x064;
pub const VIRTIO_MMIO_STATUS: usize = 0x070;
pub const VIRTIO_MMIO_QUEUE_DESC_LOW: usize = 0x080;
pub const VIRTIO_MMIO_QUEUE_DESC_HIGH: usize = 0x084;
pub const VIRTIO_MMIO_QUEUE_AVAIL_LOW: usize = 0x090;
pub const VIRTIO_MMIO_QUEUE_AVAIL_HIGH: usize = 0x094;
pub const VIRTIO_MMIO_QUEUE_USED_LOW: usize = 0x0a0;
pub const VIRTIO_MMIO_QUEUE_USED_HIGH: usize = 0x0a4;
pub const VIRTIO_MMIO_CONFIG_GENERATION: usize = 0x0fc;
pub const VIRTIO_MMIO_CONFIG: usize = 0x100;
pub const VIRTIO_MMIO_REGS_END: usize = 0x200;

pub enum VirtioDeviceType {
    None = 0,
    Net = 1,
    Block = 2,
}

pub struct VirtMmioRegs {
    pub magic: u32,
    pub version: u32,
    pub device_id: u32,
    pub vendor_id: u32,
    pub dev_feature: u32,
    pub dev_feature_sel: u32,
    pub drv_feature: u32,
    pub drv_feature_sel: u32,
    pub guest_page_size: u32,
    pub q_sel: u32,
    pub q_num_max: u32,
    pub q_align: u32,
    pub q_pfn: u32,
    pub irt_stat: u32,
    pub irt_ack: u32,
    pub dev_stat: u32,
}

impl VirtMmioRegs {
    pub fn new(id: VirtioDeviceType) -> Self
    {
        Self {
            magic: 0x74726976,
            version: 0x1,
            vendor_id: 0x8888,
            device_id: id as u32,
            dev_feature: 0,
            dev_feature_sel: 0,
            drv_feature: 0,
            drv_feature_sel: 0,
            guest_page_size: 0x1000,
            q_sel: 0,
            q_num_max: 256,
            q_align: 0x1000,
            q_pfn: 0,
            irt_stat: 0,
            irt_ack: 0,
            dev_stat: 0,
        }
    }
}



