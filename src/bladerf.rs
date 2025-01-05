use crate::{bladerf_drop, error::*, sys::*, types::*};
use enum_map::EnumMap;
use ffi::{c_char, c_void, CStr, CString};
use log::warn;
use marker::PhantomData;
use mem::ManuallyDrop;
use parking_lot::{lock_api::MutexGuard, Mutex};
use path::Path;
use private::Sealed;
use std::*;
use sync::RwLock;
use time::Duration;

// Macro to simplify integer returns
macro_rules! check_res {
    ($e:expr) => (
    	if $e < 0 {
			return Err($crate::Error::from_bladerf_code($e as isize))
		}
	);
}

pub const FPGA_BITSTREAM_VAR_NAME: &str = "BLADERF_RS_FPGA_BITSTREAM_PATH";

mod private {
    pub trait Sealed {}
}

pub trait HardwareVariant: Sealed {}

pub struct BladeRf1 {}
impl Sealed for BladeRf1 {}
impl HardwareVariant for BladeRf1 {}

pub struct BladeRf2 {}
impl Sealed for BladeRf2 {}
impl HardwareVariant for BladeRf2 {}

pub struct Unknown {}
impl Sealed for Unknown {}
impl HardwareVariant for Unknown {}

// unsafe impl<D: HardwareVariant> Send for BladeRF<D> {}
// unsafe impl<D: HardwareVariant> Sync for BladeRF<D> {}

// impl<D: HardwareVariant> Drop for BladeRF<D> {
//     fn drop(&mut self) {
//         let enabled_modules = *self.enabled_modules.get_mut();
//         for (channel, enabled) in enabled_modules {
//             if enabled {
//                 if let Err(e) = self.disable_module(channel) {
//                     warn!("Failed to disable module {channel:?} on Drop: {e:?}");
//                 }
//             }
//         }

//         unsafe { bladerf_close(self.device) }
//     }
// }

// trait BladeRf {
// fn open_first() -> Result<Self<Unknown>> {
//     log::info!("Opening first bladerf");
//     let mut device = std::ptr::null_mut();
//     let res = unsafe { bladerf_open(&mut device as *mut *mut _, ptr::null()) };
//     check_res!(res);
//     Ok(BladeRF::<Unknown> {
//         device,
//         enabled_modules: Mutex::new(EnumMap::default()),
//         format_sync: RwLock::new(None),
//         _p: PhantomData,
//     })
// }

/// Open a BladeRF device by identifier
//     pub fn open_identifier(id: &str) -> Result<BladeRF<Unknown>> {
//         let mut device = std::ptr::null_mut();
//         let c_string = ffi::CString::new(id)
//             .map_err(|e| Error::msg(format!("Invalid c string `{id}`: {e:?}")))?;
//         let res = unsafe { bladerf_open(&mut device as *mut *mut _, c_string.as_ptr()) };

//         check_res!(res);
//         Ok(BladeRF::<Unknown> {
//             device,
//             enabled_modules: Mutex::new(EnumMap::default()),
//             format_sync: RwLock::new(None),
//             _p: PhantomData,
//         })
//     }

//     /// Open a BladeRF device by devinfo object
//     pub fn open_with_devinfo(devinfo: &DevInfo) -> Result<BladeRF<Unknown>> {
//         let mut devinfo_ptr = devinfo.0;
//         let mut device = std::ptr::null_mut();

//         let res = unsafe {
//             bladerf_open_with_devinfo(&mut device as *mut *mut _, &mut devinfo_ptr as *mut _)
//         };

//         check_res!(res);
//         Ok(BladeRF::<Unknown> {
//             device,
//             enabled_modules: Mutex::new(EnumMap::default()),
//             format_sync: RwLock::new(None),
//             _p: PhantomData,
//         })
//     }
// }

unsafe impl Send for BladeRfAny {}
unsafe impl Sync for BladeRfAny {}

struct BladeRfAny {
    device: *mut bladerf,
    enabled_modules: Mutex<EnumMap<Channel, bool>>,
    format_sync: RwLock<Option<Format>>,
}

impl BladeRF for BladeRfAny {
    fn get_device_ptr(&self) -> *mut bladerf {
        self.device
    }

    fn get_enabled_modules(&self) -> MutexGuard<'_, parking_lot::RawMutex, EnumMap<Channel, bool>> {
        self.enabled_modules.lock()
    }

    // fn get_enabled_modules_mut(&mut self) -> &mut EnumMap<Channel, bool> {
    //     self.enabled_modules.get_mut()
    // }
}

impl Drop for BladeRfAny {
    fn drop(&mut self) {
        bladerf_drop(self);
    }
}

pub trait BladeRF: Sized + Drop {
    fn get_device_ptr(&self) -> *mut bladerf;

