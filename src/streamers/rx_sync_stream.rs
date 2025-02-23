use std::borrow::Borrow;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use libbladerf_sys as sys;

use crate::BladeRF;
use crate::BladeRf1;
use crate::BladeRf2;
use crate::BladeRfAny;
use crate::Channel;
use crate::ChannelLayoutRx;
use crate::Result;
use crate::RxChannel;
use crate::SampleFormat;

use super::SyncConfig;

pub struct RxSyncStream<T: Borrow<D>, F: SampleFormat, D: BladeRF> {
    pub(crate) dev: T,
    pub(crate) layout: ChannelLayoutRx,
    pub(crate) _devtype: PhantomData<D>,
    pub(crate) _format: PhantomData<F>,
}

impl<T: Borrow<D>, F: SampleFormat, D: BladeRF> RxSyncStream<T, F, D> {
    pub fn read(&self, buffer: &mut [F], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_rx(
                self.dev.borrow().get_device_ptr(),
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                std::ptr::null_mut(),
                timeout.as_millis() as u32,
            )
        };
        check_res!(res);
        Ok(())
    }

    pub(crate) fn new(
        dev: T,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<T, F, D>> {
        unsafe {
            dev.borrow().set_sync_config::<F>(config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev,
            layout,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

impl<'a, F: SampleFormat, D: BladeRF> RxSyncStream<&'a D, F, D> {
    fn reconfigure_inner<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&'a D, NF, D>> {
        RxSyncStream::new(self.dev, config, layout)
    }
}

impl<F: SampleFormat, D: BladeRF> RxSyncStream<Arc<D>, F, D> {
    fn reconfigure_inner<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<D>, NF, D>> {
        RxSyncStream::new(self.dev.clone(), config, layout)
    }
}

impl<T: Borrow<D>, F: SampleFormat, D: BladeRF> Drop for RxSyncStream<T, F, D> {
    fn drop(&mut self) {
        // Ignore the results, just try disable both channels even if they don't exist on the dev.
        let _ = self.dev.borrow().set_enable_module(Channel::Rx0, false);
        let _ = self.dev.borrow().set_enable_module(Channel::Rx1, false);
    }
}

////////////////////////////////////////////////////////////////////////////////
// RX Stream Brf1

impl<T: Borrow<BladeRf1> + Clone, F: SampleFormat> RxSyncStream<T, F, BladeRf1> {
    pub fn enable(&self) -> Result<()> {
        self.dev.borrow().set_enable_module(Channel::Rx0, true)
    }

    pub fn disable(&self) -> Result<()> {
        self.dev.borrow().set_enable_module(Channel::Rx0, false)
    }
}

impl<'a, F: SampleFormat> RxSyncStream<&'a BladeRf1, F, BladeRf1> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
    ) -> Result<RxSyncStream<&'a BladeRf1, NF, BladeRf1>> {
        self.reconfigure_inner(config, ChannelLayoutRx::SISO(RxChannel::Rx0))
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRf1>, F, BladeRf1> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
    ) -> Result<RxSyncStream<Arc<BladeRf1>, NF, BladeRf1>> {
        self.reconfigure_inner(config, ChannelLayoutRx::SISO(RxChannel::Rx0))
    }
}

////////////////////////////////////////////////////////////////////////////////
// RX Stream Brf2

impl<T: Borrow<BladeRf2> + Clone, F: SampleFormat> RxSyncStream<T, F, BladeRf2> {
    pub fn enable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutRx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), true),
            ChannelLayoutRx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Rx0, true)?;
                self.dev.borrow().set_enable_module(Channel::Rx1, true)?;
                Ok(())
            }
        }
    }

    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutRx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), false),
            ChannelLayoutRx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Rx0, false)?;
                self.dev.borrow().set_enable_module(Channel::Rx1, false)?;
                Ok(())
            }
        }
    }
}

impl<'a, F: SampleFormat> RxSyncStream<&'a BladeRf2, F, BladeRf2> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&'a BladeRf2, NF, BladeRf2>> {
        self.reconfigure_inner(config, layout)
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRf2>, F, BladeRf2> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<BladeRf2>, NF, BladeRf2>> {
        self.reconfigure_inner(config, layout)
    }
}

////////////////////////////////////////////////////////////////////////////////
// RX Stream BrfAny

impl<T: Borrow<BladeRfAny> + Clone, F: SampleFormat> RxSyncStream<T, F, BladeRfAny> {
    pub fn enable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutRx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), true),
            ChannelLayoutRx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Rx0, true)?;
                self.dev.borrow().set_enable_module(Channel::Rx1, true)?;
                Ok(())
            }
        }
    }

    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutRx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), false),
            ChannelLayoutRx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Rx0, false)?;
                self.dev.borrow().set_enable_module(Channel::Rx1, false)?;
                Ok(())
            }
        }
    }
}

impl<'a, F: SampleFormat> RxSyncStream<&'a BladeRfAny, F, BladeRfAny> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&'a BladeRfAny, NF, BladeRfAny>> {
        self.reconfigure_inner(config, layout)
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRfAny>, F, BladeRfAny> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<BladeRfAny>, NF, BladeRfAny>> {
        self.reconfigure_inner(config, layout)
    }
}
