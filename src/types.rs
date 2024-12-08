use crate::{sys::*, BladeRF, Error, Result};

use bytemuck::cast_slice;
use enum_map::Enum;
use num_complex::Complex;
use std::{cmp, ffi::CStr};
use strum::FromRepr;

/// BladeRF module config object
#[derive(Clone, Debug)]
pub struct ModuleConfig {
    pub frequency: u64,
    pub sample_rate: u32,
    pub bandwidth: u32,
    /// Set overall system gain
    pub gain: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    /// Textual description of the release, or None if not available or if not UTF-8
    pub describe: Option<&'static str>,
}

impl Version {
    /// Converts the ffi type `bladerf_version` to `Self`.
    ///
    /// # Safety
    /// `version` must come from a bladerf ffi call.
    /// More specifically:
    /// `version.describe` must be a null-terminated, immutable, statically-allocated (always valid),
    /// string.
    pub unsafe fn from_ffi(version: &bladerf_version) -> Self {
        let describe = if !version.describe.is_null() {
            // SAFETY: bladefr docs on field say do not try to modify or free this,
            // which sounds like a static lifetime to me
            let cstr = unsafe { CStr::from_ptr::<'static>(version.describe) };
            cstr.to_str().ok()
        } else {
            None
        };

        Version {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            describe,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(desc) = self.describe {
            f.write_fmt(format_args!(
                "v{}.{}.{} ({})",
                self.major, self.minor, self.patch, desc
            ))
        } else {
            f.write_fmt(format_args!(
                "v{}.{}.{}",
                self.major, self.minor, self.patch,
            ))
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let major_ord = self.major.cmp(&other.major);
        if major_ord != cmp::Ordering::Equal {
            return major_ord;
        }
        let minor_ord = self.minor.cmp(&other.minor);
        if minor_ord != cmp::Ordering::Equal {
            return minor_ord;
        }
        self.patch.cmp(&other.patch)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for Version {}

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

pub struct RationalRate {
    /// Integer portion
    pub integer: u64,
    /// Numerator in fractional portion
    pub num: u64,
    /// Denominator in fractional portion. This must be greater than 0
    pub den: u64,
}

impl From<bladerf_rational_rate> for RationalRate {
    fn from(rate: bladerf_rational_rate) -> Self {
        Self {
            integer: rate.integer,
            num: rate.num,
            den: rate.den,
        }
    }
}

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Backend {
    Any = bladerf_backend_BLADERF_BACKEND_ANY as i32,
    Linux = bladerf_backend_BLADERF_BACKEND_LINUX as i32,
    LibUsb = bladerf_backend_BLADERF_BACKEND_LIBUSB as i32,
    Cypress = bladerf_backend_BLADERF_BACKEND_CYPRESS as i32,
    Dummy = bladerf_backend_BLADERF_BACKEND_DUMMY as i32,
}

impl TryFrom<bladerf_backend> for Backend {
    type Error = Error;

    fn try_from(backend: bladerf_backend) -> Result<Self> {
        Self::from_repr(backend as i32)
            .ok_or_else(|| format!("Invalid bladerf backend: {backend}").into())
    }
}

impl From<Backend> for bladerf_backend {
    fn from(value: Backend) -> Self {
        value as i32 as bladerf_backend
    }
}

/// Information about a bladerf device connect to the system
#[derive(Clone, Debug)]
pub struct DevInfo(pub(crate) bladerf_devinfo);

impl DevInfo {
    pub fn backend(&self) -> Result<Backend> {
        self.0.backend.try_into()
    }
    pub fn serial(&self) -> String {
        String::from_utf8_lossy(cast_slice(&self.0.serial[..32])).to_string()
    }
    pub fn usb_bus(&self) -> Option<u8> {
        Some(self.0.usb_bus)
    }
    pub fn usb_addr(&self) -> Option<u8> {
        Some(self.0.usb_addr)
    }
    pub fn instance(&self) -> u32 {
        self.0.instance
    }
    pub fn manufacturer(&self) -> String {
        // TODO: This seems to be `Nuandwn>` instead of `Nuandwn` (what bladeRF-cli --probe gets)
        String::from_utf8_lossy(cast_slice(&self.0.manufacturer)).to_string()
    }
    pub fn product(&self) -> String {
        String::from_utf8_lossy(cast_slice(&self.0.product)).to_string()
    }

    pub fn open(&self) -> Result<BladeRF> {
        BladeRF::open_with_devinfo(self)
    }
}

impl From<bladerf_devinfo> for DevInfo {
    fn from(dev: bladerf_devinfo) -> Self {
        Self(dev)
    }
}

/// Combined RX and TX config
pub struct Config {
    pub tx: ModuleConfig,
    pub rx: ModuleConfig,
}

#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Channel {
    Rx1 = bladerf_channel_layout_BLADERF_RX_X1 as i32,
    Rx2 = bladerf_channel_layout_BLADERF_RX_X2 as i32,
    Tx1 = bladerf_channel_layout_BLADERF_TX_X1 as i32,
    Tx2 = bladerf_channel_layout_BLADERF_TX_X2 as i32,
}

impl Channel {
    pub fn is_rx(&self) -> bool {
        matches!(self, Channel::Rx1 | Channel::Rx2)
    }
    pub fn is_tx(&self) -> bool {
        matches!(self, Channel::Tx1 | Channel::Tx2)
    }
}

impl TryFrom<bladerf_channel> for Channel {
    type Error = Error;

    fn try_from(channel: bladerf_channel) -> Result<Self> {
        Self::from_repr(channel).ok_or_else(|| format!("Invalid bladerf channel: {channel}").into())
    }
}

// Additional types for Metadata
#[derive(Clone, Debug)]
pub struct Metadata {
    pub timestamp: u64,
    pub flags: u32,
    // Add other fields as necessary
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            timestamp: 0,
            flags: 0,
        }
    }
}

impl From<&bladerf_metadata> for Metadata {
    fn from(meta: &bladerf_metadata) -> Self {
        Self {
            timestamp: meta.timestamp,
            flags: meta.flags,
        }
    }
}

impl From<&Metadata> for bladerf_metadata {
    fn from(val: &Metadata) -> Self {
        bladerf_metadata {
            timestamp: val.timestamp,
            flags: val.flags,
            status: 0,
            actual_count: 0,
            reserved: [0u8; 32],
        }
    }
}

// Direction Enum
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Direction {
    RX = bladerf_direction_BLADERF_RX,
    TX = bladerf_direction_BLADERF_TX,
}

impl From<Direction> for bladerf_direction {
    fn from(dir: Direction) -> Self {
        dir as bladerf_direction
    }
}

impl TryFrom<bladerf_direction> for Direction {
    type Error = Error;

    fn try_from(value: bladerf_direction) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid Direction value: {value}")))
    }
}

/// Loopback configuration
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Loopback {
    None = bladerf_loopback_BLADERF_LB_NONE,
    RfLna1 = bladerf_loopback_BLADERF_LB_RF_LNA1,
    RfLna2 = bladerf_loopback_BLADERF_LB_RF_LNA2,
    RfLna3 = bladerf_loopback_BLADERF_LB_RF_LNA3,
    Firmware = bladerf_loopback_BLADERF_LB_FIRMWARE,
    RficBist = bladerf_loopback_BLADERF_LB_RFIC_BIST,
    BbTxlpfRxlpf = bladerf_loopback_BLADERF_LB_BB_TXLPF_RXLPF,
    BbTxlpfRxvga2 = bladerf_loopback_BLADERF_LB_BB_TXLPF_RXVGA2,
    BbTxvga1Rxlpf = bladerf_loopback_BLADERF_LB_BB_TXVGA1_RXLPF,
    BbTxvga1Rxvga2 = bladerf_loopback_BLADERF_LB_BB_TXVGA1_RXVGA2,
}

impl TryFrom<bladerf_loopback> for Loopback {
    type Error = Error;

