// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Signal paths on the XB200 Transverter board.
///
/// See docs for the [Xb200](crate::expansion_boards::Xb200) for links and more details.
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Xb200Path {
    /// Bypass the mixer and filterbanks, using the BladeRF as if the transverter board was not there.
    Bypass = bladerf_xb200_path_BLADERF_XB200_BYPASS as u32,
    /// Pass signal through the mixer to up-convert or down-convert.
    Mix = bladerf_xb200_path_BLADERF_XB200_MIX as u32,
}

impl TryFrom<bladerf_xb200_path> for Xb200Path {
    type Error = Error;

    fn try_from(value: bladerf_xb200_path) -> Result<Self> {
        Self::from_repr(value as u32)
            .ok_or_else(|| Error::msg(format!("Invalid Expansion Module value: {value}")))
    }
}
