use num_complex::Complex;
use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Format {
    // TODO: See if we can pull in the bladerf docs wholesale
    #[doc = "[`bladerf_format_BLADERF_FORMAT_SC16_Q11`]"]
    Sc16Q11 = bladerf_format_BLADERF_FORMAT_SC16_Q11,
    #[doc = "[`bladerf_format_BLADERF_FORMAT_SC8_Q7`]"]
    Sc8Q7 = bladerf_format_BLADERF_FORMAT_SC8_Q7,
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
        Self::from_repr(format).ok_or_else(|| format!("Invalid bladerf format: {format}").into())
    }
}

/// Supported sample types from the bladeRF.
///
/// # Safety
/// `is_compatible` must only return true if it is valid to re-interpret bytes from the device as `Self`.
///
/// Currently this is only implemented for:
/// - `Format::Sc16Q11` => `Complex<i16>`
/// - `Format::Sc8Q7` => `Complex<i8>`
pub unsafe trait SampleFormat: Sized {
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
unsafe impl SampleFormat for Complex<i16> {
    fn is_compatible(format: Format) -> bool {
        matches!(format, Format::Sc16Q11)
    }
}

unsafe impl SampleFormat for Complex<i8> {
    fn is_compatible(format: Format) -> bool {
        matches!(format, Format::Sc8Q7)
    }
}