    fn try_from(loopback: bladerf_loopback) -> Result<Self> {
        Self::from_repr(loopback)
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
            mode: Loopback::from_repr(mode_info.mode).unwrap_or(Loopback::None),
        }
    }
}

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum Format {
    // TODO: See if we can pull in the bladerf docs wholesale
    #[doc = "[`bladerf_format_BLADERF_FORMAT_SC16_Q11`]"]
    Sc16Q11 = bladerf_format_BLADERF_FORMAT_SC16_Q11,
    #[doc = "[`bladerf_format_BLADERF_FORMAT_SC8_Q7`]"]
    Sc8Q7 = bladerf_format_BLADERF_FORMAT_SC8_Q7,
    // TODO: implement meta parsing
    // #[doc = "[`bladerf_format_BLADERF_FORMAT_SC16_Q11_META`]"]
    // Sc16Q11Meta = bladerf_format_BLADERF_FORMAT_SC16_Q11_META,
    // #[doc = "[`bladerf_format_BLADERF_FORMAT_PACKET_META`]"]
    // PacketMeta = bladerf_format_BLADERF_FORMAT_PACKET_META,
    // #[doc = "[`bladerf_format_BLADERF_FORMAT_SC8_Q7_META`]"]
    // Sc8Q7Meta = bladerf_format_BLADERF_FORMAT_SC8_Q7_META,
}