    fn info(&self) -> Result<DevInfo> {
        let mut info = bladerf_devinfo {
            backend: 0,
            serial: [0; 33],
            usb_bus: 0,
            usb_addr: 0,
            instance: 0,
            manufacturer: [0; 33],
            product: [0; 33],
        };
        let res = unsafe { bladerf_get_devinfo(self.get_device_ptr(), &mut info as *mut _) };
        check_res!(res);
        Ok(info.into())
    }

    // Device Properties and Information
    // http://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html

    fn get_serial(&self) -> Result<String> {
        let mut serial_data = [0i8; BLADERF_SERIAL_LENGTH as usize];

        let res =
            unsafe { bladerf_get_serial(self.get_device_ptr(), serial_data.as_mut_ptr().cast()) };

        check_res!(res);
        let serial_cstr = unsafe { CStr::from_ptr(serial_data.as_ptr().cast()) };
        let serial_str = serial_cstr
            .to_str()
            .map_err(|e| Error::msg(format!("Serial number is not UTF-8: {e:?}")))?;

        Ok(serial_str.to_string())
    }

    fn get_fpga_size(&self) -> Result<bladerf_fpga_size> {
        let mut fpga_size: bladerf_fpga_size = bladerf_fpga_size_BLADERF_FPGA_UNKNOWN;
        let res = unsafe { bladerf_get_fpga_size(self.get_device_ptr(), &mut fpga_size) };
        check_res!(res);
        Ok(fpga_size)
    }

    fn firmware_version(&self) -> Result<Version> {
        let mut version = bladerf_version {
            major: 0,
            minor: 0,
            patch: 0,
            describe: std::ptr::null(),
        };

        let res = unsafe { bladerf_fw_version(self.get_device_ptr(), &mut version) };
        check_res!(res);

        // SAFETY: came from bladerf ffi
        Ok(unsafe { Version::from_ffi(&version) })
    }

    fn is_fpga_configured(&self) -> Result<bool> {
        let res = unsafe { bladerf_is_fpga_configured(self.get_device_ptr()) };
        check_res!(res);

        match res {
            1 => Ok(true),
            0 => Ok(false),
            _ => panic!("bladerf_is_fpga_configured returned invalid value: {res}"),
        }
    }

    fn fpga_version(&self) -> Result<Version> {
        let mut version = bladerf_version {
            major: 0,
            minor: 0,
            patch: 0,
            describe: std::ptr::null(),
        };

        let res = unsafe { bladerf_fpga_version(self.get_device_ptr(), &mut version) };
        check_res!(res);

        // SAFETY: came from bladerf ffi
        Ok(unsafe { Version::from_ffi(&version) })
    }

    // RX & TX Module Control
    // http://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___m_o_d_u_l_e.html

    fn get_enabled_modules(&self) -> MutexGuard<'_, parking_lot::RawMutex, EnumMap<Channel, bool>>;

    // fn get_enabled_modules_mut(&mut self) -> &mut EnumMap<Channel, bool>;

    fn enable_module(&self, channel: Channel) -> Result<()> {
        self.set_module_enabled(channel, true)
    }

    fn disable_module(&self, channel: Channel) -> Result<()> {
        self.set_module_enabled(channel, false)
    }

    fn set_module_enabled(&self, channel: Channel, enable: bool) -> Result<()> {
        let mut enabled_modules = self.get_enabled_modules();

        let res = unsafe {
            bladerf_enable_module(self.get_device_ptr(), channel as bladerf_channel, enable)
        };
        check_res!(res);

        enabled_modules[channel] = enable;
        Ok(())
    }

    // Gain Control
    // http://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html

    // Sampling Control

    fn set_sample_rate(&self, channel: Channel, rate: u32) -> Result<u32> {
        let mut actual: u32 = 0;

        let res = unsafe {
            bladerf_set_sample_rate(
                self.get_device_ptr(),
                channel as bladerf_module,
                rate,
                &mut actual,
            )
        };
        check_res!(res);
        Ok(actual)
    }

    fn set_rational_sample_rate(
        &self,
        channel: Channel,
        rate: bladerf_rational_rate,
    ) -> Result<RationalRate> {
        let mut rate = rate;
        let mut actual = bladerf_rational_rate {
            integer: 0,
            num: 0,
            den: 0,
        };
        let res = unsafe {
            bladerf_set_rational_sample_rate(
                self.get_device_ptr(),
                channel as bladerf_module,
                &mut rate,
                &mut actual,
            )
        };
        check_res!(res);
        Ok(actual.into())
    }

    fn get_sample_rate(&self, channel: Channel) -> Result<u32> {
        let mut rate: u32 = 0;

        let res = unsafe {
            bladerf_get_sample_rate(self.get_device_ptr(), channel as bladerf_channel, &mut rate)
        };
        check_res!(res);
        Ok(rate)
    }

    fn get_rational_sample_rate(&self, channel: Channel) -> Result<RationalRate> {
        let mut rate = bladerf_rational_rate {
            integer: 0,
            num: 0,
            den: 0,
        };

        let res = unsafe {
            bladerf_get_rational_sample_rate(
                self.get_device_ptr(),
                channel as bladerf_module,
                &mut rate,
            )
        };
        check_res!(res);
        Ok(rate.into())
    }

