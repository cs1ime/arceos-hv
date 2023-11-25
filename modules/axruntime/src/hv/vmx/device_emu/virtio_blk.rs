use hypercraft::{HyperResult, HyperError};
use spin::Mutex;
use core::{cell::{RefCell, RefMut}, borrow::BorrowMut};
use crate::hv::vmx::device_emu::Arc;

use super::{MmioDevice, virtio_mmio::{VirtMmioRegs, VirtioDeviceType}};

pub struct VirtBlk {
    mmio_start: usize,
    mmio_size: usize,
    inner: Arc<Mutex<VirtBlkInner>>
}

struct VirtBlkInner {
    regs: VirtMmioRegs,
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

impl MmioDevice for VirtBlk {
    
    fn mmio_range(&self) -> core::ops::Range<usize> {
        self.mmio_start..(self.mmio_start+self.mmio_size)
    }
    fn read(&self,offset: usize) -> HyperResult<u32> {
        info!("blk read offset = {:#x}",offset);
        Ok(0)
    }
    fn write(&self,offset: usize,value: u32) -> HyperResult {
        info!("blk write offset = {:#x} value = {:#x}",offset,value);
        Ok(())
    }
}

impl VirtBlkInner {
    fn new() -> Self{
        Self {
            regs: VirtMmioRegs::new(VirtioDeviceType::Block)
        }
    }
}


