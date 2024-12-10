use strum::FromRepr;

use crate::{sys::*, Error, Result};

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
