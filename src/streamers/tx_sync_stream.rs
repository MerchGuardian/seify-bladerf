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
use crate::ChannelLayout;
use crate::ChannelLayoutTx;
use crate::Result;
use crate::SampleFormat;

use super::SyncConfig;

pub struct TxSyncStream<'a, T: SampleFormat, D: BladeRF> {
    pub(crate) dev: &'a D,
    pub(crate) layout: ChannelLayoutTx,
    pub(crate) _format: PhantomData<T>,
}

impl<'a, T: SampleFormat> TxSyncStream<'a, T, BladeRf1> {
    pub fn write(&self, buffer: &[T], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_tx(
                self.dev.device,
                buffer.as_ptr() as *const _,
                buffer.len() as u32,
                std::ptr::null_mut(),
                timeout.as_millis() as u32,
            )
        };
        check_res!(res);
        Ok(())
    }

    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
    ) -> Result<TxSyncStream<'a, NF, BladeRf1>> {
        unsafe {
            self.dev
                .set_sync_config::<NF>(config, ChannelLayout::TxSISO)?;
        }

        Ok(TxSyncStream {
            dev: self.dev,
            layout: ChannelLayoutTx::SISO(crate::TxChannel::Tx0),
            _format: PhantomData,
        })
    }

    pub fn enable(&self) -> Result<()> {
        self.dev.set_enable_module(Channel::Tx0, true)
    }

    pub fn disable(&self) -> Result<()> {
        self.dev.set_enable_module(Channel::Tx0, false)
    }
}

impl<'a, T: SampleFormat> TxSyncStream<'a, T, BladeRf2> {
    pub fn write(&self, buffer: &[T], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_tx(
                self.dev.device,
                buffer.as_ptr() as *const _,
                buffer.len() as u32,
                std::ptr::null_mut(),
                timeout.as_millis() as u32,
            )
        };
        check_res!(res);
        Ok(())
    }

    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<'a, NF, BladeRf2>> {
        unsafe {
            self.dev.set_sync_config::<NF>(config, layout.into())?;
        }

        Ok(TxSyncStream {
            dev: self.dev,
            layout,
            _format: PhantomData,
        })
    }

    pub fn enable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutTx::SISO(ch) => self.dev.set_enable_module(ch.into(), true),
            ChannelLayoutTx::MIMO => {
                self.dev.set_enable_module(Channel::Tx0, true)?;
                self.dev.set_enable_module(Channel::Tx1, true)?;
                Ok(())
            }
        }
    }

    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutTx::SISO(ch) => self.dev.set_enable_module(ch.into(), false),
            ChannelLayoutTx::MIMO => {
                self.dev.set_enable_module(Channel::Tx0, false)?;
                self.dev.set_enable_module(Channel::Tx1, false)?;
                Ok(())
            }
        }
    }
}

impl<'a, T: SampleFormat> TxSyncStream<'a, T, BladeRfAny> {
    pub fn write(&self, buffer: &[T], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_tx(
                self.dev.device,
                buffer.as_ptr() as *const _,
                buffer.len() as u32,
                std::ptr::null_mut(),
                timeout.as_millis() as u32,
            )
        };
        check_res!(res);
        Ok(())
    }

    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<'a, NF, BladeRfAny>> {
        unsafe {
            self.dev.set_sync_config::<NF>(config, layout.into())?;
        }

        Ok(TxSyncStream {
            dev: self.dev,
            layout,
            _format: PhantomData,
        })
    }

    pub fn enable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutTx::SISO(ch) => self.dev.set_enable_module(ch.into(), true),
            ChannelLayoutTx::MIMO => {
                self.dev.set_enable_module(Channel::Tx0, true)?;
                self.dev.set_enable_module(Channel::Tx1, true)?;
                Ok(())
            }
        }
    }

    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutTx::SISO(ch) => self.dev.set_enable_module(ch.into(), false),
            ChannelLayoutTx::MIMO => {
                self.dev.set_enable_module(Channel::Tx0, false)?;
                self.dev.set_enable_module(Channel::Tx1, false)?;
                Ok(())
            }
        }
    }
}

impl<T: SampleFormat, D: BladeRF> Drop for TxSyncStream<'_, T, D> {
    fn drop(&mut self) {
        // Ignore the results, just try disable both channels even if they don't exist on the dev.
        let _ = self.dev.set_enable_module(Channel::Tx0, false);
        let _ = self.dev.set_enable_module(Channel::Tx1, false);
    }
}
