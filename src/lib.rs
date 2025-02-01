//! Safe bindings for libbladerf (wrapping bladerf-sys)
//!
//!
#![allow(non_upper_case_globals)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_endian = "little"))]
compile_error!("This library only supports little endian architecture");

mod error;

pub use error::{Error, Result};
mod types;
pub use types::*;
#[macro_use]
mod bladerf;
pub use bladerf::*;
mod bladerf1;
pub use bladerf1::*;
mod bladerf2;
pub use bladerf2::*;
mod stream;
pub use stream::*;

pub use libbladerf_sys as sys;
use sys::*;

/// Returns the version of the linked `libbladerf` library.
pub fn version() -> Result<Version> {
    let mut version = bladerf_version {
        major: 0,
        minor: 0,
        patch: 0,
        describe: std::ptr::null(),
    };
    unsafe { bladerf_version(&mut version as *mut _) };

    // SAFETY: came from bladerf ffi
    Ok(unsafe { Version::from_ffi(&version) })
}

pub fn set_log_level(level: LogLevel) {
    unsafe { bladerf_log_set_verbosity(level as u32) }
}

pub fn set_usb_reset_on_open(enabled: bool) {
    unsafe { bladerf_set_usb_reset_on_open(enabled) };
}

/// List attached BladeRF devices
pub fn get_device_list() -> Result<Vec<DevInfo>> {
    let mut devices: *mut bladerf_devinfo = std::ptr::null_mut();

    let n = unsafe { bladerf_get_device_list(&mut devices as *mut *mut _) } as isize;
    check_res!(n);

    assert!(!devices.is_null());
    // SAFETY: bladerf wrote to devices
    let ffi_devs = unsafe { std::slice::from_raw_parts(devices, n as usize) };
    let devs: Vec<DevInfo> = ffi_devs.iter().map(Clone::clone).map(Into::into).collect();

    unsafe { bladerf_free_device_list(devices) };

    Ok(devs)
}