    fn get_sample_rate_range(&self, channel: Channel) -> Result<Range> {
        let mut range_ptr: *const bladerf_range = ptr::null();
        let res = unsafe {
            bladerf_get_sample_rate_range(
                self.get_device_ptr(),
                channel as bladerf_channel,
                &mut range_ptr,
            )
        };
        check_res!(res);
        if range_ptr.is_null() {
            return Err(Error::msg(
                "bladerf_get_sample_rate_range returned null pointer",
            ));
        }
        let range = unsafe { &*range_ptr };
        Ok(Range::from(range))
    }

    fn set_rx_mux(&self, mux: RxMux) -> Result<()> {
        let res = unsafe { bladerf_set_rx_mux(self.get_device_ptr(), mux as bladerf_rx_mux) };
        check_res!(res);
        Ok(())
    }

    fn get_rx_mux(&self) -> Result<RxMux> {
        let mut mux = bladerf_rx_mux_BLADERF_RX_MUX_INVALID;
        let res = unsafe { bladerf_get_rx_mux(self.get_device_ptr(), &mut mux) };
        check_res!(res);
        RxMux::try_from(mux)
    }

    // Configure bandwidth

    fn set_bandwidth(&self, channel: Channel, bandwidth: u32) -> Result<u32> {
        let mut actual: u32 = 0;
        let res = unsafe {
            bladerf_set_bandwidth(
                self.get_device_ptr(),
                channel as bladerf_channel,
                bandwidth,
                &mut actual,
            )
        };
        check_res!(res);
        Ok(actual)
    }

    fn get_bandwidth(&self, ch: Channel) -> Result<u32> {
        let mut bandwidth: u32 = 0;
        let res = unsafe {
            bladerf_get_bandwidth(self.get_device_ptr(), ch as bladerf_channel, &mut bandwidth)
        };
        check_res!(res);
        Ok(bandwidth)
    }

    fn get_bandwidth_range(&self, channel: Channel) -> Result<Range> {
        let mut range_ptr: *const bladerf_range = ptr::null();
        let res = unsafe {
            bladerf_get_bandwidth_range(
                self.get_device_ptr(),
                channel as bladerf_channel,
                &mut range_ptr,
            )
        };
        check_res!(res);
        if range_ptr.is_null() {
            return Err(Error::msg(
                "bladerf_get_bandwidth_range returned null pointer",
            ));
        }
        let range = unsafe { &*range_ptr };
        Ok(Range::from(range))
    }

    /// Set frequency band
    fn select_band(&self, channel: Channel, frequency: u64) -> Result<()> {
        let res = unsafe {
            bladerf_select_band(self.get_device_ptr(), channel as bladerf_channel, frequency)
        };
        check_res!(res);
        Ok(())
    }

    fn set_frequency(&self, channel: Channel, frequency: u64) -> Result<()> {
        let res = unsafe {
            bladerf_set_frequency(self.get_device_ptr(), channel as bladerf_channel, frequency)
        };
        check_res!(res);
        Ok(())
    }

    fn get_frequency(&self, channel: Channel) -> Result<u64> {
        let mut freq: u64 = 0;
        let res = unsafe {
            bladerf_get_frequency(self.get_device_ptr(), channel as bladerf_channel, &mut freq)
        };
        check_res!(res);
        Ok(freq)
    }

    fn get_frequency_range(&self, channel: Channel) -> Result<Range> {
        let mut range_ptr: *const bladerf_range = ptr::null();
        let res = unsafe {
            bladerf_get_frequency_range(
                self.get_device_ptr(),
                channel as bladerf_channel,
                &mut range_ptr,
            )
        };
        check_res!(res);
        if range_ptr.is_null() {
            return Err(Error::msg(
                "bladerf_get_frequency_range returned null pointer",
            ));
        }
        let range = unsafe { &*range_ptr };
        Ok(Range::from(range))
    }

    fn schedule_retune(
        &self,
        channel: Channel,
        time: u64,
        frequency: u64,
        quick_tune: Option<&mut QuickTune>,
    ) -> Result<()> {
        let quick_tune_ptr = quick_tune
            .map(|qt| qt as *mut QuickTune as *mut bladerf_quick_tune)
            .unwrap_or(ptr::null_mut());
        let res = unsafe {
            bladerf_schedule_retune(
                self.get_device_ptr(),
                channel as bladerf_channel,
                time,
                frequency,
                quick_tune_ptr,
            )
        };
        check_res!(res);
        Ok(())
    }

    fn cancel_scheduled_retune(&self, channel: Channel) -> Result<()> {
        let res = unsafe {
            bladerf_cancel_scheduled_retunes(self.get_device_ptr(), channel as bladerf_channel)
        };
        check_res!(res);
        Ok(())
    }

