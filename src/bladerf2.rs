use crate::stream::{RxSyncStream, SyncConfig, TxSyncStream};
use crate::{error::*, sys::*, types::*, BladeRF, BladeRfAny};
use marker::PhantomData;
use mem::ManuallyDrop;
use sync::atomic::{AtomicBool, Ordering};
// use parking_lot::Mutex;
use std::*;
// use sync::RwLock;

unsafe impl Send for BladeRf2 {}
unsafe impl Sync for BladeRf2 {}

pub struct BladeRf2 {
    pub(crate) device: *mut bladerf,
    rx_singleton: AtomicBool,
    tx_singleton: AtomicBool,
    // enabled_modules: Mutex<EnumMap<Channel, bool>>,
    // format_sync: RwLock<Option<Format>>,
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
        mimo: bool,
    ) -> Result<TxSyncStream<T, BladeRf2>> {
        if self.tx_singleton.load(Ordering::Relaxed) {
            return Err(Error::Msg(
                "Already have a TX stream open".to_owned().into_boxed_str(),
            ));
        } else {
            self.tx_singleton.store(true, Ordering::Relaxed);
        }

        let layout = if mimo {
            ChannelLayout::TxMIMO
        } else {
            ChannelLayout::TxSISO
        };

        unsafe {
            self.set_sync_config::<T>(config, layout)?;
        }

        Ok(TxSyncStream {
            dev: self,
            _format: PhantomData,
        })
    }

    pub fn rx_streamer<T: SampleFormat>(
        &self,
        config: &SyncConfig,
        mimo: bool,
    ) -> Result<RxSyncStream<T, BladeRf2>> {
        if self.rx_singleton.load(Ordering::Relaxed) {
            return Err(Error::Msg(
                "Already have an RX stream open".to_owned().into_boxed_str(),
            ));
        } else {
            self.rx_singleton.store(true, Ordering::Relaxed);
        }

        let layout = if mimo {
            ChannelLayout::RxMIMO
        } else {
            ChannelLayout::RxSISO
        };

        unsafe {
            self.set_sync_config::<T>(config, layout)?;
        }

        Ok(RxSyncStream {
            dev: self,
            _format: PhantomData,
        })
    }
}

impl TryFrom<BladeRfAny> for BladeRf2 {
    type Error = Error;

    fn try_from(value: BladeRfAny) -> std::result::Result<Self, Self::Error> {
        if value.get_board_name() == "bladerf2" {
            let old_dev = ManuallyDrop::new(value);

            let new_dev = BladeRf2 {
                device: old_dev.device,
                rx_singleton: AtomicBool::new(false),
                tx_singleton: AtomicBool::new(false),
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
