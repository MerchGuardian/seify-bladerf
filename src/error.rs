use embedded_hal::digital::ErrorKind;
use thiserror::Error;

/// A convenience type with [Err] variant set to be the [enum@Error] type from this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for the operations of BladeRF.
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___r_e_t_c_o_d_e_s.html>
/// See also: <https://github.com/Nuand/bladeRF/blob/fe3304d75967c88ab4f17ff37cb5daf8ff53d3e1/host/libraries/libbladeRF/src/bladerf.c#L1784-L1829>
// TODO: Can we get the #[error] macro to generate the doc comments?
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("An unexpected error occurred")]
    /// An unexpected error occurred
    Unexpected,

    #[error("Provided parameter was out of the allowable range")]
    /// Provided parameter was out of the allowable range
    Range,

    #[error("Invalid operation or parameter")]
    /// Invalid operation or parameter
    Inval,

    #[error("A memory allocation error occurred")]
    /// A memory allocation error occurred
    MEM,

    #[error("File or device I/O failure")]
    /// File or device I/O failure
    IO,

    #[error("Operation timed out")]
    /// Operation timed out
    Timeout,

    #[error("No devices available")]
    /// No devices available
    Nodev,

    #[error("Operation not supported")]
    /// Operation not supported
    Unsupported,

    #[error("Misaligned flash access")]
    /// Misaligned flash access
    Misaligned,

    #[error("Invalid checksum")]
    /// Invalid checksum
    CHECKSUM,

    #[error("File not found")]
    /// File not found
    NoFile,

    #[error("An FPGA update is required")]
    /// An FPGA update is required
    UpdateFpga,

    #[error("A firmware update is required")]
    /// A firmware update is required
    UpdateFw,

    #[error("Requested timestamp is in the past")]
    /// Requested timestamp is in the past
    TimePast,

    #[error("Could not enqueue data into full queue")]
    /// Could not enqueue data into full queue
    QueueFull,

    #[error("An FPGA operation reported a failure")]
    /// An FPGA operation reported a failure
    FpgaOp,

    #[error("Insufficient permissions for the requested operation")]
    /// Insufficient permissions for the requested operation
    Permission,

    #[error("The operation would block, but has been requested to be non-blocking")]
    /// The operation would block, but has been requested to be non-blocking
    WouldBlock,

    #[error("Insufficient initialization for the requested operation")]
    /// Insufficient initialization for the requested operation
    NotInit,

    #[error("libbladerf code {0}")]
    /// A libbladerf error code
    BladeRfCode(isize),

    /// An arbitrary string error
    #[error("{0}")]
    Msg(Box<str>),
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Msg(value.into_boxed_str())
    }
}

impl Error {
    /// Create a new arbitrary string error
    pub fn msg(msg: impl Into<String>) -> Self {
        Error::Msg(msg.into().into())
    }

    #[track_caller]
    /// Convert the error codes returned by `libbladerf` function calls into this error type.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___r_e_t_c_o_d_e_s.html>
    pub fn from_bladerf_code(code: isize) -> Self {
        // Values obtained from here:
        // https://github.com/Nuand/bladeRF/blob/fe3304d75967c88ab4f17ff37cb5daf8ff53d3e1/host/libraries/libbladeRF/include/libbladeRF.h#L4454-L4479
        match code {
            0.. => panic!("libbladerf returned positive error code: {code}"),
            -1 => Error::Unexpected,
            -2 => Error::Range,
            -3 => Error::Inval,
            -4 => Error::MEM,
            -5 => Error::IO,
            -6 => Error::Timeout,
            -7 => Error::Nodev,
            -8 => Error::Unsupported,
            -9 => Error::Misaligned,
            -10 => Error::CHECKSUM,
            -11 => Error::NoFile,
            -12 => Error::UpdateFpga,
            -13 => Error::UpdateFw,
            -14 => Error::TimePast,
            -15 => Error::QueueFull,
            -16 => Error::FpgaOp,
            -17 => Error::Permission,
            -18 => Error::WouldBlock,
            -19 => Error::NotInit,
            code => Error::BladeRfCode(code),
        }
    }
}

impl embedded_hal::digital::Error for Error {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        ErrorKind::Other
    }
}
