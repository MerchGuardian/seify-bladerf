// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// The [USB Device Speed](https://en.wikipedia.org/wiki/USB_hardware#Connectors) that the BladeRF will operate at.
///
/// Speeds not listed are not supported.
///
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html#ga9a3716d6cf5a1c25da8325fa245e92f9>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum DeviceSpeed {
    /// The Device Speed is unknown
    // TODO figure out when this can show up.
    Unknown = bladerf_dev_speed_BLADERF_DEVICE_SPEED_UNKNOWN as u32,
    /// [USB High Speed](https://en.wikipedia.org/wiki/USB_Hi-Speed)
    High = bladerf_dev_speed_BLADERF_DEVICE_SPEED_HIGH as u32,
    /// [USB SuperSpeed](https://en.wikipedia.org/wiki/USB_SuperSpeed)
    Super = bladerf_dev_speed_BLADERF_DEVICE_SPEED_SUPER as u32,
}

impl From<DeviceSpeed> for bladerf_dev_speed {
    fn from(dir: DeviceSpeed) -> Self {
        dir as bladerf_dev_speed
    }
}

impl TryFrom<bladerf_fpga_size> for DeviceSpeed {
    type Error = Error;

    fn try_from(value: bladerf_dev_speed) -> Result<Self> {
        Self::from_repr(value as u32)
            .ok_or_else(|| Error::msg(format!("Invalid Dev Speed discriminant: {value}")))
    }
}
