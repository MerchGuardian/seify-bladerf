use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Represents what USB backend is used to interface with the BladeRF
///
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_i_t.html#ga3737a52a065ebc838adf4cf426b43fb2>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Backend {
    /// Use any available backend
    Any = bladerf_backend_BLADERF_BACKEND_ANY as i32,
    /// Use the Linux kernel driver
    Linux = bladerf_backend_BLADERF_BACKEND_LINUX as i32,
    /// User the [`libusb`](https://libusb.info/)
    LibUsb = bladerf_backend_BLADERF_BACKEND_LIBUSB as i32,
    /// Use the Cypress backend (for Windows only)
    Cypress = bladerf_backend_BLADERF_BACKEND_CYPRESS as i32,
    /// Dummy backend used for development.
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