impl TryFrom<bladerf_format> for Format {
    type Error = Error;

    fn try_from(format: bladerf_format) -> Result<Self> {
        Self::from_repr(format).ok_or_else(|| format!("Invalid bladerf format: {format}").into())
    }
}

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Sampling {
    Unknown = bladerf_sampling_BLADERF_SAMPLING_UNKNOWN as i32,
    Internal = bladerf_sampling_BLADERF_SAMPLING_INTERNAL as i32,
    External = bladerf_sampling_BLADERF_SAMPLING_EXTERNAL as i32,
}

impl TryFrom<bladerf_sampling> for Sampling {
    type Error = Error;

    fn try_from(value: bladerf_sampling) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid Sampling value: {value}")))
    }
}

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum RxMux {
    Invalid = bladerf_rx_mux_BLADERF_RX_MUX_INVALID,
    Baseband = bladerf_rx_mux_BLADERF_RX_MUX_BASEBAND,
    Counter12bit = bladerf_rx_mux_BLADERF_RX_MUX_12BIT_COUNTER,
    Counter32bit = bladerf_rx_mux_BLADERF_RX_MUX_32BIT_COUNTER,
    DigitalLoopback = bladerf_rx_mux_BLADERF_RX_MUX_DIGITAL_LOOPBACK,
}

impl TryFrom<bladerf_rx_mux> for RxMux {
    type Error = Error;

    fn try_from(value: bladerf_rx_mux) -> Result<Self> {
        Self::from_repr(value).ok_or_else(|| Error::msg(format!("Invalid RxMux value: {value}")))
    }
}

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum LPFMode {
    Normal = bladerf_lpf_mode_BLADERF_LPF_NORMAL as i32,
    Bypassed = bladerf_lpf_mode_BLADERF_LPF_BYPASSED as i32,
    Disabled = bladerf_lpf_mode_BLADERF_LPF_DISABLED as i32,
}

impl TryFrom<bladerf_lpf_mode> for LPFMode {
    type Error = Error;

    fn try_from(value: bladerf_lpf_mode) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid LPFMode value: {value}")))
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct QuickTune {
    pub freqsel: u8,
    pub vcocap: u8,
    pub nint: u16,
    pub nfrac: u32,
    pub flags: u8,
}

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TuningMode {
    Host = bladerf_tuning_mode_BLADERF_TUNING_MODE_HOST,
    FPGA = bladerf_tuning_mode_BLADERF_TUNING_MODE_FPGA,
    Invalid = bladerf_tuning_mode_BLADERF_TUNING_MODE_INVALID,
}

impl TryFrom<bladerf_tuning_mode> for TuningMode {
    type Error = Error;

    fn try_from(value: bladerf_tuning_mode) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid TuningMode value: {value}")))
    }
}

/// Gain value, in decibels (dB)
pub type Gain = i32;

/// Gain control modes
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

