use strum::FromRepr;

use crate::{sys::*, Error, Result};

pub const SMB_FREQUENCY_MAX: u32 = BLADERF_SMB_FREQUENCY_MAX;
pub const SMB_FREQUENCY_MIN: u32 = BLADERF_SMB_FREQUENCY_MIN;

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum SmbMode {
    Invalid = bladerf_smb_mode_BLADERF_SMB_MODE_INVALID,
    Disabled = bladerf_smb_mode_BLADERF_SMB_MODE_DISABLED,
    Output = bladerf_smb_mode_BLADERF_SMB_MODE_OUTPUT,
    Input = bladerf_smb_mode_BLADERF_SMB_MODE_INPUT,
    Unavailable = bladerf_smb_mode_BLADERF_SMB_MODE_UNAVAILBLE,
}

impl TryFrom<bladerf_smb_mode> for SmbMode {
    type Error = Error;

    fn try_from(value: bladerf_smb_mode) -> Result<Self> {
        Self::from_repr(value).ok_or_else(|| Error::msg(format!("Invalid Sampling value: {value}")))
    }
}
