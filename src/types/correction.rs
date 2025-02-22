use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Correction value, in arbitrary units
///
/// Units taken from here: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___c_o_r_r.html#ga75dd741fde93fecb4d514a1f9a377344>
///
/// Type validation is done to ensure the values are in the correct range, returning None if they are not.
///
/// | Enum Vaiant | Units |
/// |---------|---------|
/// | DcOffsetI | Adjusts the in-phase DC offset. Valid values are [-2048, 2048], which are scaled to the available control bits. |
/// | DcOffsetQ | Adjusts the quadrature DC offset. Valid values are [-2048, 2048], which are scaled to the available control bits. |
/// | Phase | Adjusts phase correction of [-10, 10] degrees, via a provided count value of [-4096, 4096]. |
/// | Gain | Adjusts gain correction value in [-1.0, 1.0], via provided values in the range of [-4096, 4096]. |
pub trait CorrectionValue: Sized {
    const TYPE: Correction;

    const MAX: i16;
    const MIN: i16;

    fn new(value: i16) -> Option<Self> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Some(unsafe { Self::new_unchecked(value) })
        } else {
            None
        }
    }

    fn new_saturating(value: i16) -> Self {
        if value > Self::MAX {
            unsafe { Self::new_unchecked(Self::MAX) }
        } else if value < Self::MIN {
            unsafe { Self::new_unchecked(Self::MIN) }
        } else {
            unsafe { Self::new_unchecked(value) }
        }
    }

    fn max() -> Self {
        unsafe { Self::new_unchecked(Self::MAX) }
    }

    fn min() -> Self {
        unsafe { Self::new_unchecked(Self::MIN) }
    }

    fn value(&self) -> i16;

    /// # Safety
    /// Make sure the value is within the range for the given correction
    unsafe fn new_unchecked(val: i16) -> Self;
}

#[derive(Debug, Clone, Copy)]
pub struct CorrectionDcOffsetI(pub i16);

impl CorrectionValue for CorrectionDcOffsetI {
    const TYPE: Correction = Correction::DcOffsetI;

    const MAX: i16 = 2048;
    const MIN: i16 = -2048;

    fn value(&self) -> i16 {
        self.0
    }

    unsafe fn new_unchecked(value: i16) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CorrectionDcOffsetQ(pub i16);

impl CorrectionValue for CorrectionDcOffsetQ {
    const TYPE: Correction = Correction::DcOffsetQ;

    const MAX: i16 = 2048;
    const MIN: i16 = -2048;

    fn value(&self) -> i16 {
        self.0
    }

    unsafe fn new_unchecked(value: i16) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CorrectionPhase(pub i16);

impl CorrectionValue for CorrectionPhase {
    const TYPE: Correction = Correction::Phase;

    const MAX: i16 = 4096;
    const MIN: i16 = -4096;

    fn value(&self) -> i16 {
        self.0
    }

    unsafe fn new_unchecked(value: i16) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CorrectionGain(pub i16);

impl CorrectionValue for CorrectionGain {
    const TYPE: Correction = Correction::Gain;

    const MAX: i16 = 4096;
    const MIN: i16 = -4096;

    fn value(&self) -> i16 {
        self.0
    }

    unsafe fn new_unchecked(value: i16) -> Self {
        Self(value)
    }
}

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
