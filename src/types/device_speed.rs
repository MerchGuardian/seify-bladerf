// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum DeviceSpeed {
    Unknown = bladerf_dev_speed_BLADERF_DEVICE_SPEED_UNKNOWN as u32,
    High = bladerf_dev_speed_BLADERF_DEVICE_SPEED_HIGH as u32,
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