/// Range struct to represent `bladerf_range`
#[derive(Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl Range {
    pub fn contains(&self, query: impl Into<u64>) -> bool {
        let steps = (query.into() as f64 - self.min) / self.step;
        steps % 1.0 < 1e-8
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:.0}..{:.0} (step {:.0})",
            self.min, self.max, self.step,
        ))
    }
}

impl From<&bladerf_range> for Range {
    fn from(range: &bladerf_range) -> Self {
        Self {
            min: range.min as f64 * range.scale as f64,
            max: range.max as f64 * range.scale as f64,
            step: range.step as f64 * range.scale as f64,
        }
    }
}

/// Correction value, in arbitrary units
///
/// Units taken from here: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___c_o_r_r.html#ga75dd741fde93fecb4d514a1f9a377344>
///
/// Type validation is done to ensure the values are in the correct range, returning None if they are not.
///
/// | Enum Vaiant | Units |
/// |---------|---------|
/// | DcOffsetI | Adjusts the in-phase DC offset. Valid values are [-2048, 2048], which are scaled to the available control bits. |
/// | DcOffsetQ | Adjusts the quadrature DC offset. Valid values are [-2048, 2048], which are scaled to the available control bits. |
/// | Phase | Adjusts phase correction of [-10, 10] degrees, via a provided count value of [-4096, 4096]. |
/// | Gain | Adjusts gain correction value in [-1.0, 1.0], via provided values in the range of [-4096, 4096]. |

#[derive(Debug, Clone, Copy)]
pub enum CorrectionValue {
    DcOffsetI(i16),
    DcOffsetQ(i16),
    Phase(i16),
    Gain(i16),
}

impl CorrectionValue {
    pub fn new_gain(gain: i16) -> Option<CorrectionValue> {
        match gain {
            -4096..4096 => Some(CorrectionValue::Gain(gain)),
            _ => None,
        }
    }

    pub fn new_phase(phase: i16) -> Option<CorrectionValue> {
        match phase {
            -4096..4096 => Some(CorrectionValue::Phase(phase)),
            _ => None,
        }
    }

    pub fn new_dc_offset_i(offset: i16) -> Option<CorrectionValue> {
        match offset {
            -2048..2048 => Some(CorrectionValue::DcOffsetI(offset)),
            _ => None,
        }
    }

    pub fn new_dc_offset_q(offset: i16) -> Option<CorrectionValue> {
        match offset {
            -2048..2048 => Some(CorrectionValue::DcOffsetQ(offset)),
            _ => None,
        }
    }

    /// # Safety
    /// This does not do type validation.
    pub unsafe fn new_from_raw(corr: Correction, value: i16) -> CorrectionValue {
        match corr {
            Correction::DcOffsetI => CorrectionValue::DcOffsetI(value),
            Correction::DcOffsetQ => CorrectionValue::DcOffsetQ(value),
            Correction::Phase => CorrectionValue::Gain(value),
            Correction::Gain => CorrectionValue::Phase(value),
        }
    }

    pub fn into_inner(self) -> i16 {
        match self {
            CorrectionValue::DcOffsetI(val) => val,
            CorrectionValue::DcOffsetQ(val) => val,
            CorrectionValue::Phase(val) => val,
            CorrectionValue::Gain(val) => val,
        }
    }
}

impl From<CorrectionValue> for Correction {
    fn from(value: CorrectionValue) -> Self {
        match value {
            CorrectionValue::DcOffsetI(_) => Correction::DcOffsetI,
            CorrectionValue::DcOffsetQ(_) => Correction::DcOffsetQ,
            CorrectionValue::Phase(_) => Correction::Phase,
            CorrectionValue::Gain(_) => Correction::Gain,
        }
    }
}

/// Correction parameter selection
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Correction {
    DcOffsetI = bladerf_correction_BLADERF_CORR_DCOFF_I as i32,
    DcOffsetQ = bladerf_correction_BLADERF_CORR_DCOFF_Q as i32,
    Phase = bladerf_correction_BLADERF_CORR_PHASE as i32,
    Gain = bladerf_correction_BLADERF_CORR_GAIN as i32,
}

