use crate::{error::*, sys::*, types::*, RxSyncStream, SyncConfig, TxSyncStream};
use ffi::{c_char, CStr, CString};
use marker::PhantomData;
use path::Path;
use std::{mem::ManuallyDrop, *};
use sync::atomic::{AtomicBool, Ordering};

// Macro to simplify integer returns
macro_rules! check_res {
    ($e:expr) => (
    	if $e < 0 {
			return Err($crate::Error::from_bladerf_code($e as isize))
		}
	);
}

pub const FPGA_BITSTREAM_VAR_NAME: &str = "BLADERF_RS_FPGA_BITSTREAM_PATH";

unsafe impl Send for BladeRfAny {}
unsafe impl Sync for BladeRfAny {}

pub struct BladeRfAny {
    pub(crate) device: *mut bladerf,
    pub(crate) rx_stream_configured: AtomicBool,
    pub(crate) tx_stream_configured: AtomicBool,
}

impl BladeRfAny {
    pub fn open_first() -> Result<Self> {
        log::info!("Opening first bladerf");
        let mut device = std::ptr::null_mut();
        let res = unsafe { bladerf_open(&mut device, ptr::null()) };
        check_res!(res);
        Ok(BladeRfAny {
            device,
            rx_stream_configured: AtomicBool::new(false),
            tx_stream_configured: AtomicBool::new(false),
        })
    }

    pub fn open_identifier(id: &str) -> Result<Self> {
        let mut device = std::ptr::null_mut();
        let c_string =
            CString::new(id).map_err(|e| Error::msg(format!("Invalid c string `{id}`: {e:?}")))?;
        let res = unsafe { bladerf_open(&mut device, c_string.as_ptr()) };

        check_res!(res);
        Ok(BladeRfAny {
            device,
            rx_stream_configured: AtomicBool::new(false),
            tx_stream_configured: AtomicBool::new(false),
        })
    }

    pub fn open_with_devinfo(devinfo: &DevInfo) -> Result<Self> {
        let mut devinfo_ptr = devinfo.0;
        let mut device = std::ptr::null_mut();

        let res = unsafe { bladerf_open_with_devinfo(&mut device, &mut devinfo_ptr as *mut _) };

        check_res!(res);
        Ok(BladeRfAny {
            device,
            rx_stream_configured: AtomicBool::new(false),
            tx_stream_configured: AtomicBool::new(false),
        })
    }

    pub fn tx_streamer<T: SampleFormat>(
        &self,
        config: &SyncConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<T, Self>> {
        // TODO: Decide Ordering
        self.tx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an TX stream open".to_owned().into_boxed_str())
            })?;

        unsafe {
            self.set_sync_config::<T>(config, layout.into())?;
        }

        Ok(TxSyncStream {
            dev: self,
            layout,
            _format: PhantomData,
        })
    }

    pub fn rx_streamer<T: SampleFormat>(
        &self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&Self, T, BladeRfAny>> {
        // TODO: Decide Ordering
        self.rx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an RX stream open".to_owned().into_boxed_str())
            })?;

        unsafe {
            self.set_sync_config::<T>(config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev: self,
            layout,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

impl BladeRF for BladeRfAny {
    fn get_device_ptr(&self) -> *mut bladerf {
        self.device
    }
}

impl Drop for BladeRfAny {
    fn drop(&mut self) {
        unsafe { self.close() };
    }
}

// Allow drop bounds as a way to make sure we implement the drop trait for our BladeRf device structs
#[allow(drop_bounds)]
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

    fn get_device_speed(&self) -> Result<DeviceSpeed> {
        let speed = unsafe { bladerf_device_speed(self.get_device_ptr()) };
        speed.try_into()
    }

    fn get_fpga_size(&self) -> Result<FpgaSize> {
        let mut fpga_size: bladerf_fpga_size = bladerf_fpga_size_BLADERF_FPGA_UNKNOWN;
        let res = unsafe { bladerf_get_fpga_size(self.get_device_ptr(), &mut fpga_size) };
        check_res!(res);
        fpga_size.try_into()
    }

    fn get_firmware_version(&self) -> Result<Version> {
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

    fn get_fpga_version(&self) -> Result<Version> {
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

    /// Should only be called internally by the streamers
    ///
    /// This function is used by the streamer structs to enable or disable the module.
    /// If this function is called elsewhere, it may cause issues with the used streamer.
    #[doc(hidden)]
    fn set_enable_module(&self, channel: Channel, enable: bool) -> Result<()> {
        let res = unsafe {
            bladerf_enable_module(self.get_device_ptr(), channel as bladerf_channel, enable)
        };
        check_res!(res);
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

    /// # Safety
    /// Intended for internal use only.
    /// This is used to reconfigure a synchronous stream and is called by the streamer structs.
    ///
    /// If this function is called elsewhere, it may cause issues with the used streamer since the type can arbitraily be changed.
    #[doc(hidden)]
    unsafe fn set_sync_config<T: SampleFormat>(
        &self,
        config: &SyncConfig,
        layout: ChannelLayout,
    ) -> Result<()> {
        let res = unsafe {
            bladerf_sync_config(
                self.get_device_ptr(),
                layout as bladerf_channel_layout,
                T::FORMAT as bladerf_format,
                config.num_buffers,
                config.buffer_size,
                config.num_transfers,
                config.stream_timeout,
            )
        };
        check_res!(res);
        Ok(())
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
    fn set_correction<T: CorrectionValue>(&self, channel: Channel, corr: T) -> Result<()> {
        let correction_type: Correction = T::TYPE;
        let res = unsafe {
            bladerf_set_correction(
                self.get_device_ptr(),
                channel as bladerf_channel,
                correction_type as bladerf_correction,
                corr.value(),
            )
        };
        check_res!(res);
        Ok(())
    }

    /// Obtain the current value of the specified correction parameter
    fn get_correction<T: CorrectionValue>(&self, channel: Channel) -> Result<T> {
        let corr = T::TYPE;
        let mut value: i16 = 0;
        let res = unsafe {
            bladerf_get_correction(
                self.get_device_ptr(),
                channel as bladerf_channel,
                corr as bladerf_correction,
                &mut value,
            )
        };
        check_res!(res);
        T::new(value).ok_or(Error::Msg(
            format!("Invalid correction value returned from bladerf: {value}").into_boxed_str(),
        ))
    }

    // Corrections and Calibration

    // Corrections and calibration

    // Expansion boards

    // Expansion IO control

    // Miscellaneous

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
        let dev = ManuallyDrop::new(self);
        let res = unsafe { bladerf_device_reset(dev.get_device_ptr()) };
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

    /// # Safety
    /// Intended for internal use.
    ///
    /// This function should be called in the drop implementation of the struct that implements this trait.
    unsafe fn close(&self) {
        unsafe { bladerf_close(self.get_device_ptr()) }
    }
}
