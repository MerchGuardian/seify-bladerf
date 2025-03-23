use crate::expansion_boards::Xb200;
use crate::streamers::{RxSyncStream, StreamConfig, TxSyncStream};
use crate::{error::*, sys::*, types::*, BladeRF, BladeRfAny};
use mem::ManuallyDrop;
use std::sync::Arc;
use std::*;
use sync::atomic::{AtomicBool, Ordering};

pub struct BladeRf1 {
    pub(crate) device: *mut bladerf,
    rx_stream_configured: AtomicBool,
    tx_stream_configured: AtomicBool,
}

impl core::fmt::Debug for BladeRf1 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let BladeRf1 {
            device,
            rx_stream_configured,
            tx_stream_configured,
        } = self;
        f.debug_struct("BladeRf1")
            .field("device_info", &self.info())
            .field("device_ptr", &device)
            .field("rx_stream_configured", &rx_stream_configured)
            .field("tx_stream_configured", &tx_stream_configured)
            .finish()
    }
}

unsafe impl Send for BladeRf1 {}
unsafe impl Sync for BladeRf1 {}

impl BladeRf1 {
    /// Set the PA gain in dB
    ///
    /// Values outside the range of [ BLADERF_TXVGA2_GAIN_MIN, BLADERF_TXVGA2_GAIN_MAX ] will be clamped.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___g_a_i_n.html#ga5a32dd46a54c815aa2005e50c1a5c894>
    #[deprecated(
        since = "0.1.0",
        note = "Use `set_gain` or `set_gain_stage` BladeRF trait methods"
    )]
    pub fn set_txvga2(&self, gain: i32) -> Result<()> {
        let res = unsafe { bladerf_set_txvga2(self.device, gain) };

        check_res!(res);
        Ok(())
    }

    /// Configure the sampling of the LMS6002D to be either internal or external.
    ///
    /// Internal sampling will read from the RXVGA2 driver internal to the chip. External sampling will connect the ADC inputs to the external inputs for direct sampling.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___s_a_m_p_l_i_n_g___m_u_x.html#gacc59d418bef3f33be4f927f84244d2e5>
    pub fn set_sampling(&self, sampling: Sampling) -> Result<()> {
        let res = unsafe { bladerf_set_sampling(self.device, sampling as bladerf_sampling) };
        check_res!(res);
        Ok(())
    }

    /// Read the device's current state of RXVGA2 and ADC pin connection to figure out which sampling mode it is currently configured in.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___s_a_m_p_l_i_n_g___m_u_x.html#ga03ebe323833185c5331fd586a3dca54a>
    pub fn get_sampling(&self) -> Result<Sampling> {
        let mut sampling = bladerf_sampling_BLADERF_SAMPLING_UNKNOWN;
        let res = unsafe { bladerf_get_sampling(self.device, &mut sampling) };
        check_res!(res);
        Sampling::try_from(sampling)
    }

    /// Set the LMS LPF mode to bypass or disable it
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___l_p_f___b_y_p_a_s_s.html#gada00003e9e306dec346970052c27b107>
    pub fn set_lpf_mode(&self, channel: Channel, lpf_mode: LPFMode) -> Result<()> {
        let res = unsafe {
            bladerf_set_lpf_mode(
                self.device,
                channel as bladerf_channel,
                lpf_mode as bladerf_lpf_mode,
            )
        };
        check_res!(res);
        Ok(())
    }

    /// Get the current mode of the LMS LPF
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___l_p_f___b_y_p_a_s_s.html#ga8157c866e3ef2902cb51776c0c0f1cc5>
    pub fn get_lpf_mode(&self, channel: Channel) -> Result<LPFMode> {
        let mut lpf_mode = bladerf_lpf_mode_BLADERF_LPF_NORMAL;
        let res =
            unsafe { bladerf_get_lpf_mode(self.device, channel as bladerf_channel, &mut lpf_mode) };
        check_res!(res);
        LPFMode::try_from(lpf_mode)
    }

    /// Set the current mode of operation of the SMB clock port
    ///
    /// In a MIMO configuration, one "master" device should first be configured to output its reference clock to the slave devices via:
    /// ```no_run
    /// # use bladerf::{BladeRf1, BladeRfAny, SmbMode};
    /// let device: BladeRf1 = BladeRfAny::open_first().unwrap().try_into().unwrap();
    /// device.set_smb_mode(SmbMode::Output).unwrap();
    /// ```
    ///
    /// Next, all "slave" devices should be configured to use the reference clock provided on the SMB clock port (instead of using their on-board reference) via:
    /// ```no_run
    /// # use bladerf::{BladeRf1, BladeRfAny, SmbMode};
    /// let device: BladeRf1 = BladeRfAny::open_first().unwrap().try_into().unwrap();
    /// device.set_smb_mode(SmbMode::Input).unwrap();
    /// ```
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#ga42184eb5678f687c7542b3e2abe3bb71>
    pub fn set_smb_mode(&self, mode: SmbMode) -> Result<()> {
        let res = unsafe { bladerf_set_smb_mode(self.device, mode as bladerf_smb_mode) };
        check_res!(res);
        Ok(())
    }

    /// Get the current mode of operation of the SMB clock port
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#ga622fcc384ac9192576c95b5fd6318d25>
    pub fn get_smb_mode(&self) -> Result<SmbMode> {
        let mut mode = bladerf_smb_mode_BLADERF_SMB_MODE_INVALID;
        let res = unsafe { bladerf_get_smb_mode(self.device, &mut mode) };
        check_res!(res);
        SmbMode::try_from(mode)
    }

    /// Set the SMB clock port frequency in rational Hz
    ///
    /// The frequency must be between [SMB_FREQUENCY_MIN] and [SMB_FREQUENCY_MAX].
    ///
    /// This function inherently configures the SMB clock port as an output. Do not call [BladeRf1::set_smb_mode] with [SmbMode::Output], as this will reset the output frequency to the 38.4 MHz reference.
    ///
    /// # Safety
    /// This clock should not be set if an expansion board is connected.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#gae3695b112ac64e13c90fed57b34e3207>
    pub unsafe fn set_rational_smb_frequency(
        &self,
        frequency: RationalRate,
    ) -> Result<RationalRate> {
        let mut actual_freq = bladerf_rational_rate {
            integer: 0,
            num: 0,
            den: 0,
        };
        // Despite frequency being passes as a &mut reference, the value is not actually mutated, so no need to pass it back to the user.
        let res = unsafe {
            bladerf_set_rational_smb_frequency(self.device, &mut frequency.into(), &mut actual_freq)
        };
        check_res!(res);
        Ok(actual_freq.into())
    }

    /// Read the SMB connector output frequency in rational Hz
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#gadaae300944054b14a3b3e25253db2d68>
    pub fn get_rational_smb_frequency(&self) -> Result<RationalRate> {
        let mut freq = bladerf_rational_rate {
            integer: 0,
            num: 0,
            den: 0,
        };
        let res = unsafe { bladerf_get_rational_smb_frequency(self.device, &mut freq) };
        check_res!(res);
        Ok(freq.into())
    }

    /// Set the SMB connector output frequency in Hz. Use [BladeRf1::set_rational_smb_frequency] for more arbitrary values.
    ///
    /// The frequency must be between [SMB_FREQUENCY_MIN] and [SMB_FREQUENCY_MAX].
    ///
    /// This function inherently configures the SMB clock port as an output. Do not call [BladeRf1::set_smb_mode] with [SmbMode::Output], as this will reset the output frequency to the 38.4 MHz reference.
    ///
    /// # Safety
    /// This clock should not be set if an expansion board is connected.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#gac1f39fe1facf7453d6f6fba2b5b464f1>
    pub unsafe fn set_smb_frequency(&self, frequency: u32) -> Result<u32> {
        let mut actual_freq = 0;
        let res = unsafe { bladerf_set_smb_frequency(self.device, frequency, &mut actual_freq) };
        check_res!(res);
        Ok(actual_freq)
    }

    /// Read the SMB connector output frequency in Hz
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_m_b___c_l_o_c_k.html#ga76f183a914d500fc335f207c573cfdf4>
    pub fn get_smb_frequency(&self) -> Result<u32> {
        let mut freq = 0;
        let res = unsafe { bladerf_get_smb_frequency(self.device, &mut freq) };
        check_res!(res);
        Ok(freq)
    }

    pub fn tx_streamer<T: SampleFormat>(
        &self,
        config: StreamConfig,
    ) -> Result<TxSyncStream<&Self, T, BladeRf1>> {
        // TODO: Decide Ordering
        self.tx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an TX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { TxSyncStream::new(self, config, ChannelLayoutTx::SISO(TxChannel::Tx0)) }
    }

    pub fn tx_streamer_arc<T: SampleFormat>(
        device: Arc<Self>,
        config: StreamConfig,
    ) -> Result<TxSyncStream<Arc<Self>, T, Self>> {
        // TODO: Decide Ordering
        device
            .tx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an TX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { TxSyncStream::new(device, config, ChannelLayoutTx::SISO(TxChannel::Tx0)) }
    }

    pub fn rx_streamer<T: SampleFormat>(
        &self,
        config: StreamConfig,
    ) -> Result<RxSyncStream<&Self, T, BladeRf1>> {
        // TODO: Decide Ordering
        self.rx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an RX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { RxSyncStream::new(self, config, ChannelLayoutRx::SISO(RxChannel::Rx0)) }
    }

    pub fn rx_streamer_arc<T: SampleFormat>(
        device: Arc<Self>,
        config: StreamConfig,
    ) -> Result<RxSyncStream<Arc<Self>, T, BladeRf1>> {
        // TODO: Decide Ordering
        device
            .rx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an RX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { RxSyncStream::new(device, config, ChannelLayoutRx::SISO(RxChannel::Rx0)) }
    }

    // TODO move to BladeRF trait
    fn expansion_attach(&self, module: ExpansionModule) -> Result<()> {
        let res = unsafe { bladerf_expansion_attach(self.device, module as bladerf_xb) };
        check_res!(res);
        Ok(())
    }

    // TODO move to BladeRF trait
    pub fn get_attached_expansion(&self) -> Result<ExpansionModule> {
        let mut module = bladerf_xb_BLADERF_XB_NONE;
        let res = unsafe { bladerf_expansion_get_attached(self.device, &mut module) };
        check_res!(res);
        ExpansionModule::try_from(module)
    }

    /// Gets the [Xb200] struct allowing for control of the XB200 transverter board
    pub fn get_xb200(&self) -> Result<Xb200> {
        self.expansion_attach(ExpansionModule::Xb200)?;
        Ok(Xb200 {
            device: self,
            periph_taken: false,
        })
    }
}

impl TryFrom<BladeRfAny> for BladeRf1 {
    type Error = Error;

    fn try_from(value: BladeRfAny) -> std::result::Result<Self, Self::Error> {
        if value.get_board_name() == "bladerf1" {
            let old_dev = ManuallyDrop::new(value);

            let new_dev = BladeRf1 {
                device: old_dev.device,
                rx_stream_configured: AtomicBool::new(false),
                tx_stream_configured: AtomicBool::new(false),
            };

            Ok(new_dev)
        } else {
            Err(Error::Unsupported)
        }
    }
}

impl BladeRF for BladeRf1 {
    fn get_device_ptr(&self) -> *mut bladerf {
        self.device
    }
}

impl Drop for BladeRf1 {
    fn drop(&mut self) {
        unsafe { self.close() };
    }
}
