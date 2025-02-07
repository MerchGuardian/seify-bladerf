// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Direction Enum
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Direction {
    RX = bladerf_direction_BLADERF_RX as u32,
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
