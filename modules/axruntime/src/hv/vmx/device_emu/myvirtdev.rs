use super::PortIoDevice;
use hypercraft::{HyperResult, HyperError};
use spin::Mutex;
use core::{cell::{RefCell, RefMut}, borrow::BorrowMut};
use crate::hv::vmx::device_emu::Arc;

pub struct MyVirtDevice
{
    port_base: u16,
    inner: Arc<Mutex<MyVirtDeviceInner>>,
}

struct MyVirtDeviceInner
{
    latest_data : u32,
}

impl PortIoDevice for MyVirtDevice
{
    fn port_range(&self) -> core::ops::Range<u16> {
        self.port_base..self.port_base + 1
    }

    fn read(&self, _port: u16, _access_size: u8) -> HyperResult<u32> {
        let inner = self.inner.lock();
        info!("[huaji] read! ret = {:#x}",inner.latest_data);
        Ok(inner.latest_data)
    }

    fn write(&self, _port: u16, _access_size: u8, _value: u32) -> HyperResult {
        info!("[huaji] write! val = {:#x}",_value);

        let mut inner = self.inner.lock();
        inner.latest_data = _value;
        Ok(()) // ignore write
    }
}

impl MyVirtDevice {
    pub fn new(port_base: u16) -> Self {
        Self { port_base ,inner: Arc::new(Mutex::new(MyVirtDeviceInner{latest_data:0}))}
    }
}
