use spin::Mutex;

use super::virtio_blk::BlkDev;
use crate::hv::vmx::device_emu::{Vec,Arc};


pub struct RamfsDev {
    inner: Arc<Mutex<RamfsDevInner>>,
}

impl RamfsDev {
    pub fn new(ramsz: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RamfsDevInner::new(ramsz))),
        }
    }
}

impl BlkDev for RamfsDev {
    
    fn capacity(&self) -> usize {
        let inner = self.inner.lock();
        inner.ram.len() / 512
    }
    fn read(&self,sector: usize, buf: &mut[u8]) {
        let inner = self.inner.lock();
        let src = &inner.ram[sector * 512..sector * 512+buf.len()];
        buf.copy_from_slice(src);
    }
    fn write(&self,sector: usize, buf: &[u8]) {
        let mut inner = self.inner.lock();
        let dst = &mut inner.ram[sector * 512..sector * 512+buf.len()];
        dst.copy_from_slice(buf);
    }
}

struct RamfsDevInner {
    ram: Vec<u8>
}

impl RamfsDevInner {
    fn new(ramsz: usize) -> Self {
        let mut ram : Vec<u8> = Vec::new();
        ram.resize(ramsz, 1);
        Self {
            ram
        }
    }
}


