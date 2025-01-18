use crate::stream::{RxSyncStream, SyncConfig, TxSyncStream};
use crate::{bladerf_drop, error::*, sys::*, types::*, BladeRF, BladeRfAny};
use enum_map::EnumMap;
use marker::PhantomData;
use mem::ManuallyDrop;
use parking_lot::lock_api::MutexGuard;
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

    pub(crate) fn set_sync_config<T: SampleFormat>(
        &self,
        config: &SyncConfig,
        layout: ChannelLayout,
    ) -> Result<()> {
        let res = unsafe {
            bladerf_sync_config(
                self.device,
                layout as u32,
                T::FORMAT as u32,
                config.num_buffers,
                config.buffer_size,
                config.num_transfers,
                config.stream_timeout,
            )
        };
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

        self.set_sync_config::<T>(config, layout)?;

        Ok(TxSyncStream {
            dev: &self,
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

        self.set_sync_config::<T>(config, layout)?;

        Ok(RxSyncStream {
            dev: &self,
            _format: PhantomData,
        })
    }
}

impl TryFrom<BladeRfAny> for BladeRf2 {
    type Error = Error;

    fn try_from(value: BladeRfAny) -> std::result::Result<Self, Self::Error> {
        if value.get_board_name() == "bladerf2" {
            let old_dev = ManuallyDrop::new(value);

            // Use `std::ptr::read` to move non-Copy fields out of the ManuallyDrop wrapper
            // SAFETY:
            // Being a rust reference, the following hold.
            // 1. each field is valid for reads
            // 2. each field is guaranteed to be aligned
            // 3. each field is properly initialized
            // Further
            // 4. Each field is read exactly once and then not dropped, therefore no double objects are created
            // let enabled_modules = unsafe { std::ptr::read(&old_dev.enabled_modules) };
            // let format_sync = unsafe { std::ptr::read(&old_dev.format_sync) };

            // let test = (*old_dev).enabled_modules;
            let new_dev = BladeRf2 {
                device: old_dev.device,
                rx_singleton: AtomicBool::new(false),
                tx_singleton: AtomicBool::new(false),
                // enabled_modules,
                // format_sync,
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

    fn get_enabled_modules(&self) -> MutexGuard<'_, parking_lot::RawMutex, EnumMap<Channel, bool>> {
        // self.enabled_modules.lock()
        todo!()
    }

    // fn get_enabled_modules_mut(&mut self) -> &mut EnumMap<Channel, bool> {
    //     self.enabled_modules.get_mut()
    // }
}

impl Drop for BladeRf2 {
    fn drop(&mut self) {
        bladerf_drop(self);
    }
}
