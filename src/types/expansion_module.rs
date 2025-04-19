// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Represents the different expansion boards.
///
/// This is specifically for when querying what expansion board is attached with [BladeRf1::get_attached_expansion()][crate::BladeRF::get_attached_expansion()]
///
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___x_b.html#gaf5376b7092ea9750302429c2613529e7>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum ExpansionModule {
    /// No expansion module is attached.
    None = bladerf_xb_BLADERF_XB_NONE as u32,
    /// The XB100 GPIO expansion module (Discontinued)
    Xb100 = bladerf_xb_BLADERF_XB_100 as u32,
    /// The XB200 expansion module
    ///
    /// See also [Xb200][crate::expansion_boards::Xb200]
    ///
    /// Nuand's product page: <https://www.nuand.com/product/hf-vhf-transverter/>
    Xb200 = bladerf_xb_BLADERF_XB_200 as u32,
    /// The XB300 expansion module
    ///
    /// Nuand's product page: <https://www.nuand.com/product/amplifier/> (Discontinued)
    Xb300 = bladerf_xb_BLADERF_XB_300 as u32,
}

impl TryFrom<bladerf_xb> for ExpansionModule {
    type Error = Error;

    fn try_from(value: bladerf_xb) -> Result<Self> {
        Self::from_repr(value as u32)
            .ok_or_else(|| Error::msg(format!("Invalid Expansion Module value: {value}")))
    }
}
