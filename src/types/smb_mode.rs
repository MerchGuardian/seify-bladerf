use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// The maximum frequency in Hz that can be output on the SMB port. (If no expansion board is attached)
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#gaddcd05fff8cbcca5d08d45b4d4fa32cf>
pub const SMB_FREQUENCY_MAX: u32 = BLADERF_SMB_FREQUENCY_MAX;
/// The minimum frequency in Hz that can be output on the SMB port. (If no expansion board is attached)
///
/// Defined as `((38400000u * 66u) / (32 * 567))`
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#gada1f2c5610e68e414342d8e152a245b0>
pub const SMB_FREQUENCY_MIN: u32 = BLADERF_SMB_FREQUENCY_MIN;

/// Represents the configuration of the SMB Clock port (J62)
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#gad289c8e261a1f7342e9280f22a844563>
// TODO: is this valid on the bladerf 2.0 micro?
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum SmbMode {
    /// Invalid selection
    Invalid = bladerf_smb_mode_BLADERF_SMB_MODE_INVALID,
    /// SMB port signal not used, Device operates from the onboard clock.
    Disabled = bladerf_smb_mode_BLADERF_SMB_MODE_DISABLED,
    /// SMB port outputs a 38.4 MHz clock, typically used to drive another BladeRF with [SmbMode::Input]
    Output = bladerf_smb_mode_BLADERF_SMB_MODE_OUTPUT,
    /// SMB port configured as input and expects 38.4 MHz clock signal.
    Input = bladerf_smb_mode_BLADERF_SMB_MODE_INPUT,
    /// SMB port is not available since the clock signal is being used elsewhere (eg: Expansion Board)
    Unavailable = bladerf_smb_mode_BLADERF_SMB_MODE_UNAVAILBLE,
}

impl TryFrom<bladerf_smb_mode> for SmbMode {
    type Error = Error;

    fn try_from(value: bladerf_smb_mode) -> Result<Self> {
        Self::from_repr(value).ok_or_else(|| Error::msg(format!("Invalid Sampling value: {value}")))
    }
}