    fn get_quick_tune(&self, channel: Channel) -> Result<QuickTune> {
        let mut quick_tune = QuickTune {
            freqsel: 0,
            vcocap: 0,
            nint: 0,
            nfrac: 0,
            flags: 0,
        };
        let res = unsafe {
            bladerf_get_quick_tune(
                self.get_device_ptr(),
                channel as bladerf_channel,
                &mut quick_tune as *mut QuickTune as *mut bladerf_quick_tune,
            )
        };
        check_res!(res);
        Ok(quick_tune)
    }

    fn set_tuning_mode(&self, mode: TuningMode) -> Result<()> {
        let res =
            unsafe { bladerf_set_tuning_mode(self.get_device_ptr(), mode as bladerf_tuning_mode) };
        check_res!(res);
        Ok(())
    }

    // **Loopback Functions**

    /// Get loopback modes
    fn get_loopback_modes(&self) -> Result<Vec<LoopbackModeInfo>> {
        let mut modes_ptr: *const bladerf_loopback_modes = ptr::null();
        let num_modes =
            unsafe { bladerf_get_loopback_modes(self.get_device_ptr(), &mut modes_ptr) };
        if num_modes < 0 {
            return Err(Error::from_bladerf_code(num_modes as isize));
        }
        if modes_ptr.is_null() || num_modes == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: modes_ptr points to an array of num_modes elements
        let modes_slice = unsafe { slice::from_raw_parts(modes_ptr, num_modes as usize) };
        let loopback_modes: Vec<LoopbackModeInfo> = modes_slice
            .iter()
            .map(|m| LoopbackModeInfo::from(*m))
            .collect();
        Ok(loopback_modes)
    }

    /// Test if a given loopback mode is supported on this device
    fn is_loopback_mode_supported(&self, mode: Loopback) -> Result<bool> {
        let supported = unsafe {
            bladerf_is_loopback_mode_supported(self.get_device_ptr(), mode as bladerf_loopback)
        };
        Ok(supported)
    }

    /// See: <http://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___l_o_o_p_b_a_c_k.html>
    fn set_loopback(&self, loopback: Loopback) -> Result<()> {
        let res =
            unsafe { bladerf_set_loopback(self.get_device_ptr(), loopback as bladerf_loopback) };
        check_res!(res);
        Ok(())
    }

    /// Fetch loopback state
    fn get_loopback(&self) -> Result<Loopback> {
        unsafe {
            let mut loopback = bladerf_loopback_BLADERF_LB_NONE;

            let res = bladerf_get_loopback(self.get_device_ptr(), &mut loopback);
            check_res!(res);

            Loopback::try_from(loopback)
        }
    }

    // **Gain Control Functions**

    /// Set overall system gain
    fn set_gain(&self, channel: Channel, gain: Gain) -> Result<()> {
        let res =
            unsafe { bladerf_set_gain(self.get_device_ptr(), channel as bladerf_channel, gain) };
        check_res!(res);
        Ok(())
    }

    /// Get overall system gain
    fn get_gain(&self, channel: Channel) -> Result<Gain> {
        let mut gain: Gain = 0;
        let res = unsafe {
            bladerf_get_gain(self.get_device_ptr(), channel as bladerf_channel, &mut gain)
        };
        check_res!(res);
        Ok(gain)
    }

    /// Set gain control mode
    fn set_gain_mode(&self, channel: Channel, mode: GainMode) -> Result<()> {
        let res = unsafe {
            bladerf_set_gain_mode(
                self.get_device_ptr(),
                channel as bladerf_channel,
                mode as bladerf_gain_mode,
            )
        };
        check_res!(res);
        Ok(())
    }

    /// Get gain control mode
    fn get_gain_mode(&self, channel: Channel) -> Result<GainMode> {
        let mut mode = bladerf_gain_mode_BLADERF_GAIN_DEFAULT;
        let res = unsafe {
            bladerf_get_gain_mode(self.get_device_ptr(), channel as bladerf_channel, &mut mode)
        };
        check_res!(res);
        GainMode::try_from(mode)
    }

    /// Get available gain control modes
    fn get_gain_modes(&self, channel: Channel) -> Result<Vec<GainModeInfo>> {
        let mut modes_ptr: *const bladerf_gain_modes = ptr::null();
        let num_modes = unsafe {
            bladerf_get_gain_modes(
                self.get_device_ptr(),
                channel as bladerf_channel,
                &mut modes_ptr,
            )
        };
        if num_modes < 0 {
            return Err(Error::from_bladerf_code(num_modes as isize));
        }
        if modes_ptr.is_null() || num_modes == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: modes_ptr points to an array of num_modes elements
        let modes_slice = unsafe { slice::from_raw_parts(modes_ptr, num_modes as usize) };
        let gain_modes: Vec<GainModeInfo> =
            modes_slice.iter().map(|m| GainModeInfo::from(*m)).collect();
        Ok(gain_modes)
    }

