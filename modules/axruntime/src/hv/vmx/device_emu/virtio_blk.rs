use hypercraft::{HyperResult, HyperError};
use spin::Mutex;
use core::{cell::{RefCell, RefMut}, borrow::BorrowMut};
use crate::hv::vmx::device_emu::{Arc, virtio_mmio::{VIRTIO_MMIO_VERSION,VIRTIO_MMIO_MAGIC_VALUE, VIRTIO_MMIO_DEVICE_ID, VIRTIO_MMIO_QUEUE_NUM_MAX, VIRTIO_MMIO_GUEST_PAGE_SIZE}};

use super::{MmioDevice, virtio_mmio::{VirtMmioRegs, VirtioDeviceType}};

pub struct VirtBlk {
    mmio_start: usize,
    mmio_size: usize,
    inner: Arc<Mutex<VirtBlkInner>>
}

impl VirtBlk {
    pub fn new(mmio_start: usize,mmio_size: usize) -> Self{
        info!("blk created!");
        Self {
            mmio_start,
            mmio_size,
            inner: Arc::new(Mutex::new(VirtBlkInner::new())),
        }
    }
}

struct VirtBlkInner {
    regs: VirtMmioRegs,

}


impl VirtBlkInner {
    fn new() -> Self{
        Self {
            regs: VirtMmioRegs::new(VirtioDeviceType::Block)
        }
    }

    fn magic(&self) -> u32 {
        self.regs.magic
    }

}

impl MmioDevice for VirtBlk {
    
    fn mmio_range(&self) -> core::ops::Range<usize> {
        self.mmio_start..(self.mmio_start+self.mmio_size)
    }
    fn read(&self,offset: usize) -> HyperResult<u32> {
        // info!("blk read offset = {:#x}",offset);

        let r = {
            let inner = self.inner.lock();

            match offset {
                VIRTIO_MMIO_MAGIC_VALUE => Ok(inner.regs.magic),
                VIRTIO_MMIO_VERSION => Ok(inner.regs.version),
                VIRTIO_MMIO_DEVICE_ID => Ok(inner.regs.device_id),
                VIRTIO_MMIO_QUEUE_NUM_MAX => Ok(inner.regs.q_num_max),
                _ => Ok(0)
            }
        };
        info!("blk read offset = {:#x} r = {:?}",offset,r);
        r
    }
    fn write(&self,offset: usize,value: u32) -> HyperResult {
        info!("blk write offset = {:#x} value = {:#x}",offset,value);

        let mut inner = self.inner.lock();

        match offset {
            VIRTIO_MMIO_STATUS => {inner.regs.dev_stat = value},
            VIRTIO_MMIO_GUEST_PAGE_SIZE => {inner.regs.guest_page_size = value},
            _ => {}
        }
        Ok(())
    }
}

