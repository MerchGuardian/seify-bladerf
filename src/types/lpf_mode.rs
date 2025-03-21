use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// The Low Pass Filter bypass mode for the LMS6002D
///
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___l_p_f___b_y_p_a_s_s.html>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum LPFMode {
    /// The LPF is connected an enabled
    Normal = bladerf_lpf_mode_BLADERF_LPF_NORMAL as i32,
    /// The LPF is bypassed
    // TODO: Find better description
    Bypassed = bladerf_lpf_mode_BLADERF_LPF_BYPASSED as i32,
    /// The LPF is disabled
    // TODO: Find better description
    Disabled = bladerf_lpf_mode_BLADERF_LPF_DISABLED as i32,
}

impl TryFrom<bladerf_lpf_mode> for LPFMode {
    type Error = Error;

    fn try_from(value: bladerf_lpf_mode) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid LPFMode value: {value}")))
    }
}
