use core::mem::size_of;
use bitflags::bitflags;
use spin::Mutex;
use crate::hv::vmx::device_emu::{Arc};

fn queue_align_up(size: usize,q_align: u32) -> usize {
    (size + q_align as usize) & !(q_align as usize - 1)
}

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

#[repr(C)]
#[derive(Debug,Copy, Clone)]
pub struct VringAvail {
    flags: u16,
    idx: u16,
    ring: [u16; 256],
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
    inner: Arc<Mutex<VirtqInner<'static>>>,
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
}

struct VirtqInner<'a> {
    ready: usize,
    vq_index: usize,
    num: usize,
    desc_table: Option<&'a mut [VringDesc]>,
    avail: Option<&'a mut VringAvail>,
    used: Option<&'a mut VringUsed>,
    last_avail_idx: u16,
    last_used_idx: u16,
    used_flags: u16,

    desc_table_addr: usize,
    avail_addr: usize,
    used_addr: usize,
}

impl VirtqInner<'_> {
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





