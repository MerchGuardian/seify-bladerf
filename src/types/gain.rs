use std::ffi::CStr;

use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Gain value, in decibels (dB)
pub type Gain = i32;

/// Gain control modes
///
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#gae7632e9f6b3a5a182ef012c214be0f78>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum GainMode {
    /// Device-specific default (automatic, when available)
    Default = bladerf_gain_mode_BLADERF_GAIN_DEFAULT as i32,
    /// Manual gain control
    Manual = bladerf_gain_mode_BLADERF_GAIN_MGC as i32,
    /// Automatic gain control, fast attack (advanced)
    FastAttackAgc = bladerf_gain_mode_BLADERF_GAIN_FASTATTACK_AGC as i32,
    /// Automatic gain control, slow attack (advanced)
    SlowAttackAgc = bladerf_gain_mode_BLADERF_GAIN_SLOWATTACK_AGC as i32,
    /// Automatic gain control, hybrid attack (advanced)
    HybridAgc = bladerf_gain_mode_BLADERF_GAIN_HYBRID_AGC as i32,
}

impl TryFrom<bladerf_gain_mode> for GainMode {
    type Error = Error;

    fn try_from(value: bladerf_gain_mode) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid GainMode value: {value}")))
    }
}

/// Mapping between C string description of gain modes and `GainMode`
pub struct GainModeInfo {
    pub name: &'static str,
    pub mode: GainMode,
}

impl From<bladerf_gain_modes> for GainModeInfo {
    fn from(mode_info: bladerf_gain_modes) -> Self {
        let name = unsafe { CStr::from_ptr(mode_info.name) }
            .to_str()
            .unwrap_or("Unknown");
        Self {
            name,
            mode: GainMode::from_repr(mode_info.mode as i32).unwrap_or(GainMode::Default),
        }
    }
}
