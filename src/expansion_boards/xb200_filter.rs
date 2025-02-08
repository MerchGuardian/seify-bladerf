use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Xb200Filter {
    MHz50 = bladerf_xb200_filter_BLADERF_XB200_50M,
    MHz144 = bladerf_xb200_filter_BLADERF_XB200_144M,
    MHz222 = bladerf_xb200_filter_BLADERF_XB200_222M,
    Custom = bladerf_xb200_filter_BLADERF_XB200_CUSTOM,
    Auto1dB = bladerf_xb200_filter_BLADERF_XB200_AUTO_1DB,
    Auto3dB = bladerf_xb200_filter_BLADERF_XB200_AUTO_3DB,
}

impl TryFrom<bladerf_xb200_filter> for Xb200Filter {
    type Error = Error;

    fn try_from(value: bladerf_xb200_filter) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid Expansion Module value: {value}")))
    }
}