impl TryFrom<bladerf_correction> for Correction {
    type Error = Error;

    fn try_from(value: bladerf_correction) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid Correction value: {value}")))
    }
}

/// Trigger role
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TriggerRole {
    Invalid = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_INVALID,
    Disabled = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_DISABLED,
    Master = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_MASTER,
    Slave = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_SLAVE,
}

impl TryFrom<bladerf_trigger_role> for TriggerRole {
    type Error = Error;

    fn try_from(value: bladerf_trigger_role) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid TriggerRole value: {value}")))
    }
}

/// Trigger signal selection
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TriggerSignal {
    Invalid = bladerf_trigger_signal_BLADERF_TRIGGER_INVALID,
    J71_4 = bladerf_trigger_signal_BLADERF_TRIGGER_J71_4,
    J51_1 = bladerf_trigger_signal_BLADERF_TRIGGER_J51_1,
    MiniExp1 = bladerf_trigger_signal_BLADERF_TRIGGER_MINI_EXP_1,
    User0 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_0,
    User1 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_1,
    User2 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_2,
    User3 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_3,
    User4 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_4,
    User5 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_5,
    User6 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_6,
    User7 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_7,
}

impl TryFrom<bladerf_trigger_signal> for TriggerSignal {
    type Error = Error;

    fn try_from(value: bladerf_trigger_signal) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid TriggerSignal value: {value}")))
    }
}

/// Trigger configuration
pub struct Trigger {
    pub channel: Channel,
    pub role: TriggerRole,
    pub signal: TriggerSignal,
    pub options: u64,
}

impl TryFrom<bladerf_trigger> for Trigger {
    type Error = Error;

    fn try_from(t: bladerf_trigger) -> Result<Self> {
        Ok(Self {
            channel: t.channel.try_into()?,
            role: t.role.try_into()?,
            signal: t.signal.try_into()?,
            options: t.options,
        })
    }
}

/// Supported sample types from the bladeRF.
///
/// # Safety
/// `is_compatible` must only return true if it is valid to re-interpret bytes from the device as `Self`.
///
/// Currently this is only implemented for:
/// - `Format::Sc16Q11` => `Complex<i16>`
/// - `Format::Sc8Q7` => `Complex<i8>`
pub unsafe trait SampleFormat: Sized {
    /// Returns true if this data type is commutable with the given format enum
    fn is_compatible(format: Format) -> bool;

    fn check_compatability(format: Format) -> Result<()> {
        if Self::is_compatible(format) {
            Ok(())
        } else {
            Err(Error::msg(format!(
                "{} is not compatable with configured format {format:?}",
                std::any::type_name::<Self>()
            )))
        }
    }
}

// Implementations for supported types
unsafe impl SampleFormat for Complex<i16> {
    fn is_compatible(format: Format) -> bool {
        matches!(format, Format::Sc16Q11)
    }
}

unsafe impl SampleFormat for Complex<i8> {
    fn is_compatible(format: Format) -> bool {
        matches!(format, Format::Sc8Q7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_cmp() {
        let v1 = Version {
            major: 2,
            minor: 0,
            patch: 0,
            describe: None,
        };
        let v2 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: None,
        };

        assert!(v1 > v2);
        assert!(v2 < v1);

        let v1 = Version {
            major: 1,
            minor: 6,
            patch: 0,
            describe: None,
        };
        let v2 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: None,
        };

        assert!(v1 > v2);
        assert!(v2 < v1);

        let v1 = Version {
            major: 1,
            minor: 5,
            patch: 11,
            describe: None,
        };
        let v2 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: None,
        };

        assert!(v1 > v2);
        assert!(v2 < v1);

        let v1 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: Some("test"),
        };
        let v2 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: Some("another test"),
        };

        assert_eq!(v1, v2);

        let v1 = Version {
            major: 1,
            minor: 5,
            patch: 11,
            describe: None,
        };
        let v2 = Version {
            major: 1,
            minor: 6,
            patch: 0,
            describe: None,
        };
        let v3 = Version {
            major: 2,
            minor: 0,
            patch: 0,
            describe: None,
        };

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }
}
