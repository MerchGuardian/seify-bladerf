//! Safe bindings for libbladerf (wrapping bladerf-sys)

#![allow(non_upper_case_globals)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

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
mod streamers;
pub use streamers::*;

pub mod expansion_boards;

pub use libbladerf_sys as sys;
use sys::*;

/// Returns the version of the linked `libbladerf` library.
///
/// Relavent libbladerf docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___l_i_b_r_a_r_y___v_e_r_s_i_o_n.html#ga1b726a123c60e6d7f999ad8958f20134>
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

/// Sets the logging level of `libbladerf`
///
/// Messages at and above the specified [LogLevel] will be printed.
///
/// Relavent `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___l_o_g_g_i_n_g.html#gae2de133be7904c2c11224f0b08bc0b36>
pub fn set_log_level(level: LogLevel) {
    unsafe { bladerf_log_set_verbosity(level as bladerf_log_level) }
}

/// Configures if the USB device will reset after a call to `open()` without reseting the configured parameters.
///
/// Relavent `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_i_t.html#ga0c69671b4b2a8fb45848b22a76efd83d>
pub fn set_usb_reset_on_open(enabled: bool) {
    unsafe { bladerf_set_usb_reset_on_open(enabled) };
}

/// List attached BladeRF devices
pub fn get_device_list() -> Result<Vec<DevInfo>> {
    let mut devices: *mut bladerf_devinfo = std::ptr::null_mut();

    let n = unsafe { bladerf_get_device_list(&mut devices) } as isize;
    check_res!(n);

    assert!(!devices.is_null());
    // SAFETY: bladerf wrote to devices
    let ffi_devs = unsafe { std::slice::from_raw_parts(devices, n as usize) };
    let devs: Vec<DevInfo> = ffi_devs.iter().map(Clone::clone).map(Into::into).collect();

    unsafe { bladerf_free_device_list(devices) };

    Ok(devs)
}
