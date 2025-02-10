// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use fixed::{types::extra::U11, FixedI16};
use num_complex::Complex;
use strum::FromRepr;

use crate::{sys::*, Error, Result};

pub type ComplexI16 = Complex<i16>;
pub type ComplexI8 = Complex<i8>;

type FixedI11F = FixedI16<U11>;

/// Complex fixed point type with 11 fractional bits to match the 12 bit samples.
/// This way i16 values in the range [-2048, 2048) map to [-1.0, 1.0)
pub type ComplexI12 = Complex<FixedI11F>;

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Format {
    // TODO: See if we can pull in the bladerf docs wholesale
    #[doc = "[`bladerf_format_BLADERF_FORMAT_SC16_Q11`]"]
    Sc16Q11 = bladerf_format_BLADERF_FORMAT_SC16_Q11 as u32,
    #[doc = "[`bladerf_format_BLADERF_FORMAT_SC8_Q7`]"]
    Sc8Q7 = bladerf_format_BLADERF_FORMAT_SC8_Q7 as u32,
    // TODO: implement meta parsing
    // #[doc = "[`bladerf_format_BLADERF_FORMAT_SC16_Q11_META`]"]
    // Sc16Q11Meta = bladerf_format_BLADERF_FORMAT_SC16_Q11_META,
    // #[doc = "[`bladerf_format_BLADERF_FORMAT_PACKET_META`]"]
    // PacketMeta = bladerf_format_BLADERF_FORMAT_PACKET_META,
    // #[doc = "[`bladerf_format_BLADERF_FORMAT_SC8_Q7_META`]"]
    // Sc8Q7Meta = bladerf_format_BLADERF_FORMAT_SC8_Q7_META,
}

impl TryFrom<bladerf_format> for Format {
    type Error = Error;

    fn try_from(format: bladerf_format) -> Result<Self> {
        Self::from_repr(format as u32)
            .ok_or_else(|| format!("Invalid bladerf format: {format}").into())
    }
}

/// Supported sample types from the bladeRF.
///
/// # Safety
/// `is_compatible` must only return true if it is valid to re-interpret bytes from the device as `Self`.
///
/// Currently this is only implemented for:
/// - `Format::Sc16Q11` => [ComplexI16]
/// - `Format::Sc8Q7` => [ComplexI8]
/// - `Format::Sc16Q11` => [ComplexI12]
pub unsafe trait SampleFormat: Sized {
    const FORMAT: Format;

    /// Returns true if this data type is commutable with the given format enum
    fn is_compatible(format: Format) -> bool;

    fn check_compatability(format: Format) -> Result<()> {
        if Self::is_compatible(format) {
            Ok(())
        } else {
            Err(Error::msg(format!(
                "{} is not compatable with configured format {format:?}",
                std::any::type_name::<Self>()
            )))
        }
    }
}

// Implementations for supported types
unsafe impl SampleFormat for ComplexI16 {
    const FORMAT: Format = Format::Sc16Q11;

    fn is_compatible(format: Format) -> bool {
        matches!(format, Format::Sc16Q11)
    }
}

unsafe impl SampleFormat for ComplexI8 {
    const FORMAT: Format = Format::Sc8Q7;

    fn is_compatible(format: Format) -> bool {
        matches!(format, Format::Sc8Q7)
    }
}

unsafe impl SampleFormat for ComplexI12 {
    const FORMAT: Format = Format::Sc16Q11;

    fn is_compatible(format: Format) -> bool {
        matches!(format, Format::Sc16Q11)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_check_fixed_i11() {
        let neg_one = FixedI11F::from_num(-1);
        let inner = neg_one.to_bits();
        assert_eq!(inner, -2048);

        assert_eq!(neg_one.to_num::<f32>(), -1.0_f32);

        // This value would not actually be represented/output by the bladeRF.
        let pos_one = FixedI11F::from_num(1);
        let inner = pos_one.to_bits();
        assert_eq!(inner, 2048);

        assert_eq!(pos_one.to_num::<f32>(), 1.0_f32);
    }
}
