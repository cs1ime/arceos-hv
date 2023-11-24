mod i8259_pic;
mod lapic;
mod uart16550;
mod myvirtdev;
mod virtio_blk;
mod virtio_mmio;

extern crate alloc;
use alloc::{sync::Arc, vec, vec::Vec};
use hypercraft::HyperResult;

pub use self::lapic::VirtLocalApic;

pub trait PortIoDevice: Send + Sync {
    fn port_range(&self) -> core::ops::Range<u16>;
    fn read(&self, port: u16, access_size: u8) -> HyperResult<u32>;
    fn write(&self, port: u16, access_size: u8, value: u32) -> HyperResult;
}

pub struct EmuContext {
    pub address: usize,
    pub width: usize,
    pub write: bool,
    pub reg: usize,
    pub reg_width: usize,
}

pub trait MmioDevice: Send + Sync {
    fn mmio_range(&self) -> core::ops::Range<usize>;
    fn access(&self,offset: usize,write: bool);
}

pub struct VirtDeviceList {
    port_io_devices: Vec<Arc<dyn PortIoDevice>>,
    mmio_devices: Vec<Arc<dyn MmioDevice>>,
}

impl VirtDeviceList {
    pub fn find_port_io_device(&self, port: u16) -> Option<&Arc<dyn PortIoDevice>> {
        self.port_io_devices
            .iter()
            .find(|dev| dev.port_range().contains(&port))
    }
    pub fn find_mmio_device(&self, address: usize) -> Option<&Arc<dyn MmioDevice>> {
        self.mmio_devices
            .iter()
            .find(|dev| dev.mmio_range().contains(&address))
    }
}

lazy_static::lazy_static! {
    static ref VIRT_DEVICES : VirtDeviceList = VirtDeviceList {
        port_io_devices: vec![
            Arc::new(uart16550::Uart16550::new(0x3f8)), // COM1
            Arc::new(i8259_pic::I8259Pic::new(0x20)), // PIC1
            Arc::new(i8259_pic::I8259Pic::new(0xA0)), // PIC2
            Arc::new(myvirtdev::MyVirtDevice::new(0x2233)), // MyVirt1
        ],
        mmio_devices: vec![
            Arc::new(virtio_blk::VirtBlk::new(0xfef0_0000, 0x1000))
        ]
    };
}

pub fn all_virt_devices() -> &'static VirtDeviceList {
    &VIRT_DEVICES
}
