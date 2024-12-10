use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Sampling {
    Unknown = bladerf_sampling_BLADERF_SAMPLING_UNKNOWN as i32,
    Internal = bladerf_sampling_BLADERF_SAMPLING_INTERNAL as i32,
    External = bladerf_sampling_BLADERF_SAMPLING_EXTERNAL as i32,
}

impl TryFrom<bladerf_sampling> for Sampling {
    type Error = Error;

    fn try_from(value: bladerf_sampling) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid Sampling value: {value}")))
    }
}
