use crate::{error::*, sys::*, types::*, RxSyncStream, StreamConfig, TxSyncStream};
use ffi::{c_char, CStr, CString};
use path::Path;
use std::{mem::ManuallyDrop, sync::Arc, *};
use sync::atomic::{AtomicBool, Ordering};

// Macro to simplify integer returns
macro_rules! check_res {
    ($e:expr) => (
    	if $e < 0 {
			return Err($crate::Error::from_bladerf_code($e as isize))
		}
	);
}

/// Environment variable containing the path to the FPGA bitstream file
pub const FPGA_BITSTREAM_VAR_NAME: &str = "BLADERF_RS_FPGA_BITSTREAM_PATH";

unsafe impl Send for BladeRfAny {}
unsafe impl Sync for BladeRfAny {}

pub struct BladeRfAny {
    pub(crate) device: *mut bladerf,
    pub(crate) rx_stream_configured: AtomicBool,
    pub(crate) tx_stream_configured: AtomicBool,
}

impl core::fmt::Debug for BladeRfAny {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let BladeRfAny {
            device,
            rx_stream_configured,
            tx_stream_configured,
        } = self;
        f.debug_struct("BladeRfAny")
            .field("device_info", &self.info())
            .field("device_ptr", &device)
            .field("rx_stream_configured", &rx_stream_configured)
            .field("tx_stream_configured", &tx_stream_configured)
            .finish()
    }
}

impl BladeRfAny {
    /// Opens the first available BladeRF device
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_i_t.html#gab341ac98615f393da9158ea59cdb6a24>
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

    /// Opens a BladeRF device with the given device identifier string
    ///
    /// The general form of the device identifier string is:
    /// `<backend>:[device=<bus>:<addr>] [instance=<n>] [serial=<serial>]`
    ///
    /// - `<backend>`: The backend to use.
    ///     - `*`: Use any available backend
    ///     - `libusb`: libusb (See libusb changelog notes for required version, given your OS and controller)
    ///     - `cypress`: Cypress CyUSB/CyAPI backend (Windows only)
    ///
    /// - `device=<bus>:<addr>`: Specifies USB bus and address. Decimal or hex prefixed by '0x' is permitted.
    /// - `instance=<n>`: Nth instance encountered, 0-indexed.
    /// - `serial=<serial>`: Serial number of the device to open.
    ///
    /// ```no_run
    /// use bladerf::BladeRfAny;
    /// let device = BladeRfAny::open_identifier("*:serial=deadbeef").unwrap();
    /// ```
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_i_t.html#gab341ac98615f393da9158ea59cdb6a24>
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

    /// Opens a BladeRF device with the given device information
    ///
    /// A list of devices can be obtained using [get_device_list()][crate::get_device_list]
    ///
    /// ```no_run
    /// use bladerf::BladeRfAny;
    /// // Get a list of devices
    /// let devices = bladerf::get_device_list().unwrap();
    /// let device = BladeRfAny::open_with_devinfo(&devices[0]).unwrap();
    ///
    /// // Alternatively, construct DevInfo manually
    /// todo!()
    /// ```
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_i_t.html#gace4d5607aacba15ccd2d5361d5eb020e>
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
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<&Self, T, Self>> {
        // TODO: Decide Ordering
        self.tx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an TX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { TxSyncStream::new(self, config, layout) }
    }

