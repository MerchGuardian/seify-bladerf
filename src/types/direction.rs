use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Direction Enum
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Direction {
    RX = bladerf_direction_BLADERF_RX,
    TX = bladerf_direction_BLADERF_TX,
}

impl From<Direction> for bladerf_direction {
    fn from(dir: Direction) -> Self {
        dir as bladerf_direction
    }
}

impl TryFrom<bladerf_direction> for Direction {
    type Error = Error;

    fn try_from(value: bladerf_direction) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid Direction value: {value}")))
    }
}
