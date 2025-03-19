use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Describes the sampling modes/connection of the LMS6002D
///
/// This allows the user to switch between the "normal" sampling and direct samplling from the DACs and ADCs
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___s_a_m_p_l_i_n_g___m_u_x.html#gac8be10b9045b236e2bd4d705bde4b5db>
// TODO Does this apply for the BladeRF 2.0 Micro?
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Sampling {
    /// Unable to determine the sampling type
    Unknown = bladerf_sampling_BLADERF_SAMPLING_UNKNOWN as i32,
    /// Get samples from the RX/TX connector (ie: samples from the down/up-conversion to the RX/TX ports).
    ///
    /// This is the normal mode of operation for an SDR
    Internal = bladerf_sampling_BLADERF_SAMPLING_INTERNAL as i32,
    /// Direct DAC/ADC sampling from the J60 and J61 headers
    // TODO Does this apply for the BladeRF 2.0 Micro?
    External = bladerf_sampling_BLADERF_SAMPLING_EXTERNAL as i32,
}

impl TryFrom<bladerf_sampling> for Sampling {
    type Error = Error;

    fn try_from(value: bladerf_sampling) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid Sampling value: {value}")))
    }
}
