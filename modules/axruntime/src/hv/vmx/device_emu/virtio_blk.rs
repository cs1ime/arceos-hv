use hypercraft::{HyperResult, HyperError};
use spin::Mutex;
use core::{cell::{RefCell, RefMut}, borrow::BorrowMut, mem::size_of};
use crate::hv::vmx::{device_emu::{Arc, virtio_mmio::{VIRTIO_MMIO_STATUS,VIRTIO_MMIO_QUEUE_NOTIFY,VIRTIO_MMIO_VERSION,VIRTIO_MMIO_MAGIC_VALUE, VIRTIO_MMIO_DEVICE_ID, VIRTIO_MMIO_QUEUE_NUM_MAX, VIRTIO_MMIO_GUEST_PAGE_SIZE, VIRTIO_MMIO_QUEUE_PFN, VIRTIO_MMIO_QUEUE_ALIGN, VIRTIO_MMIO_CONFIG, VIRTIO_BLK_CONFIG_CAPACITY, VIRTIO_MMIO_QUEUE_NUM}, virtio_queue::{VringDesc, VringAvail, VringUsed}}, VCpu};

use super::{MmioDevice, virtio_mmio::{VirtMmioRegs, VirtioDeviceType}, virtio_queue::{Virtq, VirtQueueLayout}};

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
    capacity: u32,
    vq: Virtq,
}


impl VirtBlkInner {
    fn new() -> Self{
        Self {
            regs: VirtMmioRegs::new(VirtioDeviceType::Block),
            capacity: 100,
            vq: Virtq::default(),
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
    fn read(&self,vcpu: &mut VCpu,offset: usize) -> HyperResult<u64> {
        // info!("blk read offset = {:#x}",offset);

        let r = {
            let inner = self.inner.lock();

            if offset >= VIRTIO_MMIO_CONFIG {
                let offset = offset - VIRTIO_MMIO_CONFIG;
                match offset {
                    VIRTIO_BLK_CONFIG_CAPACITY => Ok(inner.capacity as u64),
                    _ => Ok(0),
                }
            }
            else {
                match offset {
                    VIRTIO_MMIO_MAGIC_VALUE => Ok(inner.regs.magic as u64),
                    VIRTIO_MMIO_VERSION => Ok(inner.regs.version as u64),
                    VIRTIO_MMIO_DEVICE_ID => Ok(inner.regs.device_id as u64),
                    VIRTIO_MMIO_QUEUE_NUM_MAX => Ok(inner.regs.q_num_max as u64),
                    
                    _ => Ok(0)
                }
            }

            
        };
        info!("blk read offset = {:#x} r = {:?}",offset,r);
        r
    }
    fn write(&self,vcpu: &mut VCpu,offset: usize,value: u64) -> HyperResult {
        info!("blk write offset = {:#x} value = {:#x}",offset,value);

        let mut inner = self.inner.lock();

        match offset {
            VIRTIO_MMIO_STATUS => {inner.regs.dev_stat = value as u32},
            VIRTIO_MMIO_GUEST_PAGE_SIZE => {inner.regs.guest_page_size = value as u32},
            VIRTIO_MMIO_QUEUE_PFN => {inner.regs.q_pfn = value as u32},
            VIRTIO_MMIO_QUEUE_ALIGN => {inner.regs.q_align = value as u32},
            VIRTIO_MMIO_QUEUE_NUM => {inner.regs.q_num = value as u32},
            VIRTIO_MMIO_QUEUE_NOTIFY => {
                inner.notify(vcpu);
            },
            _ => {}
        }
        Ok(())
    }
}


impl VirtBlkInner {
    fn notify(&self,vcpu: &mut VCpu) {
        if let Some(gpa_access) = vcpu.gpa_access {
            let layout = VirtQueueLayout::new(self.regs.q_num as u16,self.regs.q_align);
            let q_physaddr = self.regs.q_pfn * 0x1000;
            info!("self.regs.q_num = {}",self.regs.q_num);
            info!("avail_offset = {} , get_used_offset = {} , size = {}",layout.get_avail_offset(),layout.get_used_offset(),layout.get_size());
            
            let desc_table = 
                unsafe {
                    let addr = gpa_access(q_physaddr as usize,self.regs.q_num as usize * core::mem::size_of::<VringDesc>()) as *mut [u8];
                    &mut*(addr as *mut [VringDesc])
                };
            let avail = unsafe {
                let addr = gpa_access((q_physaddr as usize + layout.get_avail_offset()) as usize,core::mem::size_of::<VringAvail>()) as *mut [u8];
                    &mut*(addr as *mut VringAvail)
            };
            let used = unsafe {
                let addr = gpa_access((q_physaddr as usize + layout.get_used_offset()) as usize,core::mem::size_of::<VringUsed>()) as *mut [u8];
                    &mut*(addr as *mut VringUsed)
            };
            info!("desc_table = {:?}",desc_table);
            info!("avail = {:?}",avail);
            
            self.vq.set_desc_table(desc_table);
            self.vq.set_avail(avail);
            self.vq.set_used(used);

            
        }
        
        

    }
}