    /// Get range of overall system gain
    fn get_gain_range(&self, channel: Channel) -> Result<Range> {
        let mut range_ptr: *const bladerf_range = ptr::null();
        let res = unsafe {
            bladerf_get_gain_range(
                self.get_device_ptr(),
                channel as bladerf_channel,
                &mut range_ptr,
            )
        };
        check_res!(res);
        if range_ptr.is_null() {
            return Err(Error::msg("bladerf_get_gain_range returned null pointer"));
        }
        let range = unsafe { &*range_ptr };
        Ok(Range::from(range))
    }

    /// Set the gain for a specific gain stage
    fn set_gain_stage(&self, channel: Channel, stage: &str, gain: Gain) -> Result<()> {
        let stage_cstr = CString::new(stage).map_err(|_| Error::msg("Invalid stage string"))?;
        let res = unsafe {
            bladerf_set_gain_stage(
                self.get_device_ptr(),
                channel as bladerf_channel,
                stage_cstr.as_ptr(),
                gain,
            )
        };
        check_res!(res);
        Ok(())
    }

    /// Get the gain for a specific gain stage
    fn get_gain_stage(&self, channel: Channel, stage: &str) -> Result<Gain> {
        let stage_cstr = CString::new(stage).map_err(|_| Error::msg("Invalid stage string"))?;
        let mut gain: Gain = 0;
        let res = unsafe {
            bladerf_get_gain_stage(
                self.get_device_ptr(),
                channel as bladerf_channel,
                stage_cstr.as_ptr(),
                &mut gain as *mut bladerf_gain,
            )
        };
        check_res!(res);
        Ok(gain)
    }

    /// Get gain range of a specific gain stage
    fn get_gain_stage_range(&self, channel: Channel, stage: &str) -> Result<Range> {
        let stage_cstr = CString::new(stage).map_err(|_| Error::msg("Invalid stage string"))?;
        let mut range_ptr: *const bladerf_range = ptr::null();
        let res = unsafe {
            bladerf_get_gain_stage_range(
                self.get_device_ptr(),
                channel as bladerf_channel,
                stage_cstr.as_ptr(),
                &mut range_ptr,
            )
        };
        check_res!(res);
        assert!(!range_ptr.is_null());

        // SAFETY: non-null, set by libusb
        Ok(Range::from(unsafe { &*range_ptr }))
    }

    /// Get a list of available gain stages
    fn get_gain_stages(&self, channel: Channel) -> Result<Vec<String>> {
        // First, call with count = 0 to get the number of stages
        let num_stages = unsafe {
            bladerf_get_gain_stages(
                self.get_device_ptr(),
                channel as bladerf_channel,
                ptr::null_mut(),
                0,
            )
        };
        check_res!(num_stages);
        let num_stages = num_stages as usize;
        if num_stages == 0 {
            return Ok(Vec::new());
        }

        // Allocate an array to hold the pointers
        let mut stages: Vec<*const c_char> = vec![ptr::null(); num_stages];
        let res = unsafe {
            bladerf_get_gain_stages(
                self.get_device_ptr(),
                channel as bladerf_channel,
                stages.as_mut_ptr(),
                num_stages,
            )
        };
        check_res!(res);

        // Now, convert the pointers to Rust strings
        let stages: Vec<_> = stages
            .into_iter()
            .flat_map(|ptr| {
                if ptr.is_null() {
                    None
                } else {
                    unsafe { CStr::from_ptr(ptr).to_str() }
                        .ok()
                        .map(ToString::to_string)
                }
            })
            .collect();

        Ok(stages)
    }

    // **Trigger Functions**

    /// Initialize a trigger
    fn trigger_init(&self, channel: Channel, signal: TriggerSignal) -> Result<Trigger> {
        let mut trigger = bladerf_trigger {
            channel: 0,
            role: 0,
            signal: 0,
            options: 0,
        };
        let res = unsafe {
            bladerf_trigger_init(
                self.get_device_ptr(),
                channel as bladerf_channel,
                signal as bladerf_trigger_signal,
                &mut trigger as *mut bladerf_trigger,
            )
        };
        check_res!(res);
        trigger.try_into()
    }

    /// Configure and (dis)arm a trigger on the specified device
    fn trigger_arm(&self, trigger: &Trigger, arm: bool) -> Result<()> {
        let res = unsafe {
            bladerf_trigger_arm(
                self.get_device_ptr(),
                trigger as *const Trigger as *const bladerf_trigger,
                arm,
                0,
                0,
            )
        };
        check_res!(res);
        Ok(())
    }

    /// Fire a trigger event
    fn trigger_fire(&self, trigger: &Trigger) -> Result<()> {
        let res = unsafe {
            bladerf_trigger_fire(
                self.get_device_ptr(),
                trigger as *const Trigger as *const bladerf_trigger,
            )
        };
        check_res!(res);
        Ok(())
    }

