// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum LogLevel {
    Verbose = bladerf_log_level_BLADERF_LOG_LEVEL_VERBOSE as u32,
    Debug = bladerf_log_level_BLADERF_LOG_LEVEL_DEBUG as u32,
    Info = bladerf_log_level_BLADERF_LOG_LEVEL_INFO as u32,
    Warning = bladerf_log_level_BLADERF_LOG_LEVEL_WARNING as u32,
    Error = bladerf_log_level_BLADERF_LOG_LEVEL_ERROR as u32,
    Critical = bladerf_log_level_BLADERF_LOG_LEVEL_CRITICAL as u32,
    Silent = bladerf_log_level_BLADERF_LOG_LEVEL_SILENT as u32,
}

impl TryFrom<bladerf_log_level> for LogLevel {
    type Error = Error;

    fn try_from(level: bladerf_log_level) -> Result<Self> {
        Self::from_repr(level as u32)
            .ok_or_else(|| format!("Invalid bladerf log level: {level}").into())
    }
}
