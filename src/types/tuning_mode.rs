use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TuningMode {
    Host = bladerf_tuning_mode_BLADERF_TUNING_MODE_HOST,
    FPGA = bladerf_tuning_mode_BLADERF_TUNING_MODE_FPGA,
    Invalid = bladerf_tuning_mode_BLADERF_TUNING_MODE_INVALID,
}

impl TryFrom<bladerf_tuning_mode> for TuningMode {
    type Error = Error;

    fn try_from(value: bladerf_tuning_mode) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid TuningMode value: {value}")))
    }
}
