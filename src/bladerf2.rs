use crate::streamers::{RxSyncStream, SyncConfig, TxSyncStream};
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

impl BladeRf2 {
    pub fn get_bias_tee(&self, channel: Channel) -> Result<bool> {
        let mut enable = false;
        let res =
            unsafe { bladerf_get_bias_tee(self.device, channel as bladerf_channel, &mut enable) };
        check_res!(res);
        Ok(enable)
    }

    pub fn set_bias_tee(&self, channel: Channel, enable: bool) -> Result<()> {
        let res = unsafe { bladerf_set_bias_tee(self.device, channel as bladerf_channel, enable) };
        check_res!(res);
        Ok(())
    }

    pub fn tx_streamer<T: SampleFormat>(
        &self,
        config: &SyncConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<&Self, T, BladeRf2>> {
        // TODO: Decide Ordering
        self.tx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an TX stream open".to_owned().into_boxed_str())
            })?;

        TxSyncStream::new(self, config, layout)
    }

    pub fn rx_streamer<T: SampleFormat>(
        &self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&Self, T, BladeRf2>> {
        // TODO: Decide Ordering
        self.rx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an RX stream open".to_owned().into_boxed_str())
            })?;

        RxSyncStream::new(self, config, layout)
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
