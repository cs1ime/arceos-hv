use core::mem::size_of;
use bitflags::bitflags;
use spin::Mutex;
use crate::hv::vmx::device_emu::{Arc};

fn queue_align_up(size: usize,q_align: u32) -> usize {
    (size + q_align as usize) & !(q_align as usize - 1)
}

pub const VIRTQ_READY: usize = 1;
pub const VIRTQ_DESC_F_NEXT: u16 = 1;
pub const VIRTQ_DESC_F_WRITE: u16 = 2;

pub struct VirtQueueLayout {
    avail_offset: usize,
    used_offset: usize,
    size: usize,
}

impl VirtQueueLayout {

    pub fn new(queue_size: u16,q_align: u32) -> Self {
        assert!(
            queue_size.is_power_of_two(),
            "queue size should be a power of 2"
        );
        let queue_size = queue_size as usize;
        let desc = size_of::<VringDesc>() * queue_size;
        let avail = size_of::<u16>() * (3 + queue_size);
        let used = size_of::<u16>() * 3 + size_of::<VringUsedElem>() * queue_size;
        VirtQueueLayout {
            avail_offset: desc,
            used_offset: queue_align_up(desc + avail,q_align),
            size: queue_align_up(desc + avail,q_align) + queue_align_up(used,q_align),
        }
    }
    pub fn get_avail_offset(&self) -> usize {
        self.avail_offset
    }
    pub fn get_used_offset(&self) -> usize {
        self.used_offset
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
}

#[derive(Debug,Copy, Clone)]
pub struct VringDesc {
    /*Address (guest-physical)*/
    pub addr: usize,
    /* Length */
    len: u32,
    /* The flags as indicated above */
    flags: u16,
    /* We chain unused descriptors via this, too */
    next: u16,
}

impl VringDesc {
    pub fn desc_has_next(&self) -> bool {
        self.flags & VIRTQ_DESC_F_NEXT != 0
    }
    pub fn desc_next_idx(&self) -> u16 {
        self.next
    }
    pub fn desc_addr(&self) -> usize {
        self.addr
    }
    pub fn desc_len(&self) -> u32 {
        self.len
    }
    pub fn desc_is_writable(&self) -> bool {
        self.flags & VIRTQ_DESC_F_WRITE as u16 != 0
    }
}

#[repr(C)]
#[derive(Debug,Copy, Clone)]
pub struct VringAvail {
    flags: u16,
    idx: u16,
    ring: [u16; 256],
}

impl VringAvail {
    pub fn get_ring_value(&self,index: usize) -> u16 {
        self.ring[index]
    }
}

#[repr(C)]
#[derive(Debug,Copy, Clone)]
struct VringUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
#[derive(Debug,Copy, Clone)]
pub struct VringUsed {
    flags: u16,
    idx: u16,
    ring: [VringUsedElem; 256],
}


pub struct Virtq{
    inner: Arc<Mutex<VirtqInner>>,
}

impl Virtq {
    pub fn default() -> Virtq {
        Virtq {
            inner: Arc::new(Mutex::new(VirtqInner::default())),
        }
    }

    pub fn set_desc_table(&self, buf: &'static mut [VringDesc]) {
        let mut inner = self.inner.lock();
        inner.desc_table = Some(buf);
    }
    pub fn set_avail(&self,buf: &'static mut VringAvail) {
        let mut inner = self.inner.lock();
        inner.avail = Some(buf);
    }
    pub fn set_used(&self,buf: &'static mut VringUsed) {
        let mut inner = self.inner.lock();
        inner.used = Some(buf);
    }

    pub fn pop_avail_idx(&self) -> Option<usize> {
        let mut inner: spin::MutexGuard<'_, VirtqInner> = self.inner.lock();
        if let Some(avail) = &inner.avail {
            info!("avail.idx = {}",avail.idx);
            info!("inner.last_avail_idx = {}",inner.last_avail_idx);
            if avail.idx == inner.last_avail_idx {
                return None;
            }
            else {
                if let Some(desc_table) = &inner.desc_table {
                    let avail_index = avail.get_ring_value(inner.last_avail_idx as usize) as usize;
                    inner.last_avail_idx=inner.last_avail_idx.wrapping_add(1);
                    Some(avail_index)
                }
                else {
                    None
                }
            }
        }
        else {
            None
        }
        
    }

    pub fn desc_by_index(&self,idx: usize) -> Option<&'static mut VringDesc>  {
        let mut inner: spin::MutexGuard<'_, VirtqInner> = self.inner.lock();
        if let Some(desc_table) = &mut inner.desc_table {
            let result = &mut desc_table[idx];
            let result_static: &'static mut VringDesc = unsafe {
                // Since we're returning a reference with a 'static lifetime,
                // you need to ensure that the referenced data lives for the 'static lifetime.
                // Be careful when using unsafe code.
                core::mem::transmute(result)
            };
            Some(result_static)
        }
        else {
            None
        }
    }


}

struct VirtqInner {
    ready: usize,
    vq_index: usize,
    num: usize,
    desc_table: Option<&'static mut [VringDesc]>,
    avail: Option<&'static mut VringAvail>,
    used: Option<&'static mut VringUsed>,
    last_avail_idx: u16,
    last_used_idx: u16,
    used_flags: u16,

    desc_table_addr: usize,
    avail_addr: usize,
    used_addr: usize,
}

impl VirtqInner {
    pub fn default() -> Self {
        VirtqInner {
            ready: 0,
            vq_index: 0,
            num: 0,
            desc_table: None,
            avail: None,
            used: None,
            last_avail_idx: 0,
            last_used_idx: 0,
            used_flags: 0,

            desc_table_addr: 0,
            avail_addr: 0,
            used_addr: 0,
        }
    }
    
}





