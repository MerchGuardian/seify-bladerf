use crate::sys::*;

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
