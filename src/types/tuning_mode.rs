use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// The frequency tuning mode
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_u_n_i_n_g___m_o_d_e.html#ga1052a36566cb6dc311242981c9ab4c47>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TuningMode {
    /// Perform tuning algorithm on the host. This is slower, but provides easier accessiblity to diagnostic information.
    Host = bladerf_tuning_mode_BLADERF_TUNING_MODE_HOST,
    /// Perform tuning algorithm on the FPGA for faster tuning.
    FPGA = bladerf_tuning_mode_BLADERF_TUNING_MODE_FPGA,
    /// An invalid mode is set
    Invalid = bladerf_tuning_mode_BLADERF_TUNING_MODE_INVALID,
}

impl TryFrom<bladerf_tuning_mode> for TuningMode {
    type Error = Error;

    fn try_from(value: bladerf_tuning_mode) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid TuningMode value: {value}")))
    }
}
