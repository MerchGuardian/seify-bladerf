use std::ops::Add;

use num::traits::SaturatingAdd;
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

impl Add for CorrectionDcOffsetI {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let new = self.0 + rhs.0;
        match Self::new(new) {
            Some(val) => val,
            None => {
                let wrapped_offet = new - Self::MAX;
                unsafe { Self::new_unchecked(Self::MIN + wrapped_offet) }
            }
        }
    }
}

impl SaturatingAdd for CorrectionDcOffsetI {
    fn saturating_add(&self, rhs: &Self) -> Self {
        let new = self.0 + rhs.0;
        match Self::new(new) {
            Some(val) => val,
            None => unsafe { Self::new_unchecked(Self::MAX) },
        }
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

impl Add for CorrectionDcOffsetQ {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let new = self.0 + rhs.0;
        match Self::new(new) {
            Some(val) => val,
            None => {
                let wrapped_offet = new - Self::MAX;
                unsafe { Self::new_unchecked(Self::MIN + wrapped_offet) }
            }
        }
    }
}

impl SaturatingAdd for CorrectionDcOffsetQ {
    fn saturating_add(&self, rhs: &Self) -> Self {
        let new = self.0 + rhs.0;
        match Self::new(new) {
            Some(val) => val,
            None => unsafe { Self::new_unchecked(Self::MAX) },
        }
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

impl Add for CorrectionPhase {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let new = self.0 + rhs.0;
        match Self::new(new) {
            Some(val) => val,
            None => {
                let wrapped_offet = new - Self::MAX;
                unsafe { Self::new_unchecked(Self::MIN + wrapped_offet) }
            }
        }
    }
}

impl SaturatingAdd for CorrectionPhase {
    fn saturating_add(&self, rhs: &Self) -> Self {
        let new = self.0 + rhs.0;
        match Self::new(new) {
            Some(val) => val,
            None => unsafe { Self::new_unchecked(Self::MAX) },
        }
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

impl Add for CorrectionGain {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let new = self.0 + rhs.0;
        match Self::new(new) {
            Some(val) => val,
            None => {
                let wrapped_offet = new - Self::MAX;
                unsafe { Self::new_unchecked(Self::MIN + wrapped_offet) }
            }
        }
    }
}

impl SaturatingAdd for CorrectionGain {
    fn saturating_add(&self, rhs: &Self) -> Self {
        let new = self.0 + rhs.0;
        match Self::new(new) {
            Some(val) => val,
            None => unsafe { Self::new_unchecked(Self::MAX) },
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrections_add_saturating() {
        let correction_a = CorrectionDcOffsetI::new(CorrectionDcOffsetI::MAX - 8).unwrap();
        let correction_b = CorrectionDcOffsetI::new(50).unwrap();
        let new_correction = correction_a.saturating_add(&correction_b);
        assert_eq!(new_correction.value(), CorrectionDcOffsetI::MAX);

        let correction_a = CorrectionDcOffsetQ::new(CorrectionDcOffsetQ::MAX - 8).unwrap();
        let correction_b = CorrectionDcOffsetQ::new(50).unwrap();
        let new_correction = correction_a.saturating_add(&correction_b);
        assert_eq!(new_correction.value(), CorrectionDcOffsetQ::MAX);

        let correction_a = CorrectionGain::new(CorrectionGain::MAX - 8).unwrap();
        let correction_b = CorrectionGain::new(50).unwrap();
        let new_correction = correction_a.saturating_add(&correction_b);
        assert_eq!(new_correction.value(), CorrectionGain::MAX);

        let correction_a = CorrectionPhase::new(CorrectionPhase::MAX - 8).unwrap();
        let correction_b = CorrectionPhase::new(50).unwrap();
        let new_correction = correction_a.saturating_add(&correction_b);
        assert_eq!(new_correction.value(), CorrectionPhase::MAX);
    }

    #[test]
    fn corrections_add_wrapping() {
        let correction_a = CorrectionDcOffsetI::new(CorrectionDcOffsetI::MAX - 8).unwrap();
        let correction_b = CorrectionDcOffsetI::new(50).unwrap();
        let new_correction = correction_a + correction_b;
        assert_eq!(new_correction.value(), CorrectionDcOffsetI::MIN + 50 - 8);

        let correction_a = CorrectionDcOffsetQ::new(CorrectionDcOffsetQ::MAX - 8).unwrap();
        let correction_b = CorrectionDcOffsetQ::new(50).unwrap();
        let new_correction = correction_a + correction_b;
        assert_eq!(new_correction.value(), CorrectionDcOffsetQ::MIN + 50 - 8);

        let correction_a = CorrectionGain::new(CorrectionGain::MAX - 6).unwrap();
        let correction_b = CorrectionGain::new(50).unwrap();
        let new_correction = correction_a + correction_b;
        assert_eq!(new_correction.value(), CorrectionGain::MIN + 50 - 6);

        let correction_a = CorrectionPhase::new(CorrectionPhase::MAX - 6).unwrap();
        let correction_b = CorrectionPhase::new(50).unwrap();
        let new_correction = correction_a + correction_b;
        assert_eq!(new_correction.value(), CorrectionPhase::MIN + 50 - 6);
    }
}
