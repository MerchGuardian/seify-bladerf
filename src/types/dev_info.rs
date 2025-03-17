use crate::{sys::*, BladeRfAny, Result};
use bytemuck::cast_slice;

use super::Backend;

/// Information about a bladerf device connected to the system
///
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__devinfo.html>
#[derive(Clone, Debug)]
pub struct DevInfo(pub(crate) bladerf_devinfo);

impl DevInfo {
    /// The USB [Backend]/Driver that will be used to interface with the device
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__devinfo.html#a8b9925b92ef8bcfd7ebe0c26c742c5d7>
    pub fn backend(&self) -> Result<Backend> {
        self.0.backend.try_into()
    }

    /// Gets the serial number of the device.
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__devinfo.html#abe126c2e73d5e14a30c3648521a9aee0>
    pub fn serial(&self) -> String {
        // TODO, should we just do a try from, that way we can panic on some weird edge case?
        // I don't every expect to actually see it, but I imagine a scenario where there is some bug and this silently handles it.
        String::from_utf8_lossy(cast_slice(&self.0.serial[..32])).to_string()
    }

    /// The USB Bus number that the device is attached to
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__devinfo.html#aed755de9311701fa83379132e69e53df>
    pub fn usb_bus(&self) -> Option<u8> {
        // TODO: Why is this an option?
        Some(self.0.usb_bus)
    }

    /// The Address of the device on the USB Bus
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__devinfo.html#a1316ffb2f3147a76bbdf84fa2db3e490>
    pub fn usb_addr(&self) -> Option<u8> {
        // TODO: Why is this an option?
        Some(self.0.usb_addr)
    }

    /// The device instance or ID
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__devinfo.html#a020ad41f5104a1bd5bb455b2144e8885>
    pub fn instance(&self) -> u32 {
        self.0.instance
    }

    /// USB manufacturer description
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__devinfo.html#acc4554d02a09a200b647b137c73ef087>
    pub fn manufacturer(&self) -> String {
        // TODO: This seems to be `Nuandwn>` instead of `Nuandwn` (what bladeRF-cli --probe gets)
        // TODO, should we just do a try from, that way we can panic on some weird edge case?
        String::from_utf8_lossy(cast_slice(&self.0.manufacturer)).to_string()
    }

    /// The USB product description
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__devinfo.html#a4369c00791073f53ce0d4c606df27c6f>
    pub fn product(&self) -> String {
        // TODO, should we just do a try from, that way we can panic on some weird edge case?
        String::from_utf8_lossy(cast_slice(&self.0.product)).to_string()
    }

    /// Open a device using the information in this struct
    pub fn open(&self) -> Result<BladeRfAny> {
        BladeRfAny::open_with_devinfo(self)
    }
}

impl From<bladerf_devinfo> for DevInfo {
    fn from(dev: bladerf_devinfo) -> Self {
        Self(dev)
    }
}
