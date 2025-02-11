// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum FpgaSize {
    Unknown = bladerf_fpga_size_BLADERF_FPGA_UNKNOWN as u32,
    Kle40 = bladerf_fpga_size_BLADERF_FPGA_40KLE as u32,
    Kle115 = bladerf_fpga_size_BLADERF_FPGA_115KLE as u32,
    A4 = bladerf_fpga_size_BLADERF_FPGA_A4 as u32,
    A5 = bladerf_fpga_size_BLADERF_FPGA_A5 as u32,
    A9 = bladerf_fpga_size_BLADERF_FPGA_A9 as u32,
}

impl FpgaSize {
    pub fn logic_element_count_kle(&self) -> Option<u32> {
        match self {
            Self::Unknown => None,
            _ => Some(*self as u32),
        }
    }
}

impl From<FpgaSize> for bladerf_fpga_size {
    fn from(dir: FpgaSize) -> Self {
        dir as bladerf_fpga_size
    }
}

impl TryFrom<bladerf_fpga_size> for FpgaSize {
    type Error = Error;

    fn try_from(value: bladerf_fpga_size) -> Result<Self> {
        Self::from_repr(value as u32)
            .ok_or_else(|| Error::msg(format!("Invalid FPGA discriminant: {value}")))
    }
}