    /// Query the fire request status of a master trigger
    fn trigger_state(&self, trigger: &Trigger) -> Result<(bool, bool, bool)> {
        let mut is_armed = false;
        let mut has_fired = false;
        let mut fire_requested = false;
        let mut resv1 = 0u64;
        let mut resv2 = 0u64;
        let res = unsafe {
            bladerf_trigger_state(
                self.get_device_ptr(),
                trigger as *const Trigger as *const bladerf_trigger,
                &mut is_armed,
                &mut has_fired,
                &mut fire_requested,
                &mut resv1,
                &mut resv2,
            )
        };
        check_res!(res);
        Ok((is_armed, has_fired, fire_requested))
    }

    // Triggers and Synchronisation

    // **Correction Functions**

    /// Set the value of the specified correction parameter
    fn set_correction(
        &self,
        channel: Channel,
        corr: Correction,
        value: CorrectionValue,
    ) -> Result<()> {
        let res = unsafe {
            bladerf_set_correction(
                self.get_device_ptr(),
                channel as bladerf_channel,
                corr as bladerf_correction,
                value,
            )
        };
        check_res!(res);
        Ok(())
    }

    /// Obtain the current value of the specified correction parameter
    fn get_correction(&self, channel: Channel, corr: Correction) -> Result<CorrectionValue> {
        let mut value: CorrectionValue = 0;
        let res = unsafe {
            bladerf_get_correction(
                self.get_device_ptr(),
                channel as bladerf_channel,
                corr as bladerf_correction,
                &mut value,
            )
        };
        check_res!(res);
        Ok(value)
    }

    // Corrections and Calibration

    // Corrections and calibration

    // Expansion boards

    // Expansion IO control

    // Miscellaneous

    // Sample formats and metadata
    fn abc() {}

    // Asynchronous data transmission and reception

    // Synchronous data transmission and reception

    /// Configure the device for synchronous data transfer
    // fn sync_config(
    //     &self,
    //     channel: ChannelLayout,
    //     format: Format,
    //     num_buffers: u32,
    //     buffer_size: u32,
    //     num_transfers: u32,
    //     stream_timeout: Duration,
    // ) -> Result<()> {
    //     let stream_timeout_ms = stream_timeout.as_millis() as u32;
    //     let res = unsafe {
    //         bladerf_sync_config(
    //             self.get_device_ptr(),
    //             // Bindgen not precise with #define types
    //             channel as bladerf_channel_layout,
    //             format as bladerf_format,
    //             num_buffers,
    //             buffer_size,
    //             num_transfers,
    //             stream_timeout_ms,
    //         )
    //     };
    //     check_res!(res);

    //     // Store the configured format
    //     let mut fmt = self.format_sync.write().unwrap();
    //     *fmt = Some(format);

    //     Ok(())
    // }

    /// Transmit IQ samples synchronously
    // fn sync_tx<T>(
    //     &self,
    //     data: &[T],
    //     metadata: Option<&mut Metadata>,
    //     timeout: Duration,
    // ) -> Result<()>
    // where
    //     T: SampleFormat,
    // {
    //     let format_guard = self.format_sync.read().unwrap();
    //     let format = format_guard.ok_or_else(|| Error::msg("Format not configured"))?;

    //     T::check_compatability(format)?;

    //     let timeout_ms = timeout.as_millis() as u32;
    //     let mut bladerf_meta = bladerf_metadata {
    //         timestamp: 0,
    //         flags: 0,
    //         status: 0,
    //         actual_count: 0,
    //         reserved: [0u8; 32],
    //     };
    //     let meta_ptr = if let Some(meta) = &metadata {
    //         bladerf_meta.timestamp = meta.timestamp;
    //         bladerf_meta.flags = meta.flags;
    //         &mut bladerf_meta as *mut bladerf_metadata
    //     } else {
    //         std::ptr::null_mut()
    //     };

    //     let res = unsafe {
    //         bladerf_sync_tx(
    //             self.get_device_ptr(),
    //             data.as_ptr() as *const c_void,
    //             data.len() as u32,
    //             meta_ptr,
    //             timeout_ms,
    //         )
    //     };

    //     if !meta_ptr.is_null() {
    //         if let Some(meta) = metadata {
    //             *meta = Metadata::from(&bladerf_meta);
    //         }
    //     }

    //     check_res!(res);
    //     Ok(())
    // }

    /// Receive IQ samples synchronously
    // fn sync_rx<T>(
    //     &self,
    //     data: &mut [T],
    //     metadata: Option<&mut Metadata>,
    //     timeout: Duration,
    // ) -> Result<()>
    // where
    //     T: SampleFormat,
    // {
    //     let format_guard = self.format_sync.read().unwrap();
    //     let format = format_guard.ok_or_else(|| Error::msg("Format not configured"))?;

    //     T::check_compatability(format)?;

