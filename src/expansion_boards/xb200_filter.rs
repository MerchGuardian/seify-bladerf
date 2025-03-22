// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Filterbanks on the XB200 Transverter board.
///
/// See docs for the [Xb200](crate::expansion_boards::Xb200) for links and more details.
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Xb200Filter {
    /// The 50 MHz filterbank, 50-54 MHz ([6 Meter Band](https://en.wikipedia.org/wiki/6-meter_band))
    MHz50 = bladerf_xb200_filter_BLADERF_XB200_50M as u32,
    /// The 144 MHz filterbank, 144-148 MHz ([2 Meter Band](https://en.wikipedia.org/wiki/2-meter_band))
    ///
    /// NOTE: BladeRF filter specs and Product page are not consistent with what the range is.
    MHz144 = bladerf_xb200_filter_BLADERF_XB200_144M as u32,
    /// The 222 MHz filterbank, 206-235 MHz ([1.25 Meter Band](https://en.wikipedia.org/wiki/1.25-meter_band))
    MHz222 = bladerf_xb200_filter_BLADERF_XB200_222M as u32,
    /// The custom filterbank where a user can connect their own filters.
    Custom = bladerf_xb200_filter_BLADERF_XB200_CUSTOM as u32,
    /// `libbladerf` will select an appropriate filterbank based on the filters 1 dB points and the selected frequency.
    ///
    /// If none of the filterbanks are suitable, the [Custom](Xb200Filter::Custom) path will be selected.
    Auto1dB = bladerf_xb200_filter_BLADERF_XB200_AUTO_1DB as u32,
    /// `libbladerf` will select an appropriate filterbank based on the filters 3 dB points and the selected frequency.
    ///
    /// If none of the filterbanks are suitable, the [Custom](Xb200Filter::Custom) path will be selected.
    Auto3dB = bladerf_xb200_filter_BLADERF_XB200_AUTO_3DB as u32,
}

impl TryFrom<bladerf_xb200_filter> for Xb200Filter {
    type Error = Error;

    fn try_from(value: bladerf_xb200_filter) -> Result<Self> {
        Self::from_repr(value as u32)
            .ok_or_else(|| Error::msg(format!("Invalid Expansion Module value: {value}")))
    }
}
