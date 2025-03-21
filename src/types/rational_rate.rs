use crate::sys::*;

/// Rational sample rate representation
///
/// `rate = integer + (num/den)`
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__rational__rate.html>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RationalRate {
    /// Integer portion
    pub integer: u64,
    /// Numerator in fractional portion
    pub num: u64,
    /// Denominator in fractional portion. This must be greater than 0
    pub den: u64,
}

impl From<bladerf_rational_rate> for RationalRate {
    fn from(rate: bladerf_rational_rate) -> Self {
        Self {
            integer: rate.integer,
            num: rate.num,
            den: rate.den,
        }
    }
}

impl From<RationalRate> for bladerf_rational_rate {
    fn from(value: RationalRate) -> Self {
        bladerf_rational_rate {
            integer: value.integer,
            num: value.num,
            den: value.den,
        }
    }
}
