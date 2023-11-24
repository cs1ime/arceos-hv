#![no_std]

pub enum VirtioDeviceType {
    None = 0,
    Net = 1,
    Block = 2,
}

pub struct VirtMmioRegs {
    magic: u32,
    version: u32,
    device_id: u32,
    vendor_id: u32,
    q_num_max: u32,
}

impl VirtMmioRegs {
    pub fn new(id: VirtioDeviceType) -> Self
    {
        Self {
            magic: 0x74726976,
            version: 0x2,
            device_id: 0x8888,
            vendor_id: id as u32,
            q_num_max: 0,
        }
    }
}