    pub fn tx_streamer_arc<T: SampleFormat>(
        device: Arc<Self>,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<Arc<Self>, T, Self>> {
        // TODO: Decide Ordering
        device
            .tx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an TX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { TxSyncStream::new(device, config, layout) }
    }

    pub fn rx_streamer<T: SampleFormat>(
        &self,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&Self, T, BladeRfAny>> {
        // TODO: Decide Ordering
        self.rx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an RX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { RxSyncStream::new(self, config, layout) }
    }

    pub fn rx_streamer_arc<T: SampleFormat>(
        device: Arc<Self>,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<Self>, T, BladeRfAny>> {
        // TODO: Decide Ordering
        device
            .rx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an RX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { RxSyncStream::new(device, config, layout) }
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
    /// Gets a raw pointer to the device as used in `libbladerf`
    fn get_device_ptr(&self) -> *mut bladerf;

    /// Get info about the device
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_i_t.html#ga20dad2500cb682a8afa31afc56d8cd4f>
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

    /// Get the serial number of the device
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html#ga3a877bcbdc89589c95611c89e647a651>
    fn get_serial(&self) -> Result<String> {
        let mut serial_data = [0i8; BLADERF_SERIAL_LENGTH as usize];

        // TODO: This method is now depricated, should instead use bladerf_get_serial_struct(). The documentation comment links to the new version
        let res =
            unsafe { bladerf_get_serial(self.get_device_ptr(), serial_data.as_mut_ptr().cast()) };

        check_res!(res);
        let serial_cstr = unsafe { CStr::from_ptr(serial_data.as_ptr().cast()) };
        let serial_str = serial_cstr
            .to_str()
            .map_err(|e| Error::msg(format!("Serial number is not UTF-8: {e:?}")))?;

        Ok(serial_str.to_string())
    }

    /// Gets the speed of the USB bus that the device is connected to.
    ///
    /// See [DeviceSpeed] for more information.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html#gaffb76ef5b491e95584fc43d45e4ced14>
    fn get_device_speed(&self) -> Result<DeviceSpeed> {
        let speed = unsafe { bladerf_device_speed(self.get_device_ptr()) };
        speed.try_into()
    }

    /// Get the FPGA size of the device
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html#gaaec5953b58fd9bca3c0cec9f9655b6a0>
    fn get_fpga_size(&self) -> Result<FpgaSize> {
        let mut fpga_size: bladerf_fpga_size = bladerf_fpga_size_BLADERF_FPGA_UNKNOWN;
        let res = unsafe { bladerf_get_fpga_size(self.get_device_ptr(), &mut fpga_size) };
        check_res!(res);
        fpga_size.try_into()
    }

    /// Get the version of the FX3 firmware on the device
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html#gaab03fbf0ae23b0251251842c86dc8d3b>
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

    /// Check if the FPGA is configured
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html#ga6cf59976f738efde781dc676fb41f1fd>
    fn is_fpga_configured(&self) -> Result<bool> {
        let res = unsafe { bladerf_is_fpga_configured(self.get_device_ptr()) };
        check_res!(res);

        match res {
            1 => Ok(true),
            0 => Ok(false),
            _ => unreachable!("bladerf_is_fpga_configured returned invalid value: {res}"),
        }
    }

    /// Get the version of the FPGA on the device
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html#gad563a6dab55204254e2286e1c417351c>
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

    /// Configure the channel's sample rate to the specified rate in Hz.
    ///
    /// Returns the actual sample rate set.
    ///
    /// Once can use [set_rational_sample_rate][BladeRF::set_rational_sample_rate] to set a more arbitrary value
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_a_m_p_l_i_n_g.html#gaf118558cccf01ada2f2eeafed6f439e8>
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

    /// Configure the channel's sample rate as a rational fraction of Hz.
    ///
    /// Returns the actual sample rate set.
    ///
    /// Use [get_sample_rate_range][BladeRF::get_sample_rate_range] to determine the range of supported sample rates.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_a_m_p_l_i_n_g.html#gae62c8c51b2ed33b22041c5cb593c3b8d>
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

    /// Get the channel's current sample rate in Hz
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_a_m_p_l_i_n_g.html#ga836a5ad4b2f23103d14efd34d2391b6f>
    fn get_sample_rate(&self, channel: Channel) -> Result<u32> {
        let mut rate: u32 = 0;

        let res = unsafe {
            bladerf_get_sample_rate(self.get_device_ptr(), channel as bladerf_channel, &mut rate)
        };
        check_res!(res);
        Ok(rate)
    }

    /// Get the channel's sample rate in rational Hz
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_a_m_p_l_i_n_g.html#gab244448d175850ceacc5d0110f18c376>
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

    /// Get the channel's supported range of sample rates
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_a_m_p_l_i_n_g.html#ga5171399454e02fa2278c380cd0390032>
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
    /// If this function is called elsewhere, it may cause issues with the used streamer since the type can arbitrarily be changed.
    #[doc(hidden)]
    unsafe fn set_sync_config<T: SampleFormat>(
        &self,
        config: &StreamConfig,
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

    /// Set the current RX Mux mode
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___r_e_c_e_i_v_e___m_u_x.html#ga9cc18ba58d0cdf3bc311c6bdf5e99a00>
    fn set_rx_mux(&self, mux: RxMux) -> Result<()> {
        let res = unsafe { bladerf_set_rx_mux(self.get_device_ptr(), mux as bladerf_rx_mux) };
        check_res!(res);
        Ok(())
    }

    /// Gets the current RX Mux mode
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___r_e_c_e_i_v_e___m_u_x.html#ga9833afff874c98b4d021d0acad6cbc54>
    fn get_rx_mux(&self) -> Result<RxMux> {
        let mut mux = bladerf_rx_mux_BLADERF_RX_MUX_INVALID;
        let res = unsafe { bladerf_get_rx_mux(self.get_device_ptr(), &mut mux) };
        check_res!(res);
        RxMux::try_from(mux)
    }

    // Configure bandwidth

    /// Set the bandwidth of the channel to the specified value in Hz
    ///
    /// The underlying device is capable of a discrete set of bandwidth values, the actual bandwidth set is returned.
    /// Use [get_bandwidth_range][BladeRF::get_bandwidth_range] to see valid bandwidth values.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_a_n_d_w_i_d_t_h.html#ga0990053e727a23e03785d3802f55c0b3>
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

    /// Get the bandwidth of the channel
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_a_n_d_w_i_d_t_h.html#ga7bc4f8f6f9b27871da27eb7e43a6d678>
    fn get_bandwidth(&self, ch: Channel) -> Result<u32> {
        let mut bandwidth: u32 = 0;
        let res = unsafe {
            bladerf_get_bandwidth(self.get_device_ptr(), ch as bladerf_channel, &mut bandwidth)
        };
        check_res!(res);
        Ok(bandwidth)
    }

    /// Get the supported range of bandwidths for a channel
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_a_n_d_w_i_d_t_h.html#gaad232b9b2d853ac4206a44d3e6ab6a60>
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

    /// Select the appropriate band path given a frequency in Hz.
    ///
    /// <div class="warning">
    ///
    /// Most API users will not need to use this function, as [set_frequency()][BladeRF::set_frequency()] calls this internally after tuning the device.
    ///
    /// </div>
    ///
    /// The high band is used for frequency above 1.5 GHz on bladeRF1 and above 3.0 GHz on bladeRF2. Otherwise, the low band is used.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_u_n_i_n_g.html#ga6e1bbb67b270d7c7b91779f0bcac655e>
    fn select_band(&self, channel: Channel, frequency: u64) -> Result<()> {
        let res = unsafe {
            bladerf_select_band(self.get_device_ptr(), channel as bladerf_channel, frequency)
        };
        check_res!(res);
        Ok(())
    }

    /// Set channel's frequency in Hz.
    ///
    /// <div class="warning">
    ///
    /// On the bladeRF1 platform, it is recommended to keep the RX and TX frequencies at least 1 MHz apart, and to digitally mix on the RX side if reception closer to the TX frequency is required.
    ///
    /// On the bladeRF2, there is one oscillator for all RX channels and one oscillator for all TX channels. Therefore, changing one channel will change the frequency of all channels in that direction.
    ///
    /// </div>
    ///
    /// This function calls [select_band()][BladeRF::select_band] internally, and performs all other tasks required to prepare the channel for the given frequency.
    ///
    /// See also [get_frequency_range()][BladeRF::get_frequency_range] to see the valid frequency range and steps for the device.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_u_n_i_n_g.html#ga4e9b635f18a9531bcd3c6b4d2dd8a4e0>
    fn set_frequency(&self, channel: Channel, frequency: u64) -> Result<()> {
        let res = unsafe {
            bladerf_set_frequency(self.get_device_ptr(), channel as bladerf_channel, frequency)
        };
        check_res!(res);
        Ok(())
    }

    /// Get channel's current frequency in Hz
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_u_n_i_n_g.html#ga395669f90d79052411839ae3e7528335>
    fn get_frequency(&self, channel: Channel) -> Result<u64> {
        let mut freq: u64 = 0;
        let res = unsafe {
            bladerf_get_frequency(self.get_device_ptr(), channel as bladerf_channel, &mut freq)
        };
        check_res!(res);
        Ok(freq)
    }

    /// Get the supported range of frequencies for a channel
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_u_n_i_n_g.html#gaea9159af0077b00e86694a73b6261798>
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

    /// Schedule a frequency retune to occur at specified sample timestamp value.
    ///
    /// <div class="warning">
    ///
    /// A [TxSyncStream] or [RxSyncStream] must be configured with metadata (Currently cannot be used with our bindings).
    ///
    /// If the underlying queue of scheduled retune requests becomes full, [Error::QueueFull] will be returned. In this case, it should be possible to schedule a retune after the timestamp of one of the earlier requests occurs.
    ///
    /// </div>
    ///
    /// TODO: Get this moved over as a method to the streamer structs once we add the ability to do metadata
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_c_h_e_d_u_l_e_d___t_u_n_i_n_g.html#gad7bd11c5784e78af7ae8fab26f4605fa>
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

    /// Cancel all pending scheduled retune operations for the specified channel.
    ///
    /// Automatically done on [Drop]
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_c_h_e_d_u_l_e_d___t_u_n_i_n_g.html#gae6b42de62294072ffb63058001b89a42>
    fn cancel_scheduled_retune(&self, channel: Channel) -> Result<()> {
        let res = unsafe {
            bladerf_cancel_scheduled_retunes(self.get_device_ptr(), channel as bladerf_channel)
        };
        check_res!(res);
        Ok(())
    }

    /// Fetch parameters used to tune the transceiver to the current frequency for use with [schedule_retune()][BladeRF::schedule_retune] to perform a "quick retune."
    ///
    /// <div class="warning">
    /// These parameters are sensitive to changes in the operating environment, and should be "refreshed" if planning to use the "quick retune" functionality over a long period of time.
    ///
    /// [set_frequency()][BladeRF::set_frequency] or [schedule_retune()][BladeRF::schedule_retune] have previously been used to retune to the desired frequency.
    /// </div>
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_c_h_e_d_u_l_e_d___t_u_n_i_n_g.html#ga5cb5018f2acc2b25e2690e96439a029c>
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

    /// Set the device's tuning mode
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_u_n_i_n_g___m_o_d_e.html#ga0fcddbdffebc03da8f96781b0b6d096b>
    fn set_tuning_mode(&self, mode: TuningMode) -> Result<()> {
        let res =
            unsafe { bladerf_set_tuning_mode(self.get_device_ptr(), mode as bladerf_tuning_mode) };
        check_res!(res);
        Ok(())
    }

    /// Get the device's current tuning mode
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_u_n_i_n_g___m_o_d_e.html#ga9f33fa7b48ea563fd2f371b583e421a9>
    fn get_tuning_mode(&self) -> Result<TuningMode> {
        let mut mode = bladerf_tuning_mode_BLADERF_TUNING_MODE_INVALID;
        let res = unsafe { bladerf_get_tuning_mode(self.get_device_ptr(), &mut mode) };
        check_res!(res);
        TuningMode::try_from(mode)
    }

    // **Loopback Functions**

    /// Get loopback modes
    ///
    /// Populates modes with a pointer to an array of structs containing the supported loopback modes.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___l_o_o_p_b_a_c_k.html#gae16e66fde699468c4641b767fe6c29ba>
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
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___l_o_o_p_b_a_c_k.html#ga8d1c68e7de9492c18fa9a3c1af1f6a98>
    fn is_loopback_mode_supported(&self, mode: Loopback) -> Result<bool> {
        let supported = unsafe {
            bladerf_is_loopback_mode_supported(self.get_device_ptr(), mode as bladerf_loopback)
        };
        Ok(supported)
    }

    /// Apply specified loopback mode
    ///
    /// # Safety
    /// Loopback modes should only be enabled or disabled while the RX and TX channels are both disabled (and therefore, when no samples are being actively streamed). Otherwise, unexpected behavior may occur.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___l_o_o_p_b_a_c_k.html#ga8d6398bfafd7541cabc9cab5ab2bd709>
    unsafe fn set_loopback(&self, loopback: Loopback) -> Result<()> {
        let res =
            unsafe { bladerf_set_loopback(self.get_device_ptr(), loopback as bladerf_loopback) };
        check_res!(res);
        Ok(())
    }

    /// Get current loopback mode
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___l_o_o_p_b_a_c_k.html#gaec18d3745073622e8584eca93d299731>
    fn get_loopback(&self) -> Result<Loopback> {
        unsafe {
            let mut loopback = bladerf_loopback_BLADERF_LB_NONE;

            let res = bladerf_get_loopback(self.get_device_ptr(), &mut loopback);
            check_res!(res);

            Loopback::try_from(loopback)
        }
    }

    // **Gain Control Functions**

    /// Set overall system gain in dB
    ///
    /// This sets an overall system gain, optimally proportioning the gain between multiple gain stages if applicable.
    ///
    /// Use [get_gain_range()][BladeRF::get_gain_range] to see the valid gain range for the device.
    ///
    /// On receive channels, 60 dB is the maximum gain level.
    ///
    /// On transmit channels, 60 dB is defined as approximately 0 dBm. Note that this is not a calibrated value, and the actual output power will vary based on a multitude of factors.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#gade4256dc2bd29d9c9e69c39beb9e12ff>
    fn set_gain(&self, channel: Channel, gain: Gain) -> Result<()> {
        let res =
            unsafe { bladerf_set_gain(self.get_device_ptr(), channel as bladerf_channel, gain) };
        check_res!(res);
        Ok(())
    }

    /// Get overall system gain in dB
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#gaff3b110dc02420b6234252861680c987>
    fn get_gain(&self, channel: Channel) -> Result<Gain> {
        let mut gain: Gain = 0;
        let res = unsafe {
            bladerf_get_gain(self.get_device_ptr(), channel as bladerf_channel, &mut gain)
        };
        check_res!(res);
        Ok(gain)
    }

    /// Set gain control mode
    ///
    /// Sets the mode for hardware AGC. Not all channels or boards will support all possible values (e.g. transmit channels); invalid combinations will return [Error::Unsupported].
    ///
    /// The special value of [GainMode::Default] will return hardware AGC to its default value at initialization.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#ga3961ee343f228f748bf1e1a15b749c58>
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
    ///
    /// Gets the current mode for hardware AGC. If the channel or board does not meaningfully have a gain mode (e.g. transmit channels), mode will be set to [GainMode::Default].
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#ga1e60204959de8aae680b653a5d27d7bf>
    fn get_gain_mode(&self, channel: Channel) -> Result<GainMode> {
        let mut mode = bladerf_gain_mode_BLADERF_GAIN_DEFAULT;
        let res = unsafe {
            bladerf_get_gain_mode(self.get_device_ptr(), channel as bladerf_channel, &mut mode)
        };
        check_res!(res);
        GainMode::try_from(mode)
    }

    /// Get available gain control modes
    ///
    /// Returns a list of available gain modes for the specified channel.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#ga5350f1868a06dc92bef4ea0f07914f42>
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
    ///
    /// <div class="warning">
    ///     This may vary depending on the configured frequency, so it should be checked after setting the desired frequency.
    /// </div>
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#ga8dbf638a41ff41b1c03d0f1253b7229f>
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
    ///
    /// <div class="warning">
    ///
    /// Values outside the valid gain range will be clipped.
    ///
    /// </div>
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#ga58e4c9ced3c0b0e80260a5fc5ec870cf>
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
    ///
    /// Note that, in some cases, gain may be negative (e.g. transmit channels).
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#ga6c37a7504fa1603a79c2109ac8110305>
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
    ///
    /// <div class="warning">
    ///
    /// This may vary depending on the configured frequency, so it should be checked after setting the desired frequency.
    ///
    /// </div>
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#gaafea0b69946cce161725fb0e8819b3ab>
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
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___g_a_i_n.html#ga7eef4832398072f7222ee29c8e703bf8>
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
    ///
    /// # Safety
    /// See the BladeRF Docs here: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html>
    ///     
    /// The enabling and disabling of sample streams is done by calls to `enable()` and `disable()` on `RxSyncStream` and `TxSyncStream`
    ///
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html#ga3c445a33199db3a657db5fae6fa96f23>
    unsafe fn trigger_init(&self, channel: Channel, signal: TriggerSignal) -> Result<Trigger> {
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
    ///
    /// <div class="warning">
    ///
    /// Configuring two devices in the trigger chain (or both RX and TX on a single device) as masters can damage the associated FPGA pins, as this would cause contention over the trigger signal. *Ensure only one device in the chain is configured as the master!*
    ///
    /// If [Trigger::role] is set to [TriggerRole::Disabled], this will inherently disarm an armed trigger and clear any fire requests, regardless of the value of arm.
    ///
    /// </div>
    ///
    /// # Safety
    /// See the BladeRF Docs here: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html>
    ///     
    /// The enabling and disabling of sample streams is done by calls to `enable()` and `disable()` on `RxSyncStream` and `TxSyncStream`
    ///
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html#ga14afff57873c8ae591a4142d7851a869>
    unsafe fn trigger_arm(&self, trigger: &Trigger, arm: bool) -> Result<()> {
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
    ///
    /// Calling this function with a trigger whose role is anything other than [TriggerRole::Master] will yield a [Error::Inval] return value.
    ///
    /// # Safety
    /// See the BladeRF Docs here: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html>
    ///     
    /// The enabling and disabling of sample streams is done by calls to `enable()` and `disable()` on `RxSyncStream` and `TxSyncStream`
    ///
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html#gaaa2b932a3b810203952bb49c1673c124>
    unsafe fn trigger_fire(&self, trigger: &Trigger) -> Result<()> {
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
    ///
    /// # Safety
    /// See the BladeRF Docs here: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html>
    ///     
    /// The enabling and disabling of sample streams is done by calls to `enable()` and `disable()` on `RxSyncStream` and `TxSyncStream`
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html#ga63c07df2a4c7a533824d0faaeedc3a1a>
    unsafe fn trigger_state(&self, trigger: &Trigger) -> Result<(bool, bool, bool)> {
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

    // **Correction Functions**

    /// Set the value of the specified correction parameter
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___c_o_r_r.html#ga049bf80b85afe541bcb554c83ff58e34>
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
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___c_o_r_r.html#gad59e1b8ddacdfa576f82767ad3b2697d>
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

    // Miscellaneous

    /// Retrieve the current timestamp
    ///
    /// This function is only intended to be used to retrieve a coarse estimate of the current timestamp when starting up a stream. It *should not* be used as a means to accurately retrieve the current timestamp of individual samples within a running stream. The reasons for this are:
    /// - The timestamp counter will have advanced during the time that the captured value is propagated back from the FPGA to the host
    /// - The value retrieved in this manner is not tightly-coupled with specific sample positions in the stream.
    ///
    ///
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___s_t_r_e_a_m_i_n_g.html#gaf3318afe306b3a06c4b3b244546376f7>
    ///
    // TODO Finish documentation comment once we add the metadata capability
    fn get_timestamp(&self, dir: Direction) -> Result<u64> {
        let mut timestamp: u64 = 0;
        let res =
            unsafe { bladerf_get_timestamp(self.get_device_ptr(), dir.into(), &mut timestamp) };
        check_res!(res);
        Ok(timestamp)
    }

    // Device loading and programming

    /// Write FX3 firmware to the bladeRF’s SPI flash
    /// <div class="warning">This will require a power cycle to take effect</div>
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___p_r_o_g.html#ga440d82d6c5e9b915ce3e1cb1af69ecb7>
    fn flash_firmware(&self, firmware_path: impl AsRef<Path>) -> Result<()> {
        let bitstream_path = CString::new(firmware_path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(|e| Error::msg(format!("Invalid path for cstring: {e:?}")))?;

        let res = unsafe { bladerf_flash_firmware(self.get_device_ptr(), bitstream_path.as_ptr()) };
        check_res!(res);
        Ok(())
    }

    /// Reset the device, causing it to reload its firmware from flash
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___p_r_o_g.html#gac74d3ec03dae7651b4c27ea8fab84a03>
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

    /// Load device's FPGA from the given path.
    ///
    /// <div class="warning">This will require a power cycle to take effect</div>
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___p_r_o_g.html#ga2458993d78dc20c63d17093081655d08>
    fn load_fpga_path(&self, bitstream_path: impl AsRef<Path>) -> Result<()> {
        let bitstream_path = CString::new(bitstream_path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(|e| Error::msg(format!("Invalid path for cstring: {e:?}")))?;

        let res = unsafe { bladerf_load_fpga(self.get_device_ptr(), bitstream_path.as_ptr()) };
        check_res!(res);
        Ok(())
    }

    /// Write the provided FPGA image to the bladeRF's SPI flash and enable FPGA loading from SPI flash at power on (also referred to within this project as FPGA "autoloading").
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___p_r_o_g.html#gaf1d77c49beeaaad2f7fd25250b645c88>
    fn flash_fpga(&self, bitstream_path: impl AsRef<Path>) -> Result<()> {
        let bitstream_path = CString::new(bitstream_path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(|e| Error::msg(format!("Invalid path for cstring: {e:?}")))?;

        let res = unsafe { bladerf_flash_fpga(self.get_device_ptr(), bitstream_path.as_ptr()) };
        check_res!(res);
        Ok(())
    }

    /// Erase the FPGA region of SPI flash, effectively disabling FPGA autoloading
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___p_r_o_g.html#gad346e1ea98c82dde2d3c963fe6fec6e2>
    fn erase_stored_fpga(&self) -> Result<()> {
        let res = unsafe { bladerf_erase_stored_fpga(self.get_device_ptr()) };
        check_res!(res);
        Ok(())
    }

    /// Read firmware log data and write it to the specified file
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___p_r_o_g.html#ga1af00f78739d7c6fe5078075418a5fc6>
    // TODO the path should be an option where None indicates stdout and a null pointer is passed into the bladerf_get_fw_log function
    fn get_fw_log(&self, path: impl AsRef<Path>) -> Result<()> {
        let log_path = CString::new(path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(|e| Error::msg(format!("Invalid path for cstring: {e:?}")))?;
        let res = unsafe { bladerf_get_fw_log(self.get_device_ptr(), log_path.as_ptr()) };
        check_res!(res);
        Ok(())
    }

    /// Higher level control of one RF channel/module
    fn configure_module(&self, channel: Channel, config: ModuleConfig) -> Result<()> {
        self.set_frequency(channel, config.frequency)?;
        self.set_sample_rate(channel, config.sample_rate)?;
        self.set_bandwidth(channel, config.bandwidth)?;
        self.set_gain(channel, config.gain)?;

        Ok(())
    }

    /// Get the board name
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___i_n_f_o.html#gaf62ea531c9dd725733e568534df4c6ba>
    fn get_board_name(&self) -> &'static str {
        // Safety, the function returns a string that is compiled in (static I guess? is there another term I should use?)
        let name_raw = unsafe { CStr::from_ptr(bladerf_get_board_name(self.get_device_ptr())) };
        name_raw.to_str().unwrap()
    }

    /// # Safety
    /// Intended for internal use.
    ///
    /// This function should be called in the drop implementation of the struct that implements this trait.
    #[doc(hidden)]
    unsafe fn close(&self) {
        unsafe { bladerf_close(self.get_device_ptr()) }
    }
}