    //     let timeout_ms = timeout.as_millis() as u32;
    //     let mut bladerf_meta = bladerf_metadata {
    //         timestamp: 0,
    //         flags: 0,
    //         status: 0,
    //         actual_count: 0,
    //         reserved: [0u8; 32],
    //     };
    //     let meta_ptr = if let Some(meta) = &metadata {
    //         bladerf_meta.timestamp = meta.timestamp;
    //         bladerf_meta.flags = meta.flags;
    //         &mut bladerf_meta as *mut bladerf_metadata
    //     } else {
    //         std::ptr::null_mut()
    //     };

    //     let res = unsafe {
    //         bladerf_sync_rx(
    //             self.get_device_ptr(),
    //             data.as_mut_ptr() as *mut c_void,
    //             data.len() as u32,
    //             meta_ptr,
    //             timeout_ms,
    //         )
    //     };

    //     if !meta_ptr.is_null() {
    //         if let Some(meta) = metadata {
    //             *meta = Metadata::from(&bladerf_meta);
    //         }
    //     }

    //     check_res!(res);
    //     Ok(())
    // }

    /// Retrieve the current timestamp
    fn get_timestamp(&self, dir: Direction) -> Result<u64> {
        let mut timestamp: u64 = 0;
        let res =
            unsafe { bladerf_get_timestamp(self.get_device_ptr(), dir.into(), &mut timestamp) };
        check_res!(res);
        Ok(timestamp)
    }

    // Device loading and programming

