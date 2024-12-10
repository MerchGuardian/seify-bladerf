use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum LogLevel {
    Verbose = bladerf_log_level_BLADERF_LOG_LEVEL_VERBOSE,
    Debug = bladerf_log_level_BLADERF_LOG_LEVEL_DEBUG,
    Info = bladerf_log_level_BLADERF_LOG_LEVEL_INFO,
    Warning = bladerf_log_level_BLADERF_LOG_LEVEL_WARNING,
    Error = bladerf_log_level_BLADERF_LOG_LEVEL_ERROR,
    Critical = bladerf_log_level_BLADERF_LOG_LEVEL_CRITICAL,
    Silent = bladerf_log_level_BLADERF_LOG_LEVEL_SILENT,
}

impl TryFrom<bladerf_log_level> for LogLevel {
    type Error = Error;

    fn try_from(level: bladerf_log_level) -> Result<Self> {
        Self::from_repr(level).ok_or_else(|| format!("Invalid bladerf log level: {level}").into())
    }
}
