use crate::streamers::{RxSyncStream, StreamConfig, TxSyncStream};
use crate::{error::*, sys::*, types::*, BladeRF, BladeRfAny};
use mem::ManuallyDrop;
use std::*;
use sync::atomic::{AtomicBool, Ordering};

unsafe impl Send for BladeRf2 {}
unsafe impl Sync for BladeRf2 {}

pub struct BladeRf2 {
    pub(crate) device: *mut bladerf,
    rx_stream_configured: AtomicBool,
    tx_stream_configured: AtomicBool,
}

impl core::fmt::Debug for BladeRf2 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let BladeRf2 {
            device,
            rx_stream_configured,
            tx_stream_configured,
        } = self;
        f.debug_struct("BladeRf2")
            .field("device_info", &self.info())
            .field("device_ptr", &device)
            .field("rx_stream_configured", &rx_stream_configured)
            .field("tx_stream_configured", &tx_stream_configured)
            .finish()
    }
}

impl BladeRf2 {
    /// Get current bias tee state
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f2___b_i_a_s___t_e_e.html#ga308bc82fca6eaea01c714a772fd945db>
    pub fn get_bias_tee(&self, channel: Channel) -> Result<bool> {
        let mut enable = false;
        let res =
            unsafe { bladerf_get_bias_tee(self.device, channel as bladerf_channel, &mut enable) };
        check_res!(res);
        Ok(enable)
    }

    /// Enable or disable the bias tee on the specified channel.
    ///
    /// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f2___b_i_a_s___t_e_e.html#ga6289800def08a0e8f6ef77ae628e70a1>
    pub fn set_bias_tee(&self, channel: Channel, enable: bool) -> Result<()> {
        let res = unsafe { bladerf_set_bias_tee(self.device, channel as bladerf_channel, enable) };
        check_res!(res);
        Ok(())
    }

    pub fn tx_streamer<T: SampleFormat>(
        &self,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<&Self, T, BladeRf2>> {
        // TODO: Decide Ordering
        self.tx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an TX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { TxSyncStream::new(self, config, layout) }
    }

    pub fn rx_streamer<T: SampleFormat>(
        &self,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&Self, T, BladeRf2>> {
        // TODO: Decide Ordering
        self.rx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an RX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { RxSyncStream::new(self, config, layout) }
    }
}

impl TryFrom<BladeRfAny> for BladeRf2 {
    type Error = Error;

    fn try_from(value: BladeRfAny) -> std::result::Result<Self, Self::Error> {
        if value.get_board_name() == "bladerf2" {
            let old_dev = ManuallyDrop::new(value);

            let new_dev = BladeRf2 {
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

impl BladeRF for BladeRf2 {
    fn get_device_ptr(&self) -> *mut bladerf {
        self.device
    }
}

impl Drop for BladeRf2 {
    fn drop(&mut self) {
        unsafe { self.close() };
    }
}
