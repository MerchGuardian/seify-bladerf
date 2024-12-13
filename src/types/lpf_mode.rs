use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum LPFMode {
    Normal = bladerf_lpf_mode_BLADERF_LPF_NORMAL as i32,
    Bypassed = bladerf_lpf_mode_BLADERF_LPF_BYPASSED as i32,
    Disabled = bladerf_lpf_mode_BLADERF_LPF_DISABLED as i32,
}

impl TryFrom<bladerf_lpf_mode> for LPFMode {
    type Error = Error;

    fn try_from(value: bladerf_lpf_mode) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid LPFMode value: {value}")))
    }
}
