// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use std::ffi::CStr;

use crate::{sys::*, Error, Result};
use strum::FromRepr;

/// Loopback configuration
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Loopback {
    None = bladerf_loopback_BLADERF_LB_NONE as u32,
    RfLna1 = bladerf_loopback_BLADERF_LB_RF_LNA1 as u32,
    RfLna2 = bladerf_loopback_BLADERF_LB_RF_LNA2 as u32,
    RfLna3 = bladerf_loopback_BLADERF_LB_RF_LNA3 as u32,
    Firmware = bladerf_loopback_BLADERF_LB_FIRMWARE as u32,
    RficBist = bladerf_loopback_BLADERF_LB_RFIC_BIST as u32,
    BbTxlpfRxlpf = bladerf_loopback_BLADERF_LB_BB_TXLPF_RXLPF as u32,
    BbTxlpfRxvga2 = bladerf_loopback_BLADERF_LB_BB_TXLPF_RXVGA2 as u32,
    BbTxvga1Rxlpf = bladerf_loopback_BLADERF_LB_BB_TXVGA1_RXLPF as u32,
    BbTxvga1Rxvga2 = bladerf_loopback_BLADERF_LB_BB_TXVGA1_RXVGA2 as u32,
}

impl TryFrom<bladerf_loopback> for Loopback {
    type Error = Error;

    fn try_from(loopback: bladerf_loopback) -> Result<Self> {
        Self::from_repr(loopback as u32)
            .ok_or_else(|| format!("Invalid bladerf loopback mode: {loopback}").into())
    }
}

pub struct LoopbackModeInfo {
    pub name: Option<String>,
    pub mode: Loopback,
}

impl From<bladerf_loopback_modes> for LoopbackModeInfo {
    fn from(mode_info: bladerf_loopback_modes) -> Self {
        let name = unsafe { CStr::from_ptr(mode_info.name) }
            .to_str()
            .map(|s| s.to_string())
            .ok();
        Self {
            name,
            mode: Loopback::from_repr(mode_info.mode as u32).unwrap_or(Loopback::None),
        }
    }
}
