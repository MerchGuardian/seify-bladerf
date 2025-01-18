use crate::{sys::*, BladeRF, BladeRfAny, Result};
use bytemuck::cast_slice;

use super::Backend;

/// Information about a bladerf device connect to the system
#[derive(Clone, Debug)]
pub struct DevInfo(pub(crate) bladerf_devinfo);

impl DevInfo {
    pub fn backend(&self) -> Result<Backend> {
        self.0.backend.try_into()
    }
    pub fn serial(&self) -> String {
        String::from_utf8_lossy(cast_slice(&self.0.serial[..32])).to_string()
    }
    pub fn usb_bus(&self) -> Option<u8> {
        Some(self.0.usb_bus)
    }
    pub fn usb_addr(&self) -> Option<u8> {
        Some(self.0.usb_addr)
    }
    pub fn instance(&self) -> u32 {
        self.0.instance
    }
    pub fn manufacturer(&self) -> String {
        // TODO: This seems to be `Nuandwn>` instead of `Nuandwn` (what bladeRF-cli --probe gets)
        String::from_utf8_lossy(cast_slice(&self.0.manufacturer)).to_string()
    }
    pub fn product(&self) -> String {
        String::from_utf8_lossy(cast_slice(&self.0.product)).to_string()
    }

    pub fn open(&self) -> Result<BladeRfAny> {
        BladeRfAny::open_with_devinfo(self)
    }
}

impl From<bladerf_devinfo> for DevInfo {
    fn from(dev: bladerf_devinfo) -> Self {
        Self(dev)
    }
}
