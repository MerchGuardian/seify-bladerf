// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Specifies the direction of a stream.
///
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___c_h_a_n_n_e_l.html#gaeea96995bb88c2f2d4fa7da5d30a1894>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Direction {
    /// Recieve direction
    RX = bladerf_direction_BLADERF_RX as u32,
    /// Transmit direction
    TX = bladerf_direction_BLADERF_TX as u32,
}

impl From<Direction> for bladerf_direction {
    fn from(dir: Direction) -> Self {
        dir as bladerf_direction
    }
}

impl TryFrom<bladerf_direction> for Direction {
    type Error = Error;

    fn try_from(value: bladerf_direction) -> Result<Self> {
        Self::from_repr(value as u32)
            .ok_or_else(|| Error::msg(format!("Invalid Direction value: {value}")))
    }
}