    /// Write FX3 firmware to the bladeRF’s SPI flash
    /// NOTE: This will require a power cycle to take effect
    fn flash_firmware(&self, firmware_path: impl AsRef<Path>) -> Result<()> {
        let bitstream_path = CString::new(firmware_path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(|e| Error::msg(format!("Invalid path for cstring: {e:?}")))?;

        let res = unsafe { bladerf_flash_firmware(self.get_device_ptr(), bitstream_path.as_ptr()) };
        check_res!(res);
        Ok(())
    }

    /// Reset the device, causing it to reload its firmware from flash
    fn device_reset(self) -> Result<()> {
        let res = unsafe { bladerf_device_reset(self.get_device_ptr()) };
        check_res!(res);
        Ok(())
    }

    /// Uploads the fpga bitstream file from the path in env var [`FPGA_BITSTREAM_VAR_NAME`].
    fn load_fpga_from_env(&self) -> Result<()> {
        let path = std::env::var(FPGA_BITSTREAM_VAR_NAME).map_err(|e| {
            Error::msg(format!(
                "Failed to read env var {FPGA_BITSTREAM_VAR_NAME}: {e:?}"
            ))
        })?;

        self.load_fpga_path(Path::new(&path))
    }

    fn load_fpga_path(&self, bitstream_path: impl AsRef<Path>) -> Result<()> {
        let bitstream_path = CString::new(bitstream_path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(|e| Error::msg(format!("Invalid path for cstring: {e:?}")))?;

        let res = unsafe { bladerf_load_fpga(self.get_device_ptr(), bitstream_path.as_ptr()) };
        check_res!(res);
        Ok(())
    }

    fn flash_fpga(&self, bitstream_path: impl AsRef<Path>) -> Result<()> {
        let bitstream_path = CString::new(bitstream_path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(|e| Error::msg(format!("Invalid path for cstring: {e:?}")))?;

        let res = unsafe { bladerf_flash_fpga(self.get_device_ptr(), bitstream_path.as_ptr()) };
        check_res!(res);
        Ok(())
    }

    fn erase_stored_fpga(&self) -> Result<()> {
        let res = unsafe { bladerf_erase_stored_fpga(self.get_device_ptr()) };
        check_res!(res);
        Ok(())
    }

    fn get_fw_log(&self, path: impl AsRef<Path>) -> Result<()> {
        let log_path = CString::new(path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(|e| Error::msg(format!("Invalid path for cstring: {e:?}")))?;
        let res = unsafe { bladerf_get_fw_log(self.get_device_ptr(), log_path.as_ptr()) };
        check_res!(res);
        Ok(())
    }

    // Higher level control of one RF module
    fn configure_module(&self, channel: Channel, config: ModuleConfig) -> Result<()> {
        self.set_frequency(channel, config.frequency)?;
        self.set_sample_rate(channel, config.sample_rate)?;
        self.set_bandwidth(channel, config.bandwidth)?;
        self.set_gain(channel, config.gain)?;

        Ok(())
    }

    fn get_board_name(&self) -> &'static str {
        // Safety, the function returns a string that is compiled in (static I guess? is there another term I should use?)
        let name_raw = unsafe { CStr::from_ptr(bladerf_get_board_name(self.get_device_ptr())) };
        name_raw.to_str().unwrap()
    }

    // fn change_marker_traits<DN: HardwareVariant>(self) -> BladeRF<DN> {
    //     let dev_to_move = ManuallyDrop::new(self);

    //     // Use `std::ptr::read` to move non-Copy fields out of the ManuallyDrop wrapper
    //     // SAFETY:
    //     // Being a rust reference, the following hold.
    //     // 1. each field is valid for reads
    //     // 2. each field is guaranteed to be aligned
    //     // 3. each field is properly initialized
    //     // Further
    //     // 4. Each field is read exactly once and then not dropped, therefore no double objects are created
    //     let device = unsafe { std::ptr::read(&dev_to_move.get_device_ptr()) };
    //     let enabled_modules = unsafe { std::ptr::read(&dev_to_move.enabled_modules) };
    //     let format_sync = unsafe { std::ptr::read(&dev_to_move.format_sync) };

    //     BladeRF::<DN> {
    //         device,
    //         enabled_modules,
    //         format_sync,
    //         _p: PhantomData,
    //     }
    // }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // // Prevent tests running in parallel from messing stuff up
    // // Also use parking_lot since we dont care about poisoning since tests are independent
    // static DEV_MUTEX: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

    // #[test]
    // fn test_list_devices() {
    //     let _m = DEV_MUTEX.lock();

    //     let devices = crate::get_device_list().expect("");
    //     println!("Discovered devices: {:?}", devices.len());
    // }

    // #[test]
    // fn test_open() {
    //     let _m = DEV_MUTEX.lock();

    //     let _device = BladeRF::open_first().unwrap();
    // }

    // #[test]
    // fn test_open_devinfo() {
    //     let _m = DEV_MUTEX.lock();

    //     let devices = crate::get_device_list().unwrap();
    //     assert!(!devices.is_empty());
    //     let _device = BladeRF::open_with_devinfo(&devices[0]).unwrap();
    // }

    // #[test]
    // fn test_get_fw_version() {
    //     let _m = DEV_MUTEX.lock();

    //     let device = BladeRF::open_first().unwrap();

    //     let version = device.firmware_version().unwrap();
    //     println!("FW Version {:?}", version);
    // }

    // #[test]
    // fn test_get_fpga_version() {
    //     let _m = DEV_MUTEX.lock();

    //     let device = BladeRF::open_first().unwrap();

    //     let version = device.fpga_version().unwrap();
    //     println!("FPGA Version {:?}", version);
    // }

    // #[test]
    // fn test_get_serial() {
    //     let _m = DEV_MUTEX.lock();

    //     let device = BladeRF::open_first().unwrap();

    //     let serial = device.get_serial().unwrap();
    //     println!("Serial: {:?}", serial);
    //     assert!(serial.len() == 32);
    // }

    // #[test]
    // fn test_fpga_loaded() {
    //     let _m = DEV_MUTEX.lock();

    //     let device = BladeRF::open_first().unwrap();

    //     let loaded = device.is_fpga_configured().unwrap();
    //     assert!(loaded);
    // }

    // #[test]
    // fn test_loopback_modes() {
    //     let _m = DEV_MUTEX.lock();

    //     let device = BladeRF::open_first().unwrap();

    //     // Check initial is none
    //     let loopback = device.get_loopback().unwrap();
    //     assert!(loopback == Loopback::None);

    //     // Set and check loopback modes
    //     device.set_loopback(Loopback::Firmware).unwrap();
    //     let loopback = device.get_loopback().unwrap();
    //     assert!(loopback == Loopback::Firmware);

    //     // Reset
    //     device.set_loopback(Loopback::None).unwrap();

    //     let loopback = device.get_loopback().unwrap();
    //     assert!(loopback == Loopback::None);
    // }

    // #[test]
    // fn test_set_freq() {
    //     let _m = DEV_MUTEX.lock();

    //     let device = BladeRF::open_first().unwrap();

    //     let freq: u64 = 915000000;

    //     // Set and check frequency
    //     device.set_frequency(Channel::Rx0, freq).unwrap();
    //     let actual_freq = device.get_frequency(Channel::Rx0).unwrap();
    //     let diff = freq as i64 - actual_freq as i64;
    //     assert!(i64::abs(diff) < 10);
    // }

    // #[test]
    // #[ignore = "bladerf1 specific test"]
    // fn test_bladerf1_set_sampling() -> Result<()> {
    //     let _m = DEV_MUTEX.lock();

    //     let device: BladeRF<BladeRf1> = BladeRF::open_first()?.try_into()?;

    //     let desired = Sampling::Internal;

    //     device.set_sampling(desired)?;

    //     let actual = device.get_sampling().unwrap();

    //     assert_eq!(desired, actual);
    //     Ok(())
    // }

    // #[test]
    // #[ignore = "bladerf1 specific test"]
    // fn test_bladerf1_ex() {
    //     let dev = BladeRF::open_first().unwrap();
    //     let newdev: BladeRF<BladeRf1> = dev.try_into().unwrap();
    //     newdev.set_txvga2(-20).unwrap();
    //     let _fwv = newdev.firmware_version().unwrap();
    // }
}
