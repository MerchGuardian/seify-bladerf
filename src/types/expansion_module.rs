// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum ExpansionModule {
    None = bladerf_xb_BLADERF_XB_NONE as u32,
    Xb100 = bladerf_xb_BLADERF_XB_100 as u32,
    Xb200 = bladerf_xb_BLADERF_XB_200 as u32,
    Xb300 = bladerf_xb_BLADERF_XB_300 as u32,
}

impl TryFrom<bladerf_xb> for ExpansionModule {
    type Error = Error;

    fn try_from(value: bladerf_xb) -> Result<Self> {
        Self::from_repr(value as u32)
            .ok_or_else(|| Error::msg(format!("Invalid Expansion Module value: {value}")))
    }
}
