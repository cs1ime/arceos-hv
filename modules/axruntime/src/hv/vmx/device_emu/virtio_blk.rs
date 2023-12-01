use hypercraft::{HyperResult, HyperError};
use spin::Mutex;
use core::{cell::{RefCell, RefMut, Ref}, borrow::BorrowMut, mem::size_of};
use crate::hv::vmx::{device_emu::{Vec,Arc, virtio_mmio::{VIRTIO_MMIO_STATUS,VIRTIO_MMIO_QUEUE_NOTIFY,VIRTIO_MMIO_VERSION,VIRTIO_MMIO_MAGIC_VALUE, VIRTIO_MMIO_DEVICE_ID, VIRTIO_MMIO_QUEUE_NUM_MAX, VIRTIO_MMIO_GUEST_PAGE_SIZE, VIRTIO_MMIO_QUEUE_PFN, VIRTIO_MMIO_QUEUE_ALIGN, VIRTIO_MMIO_CONFIG, VIRTIO_BLK_CONFIG_CAPACITY, VIRTIO_MMIO_QUEUE_NUM}, virtio_queue::{VringDesc, VringAvail, VringUsed, VringUsedElem}}, VCpu};

use super::{MmioDevice, virtio_mmio::{VirtMmioRegs, VirtioDeviceType}, virtio_queue::{Virtq, VirtQueueLayout}, blk_ramfs::RamfsDev};

pub const VIRTIO_BLK_T_IN: usize = 0;
pub const VIRTIO_BLK_T_OUT: usize = 1;
pub const VIRTIO_BLK_T_FLUSH: usize = 4;
pub const VIRTIO_BLK_T_GET_ID: usize = 8;

/* BLOCK REQUEST STATUS*/
pub const VIRTIO_BLK_S_OK: usize = 0;
// pub const VIRTIO_BLK_S_IOERR: usize = 1;
pub const VIRTIO_BLK_S_UNSUPP: usize = 2;

#[repr(C)]
#[derive(Debug)]
struct BlkReq {
    type_: BlkReqType,
    reserved: u32,
    sector: u64,
}

#[repr(u32)]
#[derive(Debug,Clone, Copy)]
enum BlkReqType {
    In = 0,
    Out = 1,
    Flush = 4,
    Discard = 11,
    WriteZeroes = 13,
}

#[derive(Debug)]
pub struct BlkReqNode {
    req_type: u32,
    reserved: u32,
    sector: usize,
    desc_chain_head_idx: usize,
    iov: Vec<BlkIov>,
    // sum up byte for req
    iov_sum_up: usize,
    // total byte for current req
    iov_total: usize,
}

impl BlkReqNode {
    pub fn default() -> Self {
        BlkReqNode {
            req_type: 0,
            reserved: 0,
            sector: 0,
            desc_chain_head_idx: 0,
            iov: Vec::new(),
            iov_sum_up: 0,
            iov_total: 0,
        }
    }
}

#[derive(Debug)]
pub struct BlkIov {
    pub data_bg: usize,
    pub len: u32,
}


pub trait BlkDev : Send+Sync {
    fn capacity(&self) -> usize;
    fn read(&self,sector: usize, buf: &mut[u8]);
    fn write(&self,sector: usize, buf: &[u8]);
}




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
    dev: Arc<dyn BlkDev>,
    vq: Virtq,
}


impl VirtBlkInner {
    fn new() -> Self{
        Self {
            regs: VirtMmioRegs::new(VirtioDeviceType::Block),
            dev: Arc::new(RamfsDev::new(0x10_0000)),
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
                    VIRTIO_BLK_CONFIG_CAPACITY => Ok(inner.dev.capacity() as u64),
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
        // info!("blk read offset = {:#x} r = {:?}",offset,r);
        r
    }
    fn write(&self,vcpu: &mut VCpu,offset: usize,value: u64) -> HyperResult {
        // info!("blk write offset = {:#x} value = {:#x}",offset,value);

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
            // info!("self.regs.q_num = {}",self.regs.q_num);
            // info!("avail_offset = {} , get_used_offset = {} , size = {}",layout.get_avail_offset(),layout.get_used_offset(),layout.get_size());
            
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

            

            while let Some(new_desc_idx) = self.vq.pop_avail_idx() {
                let mut vreq = BlkReqNode::default();
                let mut desc_idx = new_desc_idx;
                if let Some(new_desc) = self.vq.desc_by_index(new_desc_idx) {
                    let mut desc = new_desc;

                    info!("addr = {:?}",desc);
                    let mut head = true;
                    loop {
                        if desc.desc_has_next() {
                            if head {

                                if (desc.desc_len() as usize) < core::mem::size_of::<BlkReq>() {
                                    panic!("desc.desc_len() < core::mem::size_of::<BlkReq>()");
                                }
                                let req = gpa_access(desc.desc_addr(),desc.desc_len() as usize) as *mut [u8];
                                let req = unsafe{&mut*(req as *mut BlkReq)};
                                
                                info!("req = {:?}",req);

                                vreq.req_type = req.type_ as u32;
                                vreq.sector = req.sector as usize;
                                vreq.desc_chain_head_idx = desc_idx;
                                head = false;
                            }
                            else {
                                info!("content = {:?}",desc);
                                vreq.iov.push(BlkIov{data_bg: desc.desc_addr(),len: desc.desc_len()});
                                vreq.iov_sum_up+=desc.desc_len() as usize;
                            }
                            
                            desc_idx = desc.desc_next_idx() as usize;
                            if let Some(new_desc) = self.vq.desc_by_index(desc_idx) {
                                desc = new_desc;
                                continue;
                            }
                        }
                        else {
                            /*state handler*/
                            if desc.desc_is_writable() {
                                let vstatus = gpa_access(desc.desc_addr(),desc.desc_len() as usize);
                                vstatus[0] = VIRTIO_BLK_S_OK as u8;
                            }
                        }
                        break;
                    }
                }

                info!("{:?}",vreq);

                if vreq.req_type == BlkReqType::In as u32 {
                    for iov in vreq.iov {
                        let buf = gpa_access(iov.data_bg,iov.len as usize);
                        self.dev.read(vreq.sector, buf);
                        info!("buf = {:?}",buf);
                    }
                }
                else if vreq.req_type == BlkReqType::Out as u32 {
                    for iov in vreq.iov {
                        let buf = gpa_access(iov.data_bg,iov.len as usize);
                        self.dev.write(vreq.sector, buf);
                        info!("buf = {:?}",buf);
                    }
                }

                self.vq.push_used_item(VringUsedElem { id: 0, len: 0 });
            }
            

        }
        
        

    }
}

