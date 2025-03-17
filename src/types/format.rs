// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use fixed::{types::extra::U11, FixedI16};
use num_complex::{Complex, Complex32};
use strum::FromRepr;

use crate::{sys::*, Error, Result};

pub const BRF_CI16_SAMPLE_MAX: i16 = 2047;
pub const BRF_CI16_SAMPLE_MIN: i16 = -2048;

const BRF_CI16_SCALAR: f32 = (BRF_CI16_SAMPLE_MAX + 1) as f32;

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

#[inline]
pub fn brf_ci12_to_cf32(sample: ComplexI12) -> Complex32 {
    let re: f32 = sample.re.to_num::<f32>();
    let im: f32 = sample.im.to_num::<f32>();
    Complex::new(re, im)
}

#[inline]
pub fn brf_cf32_to_ci12(sample: Complex32) -> ComplexI12 {
    let re = FixedI11F::from_num(sample.re);
    let im = FixedI11F::from_num(sample.im);
    ComplexI12::new(re, im)
}

/// This is a function to convert `Complex<i16>` into `Complex<f32>` specifically for use with the bladerf.
///
/// It converts [i16] on the range [-2048, 2048) to [f32] on the range [-1.0, 1.0).
/// As the BladeRF documentation mentions, the user is responsible for making sure the samples are in the range [-2048, 2047].
/// Violating this, while not undefined behavior (sample gets clipped/truncated), is undesired behavior.
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___s_t_r_e_a_m_i_n_g___f_o_r_m_a_t.html#ga4c61587834fd4de51a8e2d34e14a73b2>
#[inline]
pub fn brf_ci16_to_cf32(sample: ComplexI16) -> Complex32 {
    let re: f32 = f32::from(sample.re) / BRF_CI16_SCALAR;
    let im: f32 = f32::from(sample.im) / BRF_CI16_SCALAR;
    Complex::new(re, im)
}

/// This is a function to convert `Complex<f32>` into `Complex<i16>` specifically for use with the bladerf.
///
/// It converts [f32] on the range [-1.0, 1.0) to [i16] on the range [-2048, 2048).
/// As the BladeRF documentation mentions, the user is responsible for making sure the samples are in the range [-2048, 2047].
/// Violating this, while not undefined behavior (sample gets clipped/truncated), is undesired behavior.
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___s_t_r_e_a_m_i_n_g___f_o_r_m_a_t.html#ga4c61587834fd4de51a8e2d34e14a73b2>
#[inline]
pub fn brf_cf32_to_ci16(sample: Complex32) -> ComplexI16 {
    let re = (sample.re * BRF_CI16_SCALAR) as i16;
    let im = (sample.im * BRF_CI16_SCALAR) as i16;
    Complex::new(re, im)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_check_fixed_i11() {
        let neg_one = FixedI11F::from_num(-1);
        let inner = neg_one.to_bits();
        assert_eq!(inner, BRF_CI16_SAMPLE_MIN);

        assert_eq!(neg_one.to_num::<f32>(), -1.0_f32);

        // This value would not actually be represented/output by the bladeRF.
        let pos_one = FixedI11F::from_num(1);
        let inner = pos_one.to_bits();
        // In practice, `pos_one` should never be read or written to the bladerf since it has a representation of
        // 0b0001_0000_0000_0000 which is out of the range of the BladeRF's DAC.
        // Thus this comparison to BRF_CI16_SAMPLE_MAX + 1 is fine.
        assert_eq!(inner, BRF_CI16_SAMPLE_MAX + 1);

        assert_eq!(pos_one.to_num::<f32>(), 1.0_f32);
    }

    #[test]
    fn ci16_to_cf32_conversions() {
        let x = ComplexI16::new(-2048, 1024);
        let y = brf_ci16_to_cf32(x);
        assert_eq!(y, Complex32::new(-1.0, 0.5));
    }

    #[test]
    fn cf32_to_ci16_conversions() {
        let x = Complex32::new(-1.0, 0.5);
        let y = brf_cf32_to_ci16(x);
        assert_eq!(y, ComplexI16::new(-2048, 1024));
    }
}
