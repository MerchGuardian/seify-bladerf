// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Xb200Filter {
    MHz50 = bladerf_xb200_filter_BLADERF_XB200_50M as u32,
    MHz144 = bladerf_xb200_filter_BLADERF_XB200_144M as u32,
    MHz222 = bladerf_xb200_filter_BLADERF_XB200_222M as u32,
    Custom = bladerf_xb200_filter_BLADERF_XB200_CUSTOM as u32,
    Auto1dB = bladerf_xb200_filter_BLADERF_XB200_AUTO_1DB as u32,
    Auto3dB = bladerf_xb200_filter_BLADERF_XB200_AUTO_3DB as u32,
}

impl TryFrom<bladerf_xb200_filter> for Xb200Filter {
    type Error = Error;

    fn try_from(value: bladerf_xb200_filter) -> Result<Self> {
        Self::from_repr(value as u32)
            .ok_or_else(|| Error::msg(format!("Invalid Expansion Module value: {value}")))
    }
}
