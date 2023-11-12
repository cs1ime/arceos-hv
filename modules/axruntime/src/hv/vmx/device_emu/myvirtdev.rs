
use super::PortIoDevice;
use hypercraft::{HyperResult, HyperError};

pub struct MyVirtDevice
{
    port_base: u16,
}

impl PortIoDevice for MyVirtDevice
{
    fn port_range(&self) -> core::ops::Range<u16> {
        self.port_base..self.port_base + 1
    }

    fn read(&self, _port: u16, _access_size: u8) -> HyperResult<u32> {
        info!("[huaji] read!");
        Ok(0xDEADBEEF)
    }

    fn write(&self, _port: u16, _access_size: u8, _value: u32) -> HyperResult {
        info!("[huaji] write!");
        Ok(()) // ignore write
    }
}

impl MyVirtDevice {
    pub fn new(port_base: u16) -> Self {
        Self { port_base }
    }
}
