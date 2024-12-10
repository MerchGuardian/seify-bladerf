use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Correction value, in arbitrary units
pub type CorrectionValue = i16;

/// Correction parameter selection
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Correction {
    DcOffsetI = bladerf_correction_BLADERF_CORR_DCOFF_I as i32,
    DcOffsetQ = bladerf_correction_BLADERF_CORR_DCOFF_Q as i32,
    Phase = bladerf_correction_BLADERF_CORR_PHASE as i32,
    Gain = bladerf_correction_BLADERF_CORR_GAIN as i32,
}

impl TryFrom<bladerf_correction> for Correction {
    type Error = Error;

    fn try_from(value: bladerf_correction) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid Correction value: {value}")))
    }
}
