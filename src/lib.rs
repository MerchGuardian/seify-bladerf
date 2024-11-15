//! Safe bindings for libbladerf (wrapping bladerf-sys)
//!
//!
#![allow(non_upper_case_globals)]
#![deny(unsafe_op_in_unsafe_fn)]

mod error;
use std::ffi::c_char;

pub use error::{Error, Result};
mod types;
use parking_lot::RwLock;
pub use types::*;
#[macro_use]
mod bladerf;
pub use bladerf::*;

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

/// Sets the zero-overhead log callback using the C native types.
pub unsafe fn set_log_callback_fn(
    cb: Option<unsafe extern "C" fn(bladerf_log_level, *const c_char, usize)>,
) {
    unsafe { bladerf_set_log_callback(cb) }
}

static CB_DATA: RwLock<Option<Box<dyn Fn(LogLevel, &str) + Send + Sync + 'static>>> =
    RwLock::new(None);

unsafe extern "C" fn log_cb_trampoline(level: bladerf_log_level, msg: *const c_char, len: usize) {
    let Ok(level) = LogLevel::try_from(level)
        .map_err(|e| log::warn!("Log callback received invalid bladerf log level {level}: {e:?}"))
    else {
        return;
    };

    let bytes = unsafe { std::slice::from_raw_parts(msg.cast(), len) };
    match std::str::from_utf8(bytes) {
        Ok(s) => {
            // Bladerf always includes newlines. Our log callback expects trimmed lines
            let s = s.trim();
            if let Some(cb) = CB_DATA.read().as_ref() {
                cb(level, s);
            }
        }
        Err(e) => {
            log::warn!(
                "Log callback received non UTF-8 log message from bladerf {e:?}: `{bytes:?}`"
            );
        }
    }
}

/// Sets the log callback to a rust native `Fn`.
///
/// This method incurs additional overhead compared to [`set_log_callback_fn`].
pub fn set_log_callback(cb: Option<impl Fn(LogLevel, &str) + Send + Sync + 'static>) {
    let fn_ptr = cb.as_ref().map(|_| {
        log_cb_trampoline as unsafe extern "C" fn(bladerf_log_level, *const c_char, usize)
    });

    {
        let mut guard = CB_DATA.write();
        *guard = cb.map(|cb| {
            let b: Box<dyn Fn(LogLevel, &str) + Send + Sync + 'static> = Box::new(cb);
            b
        });
    }

    // NOTE: This will invoke the log callback to print the new level (guard in new scope above)
    unsafe {
        set_log_callback_fn(fn_ptr);
    }
}

/// Sets the log callback to send events to the [`log`] crate.
///
/// To reset call [`set_log_callback`] or [`set_log_callback_fn`] with `None`.
fn log_crate_callback(level: LogLevel, msg: &str) {
    match level {
        LogLevel::Verbose => log::trace!("{msg}"),
        LogLevel::Debug => log::debug!("{msg}"),
        LogLevel::Info => log::info!("{msg}"),
        LogLevel::Warning => log::warn!("{msg}"),
        LogLevel::Error => log::error!("{msg}"),
        LogLevel::Critical => log::error!("CRITICAL: {msg}"),
        LogLevel::Silent => {}
    };
}

pub fn set_log_callback_log_crate() {
    set_log_callback(Some(log_crate_callback));
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
