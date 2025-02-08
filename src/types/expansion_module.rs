use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum ExpansionModule {
    None = bladerf_xb_BLADERF_XB_NONE,
    Xb100 = bladerf_xb_BLADERF_XB_100,
    Xb200 = bladerf_xb_BLADERF_XB_200,
    Xb300 = bladerf_xb_BLADERF_XB_300,
}

impl TryFrom<bladerf_xb> for ExpansionModule {
    type Error = Error;

    fn try_from(value: bladerf_xb) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid Expansion Module value: {value}")))
    }
}
