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
use crate::ChannelLayoutRx;
use crate::ChannelLayoutTx;
use crate::Error;
use crate::Result;
use crate::SampleFormat;

pub struct SyncConfig {
    pub(crate) num_buffers: u32,
    pub(crate) buffer_size: u32,
    pub(crate) num_transfers: u32,
    pub(crate) stream_timeout: u32,
}

impl SyncConfig {
    pub fn new(
        num_buffers: u32,
        buffer_size: usize,
        num_transfers: u32,
        stream_timeout: Duration,
    ) -> Result<Self> {
        let stream_timeout = stream_timeout.as_millis().try_into().map_err(|_| {
            Error::msg(format!(
                "Stream timeout to large for u32 millis: {}",
                stream_timeout.as_millis()
            ))
        })?;

        let buffer_size: u32 = buffer_size
            .try_into()
            .map_err(|e| Error::msg(format!("Buffer size too big: {e:?}")))?;

        if buffer_size % 1024 != 0 {
            Err(Error::msg("Buffer size must be a multiple of 1024"))
        } else if num_buffers <= num_transfers {
            Err(Error::msg(
                "Number of buffers must be greater than number of transfers",
            ))
        } else {
            Ok(Self {
                num_buffers,
                buffer_size,
                num_transfers,
                stream_timeout,
            })
        }
    }
}

impl Default for SyncConfig {
    /// Values taken from <https://www.nuand.com/libbladeRF-doc/v2.5.0/sync_no_meta.html>
    fn default() -> Self {
        Self {
            num_buffers: 16,
            buffer_size: 8192,
            num_transfers: 8,
            stream_timeout: 3500,
        }
    }
}

pub struct RxSyncStream<T: Borrow<D>, F: SampleFormat, D: BladeRF> {
    pub(crate) dev: T,
    pub(crate) layout: ChannelLayoutRx,
    pub(crate) _devtype: PhantomData<D>,
    pub(crate) _format: PhantomData<F>,
}

// RX Stream Brf1

impl<T: Borrow<BladeRf1> + Clone, F: SampleFormat> RxSyncStream<T, F, BladeRf1> {
    pub fn read(&self, buffer: &mut [F], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_rx(
                self.dev.borrow().device,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                std::ptr::null_mut(),
                timeout.as_millis() as u32,
            )
        };
        check_res!(res);
        Ok(())
    }

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
        unsafe {
            self.dev
                .set_sync_config::<NF>(config, ChannelLayout::RxSISO)?;
        }
        Ok(RxSyncStream {
            dev: self.dev,
            layout: self.layout,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRf1>, F, BladeRf1> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
    ) -> Result<RxSyncStream<Arc<BladeRf1>, NF, BladeRf1>> {
        unsafe {
            self.dev
                .as_ref()
                .set_sync_config::<NF>(config, ChannelLayout::RxSISO)?;
        }
        Ok(RxSyncStream {
            dev: self.dev.clone(),
            layout: self.layout,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T: Borrow<BladeRf2> + Clone, F: SampleFormat> RxSyncStream<T, F, BladeRf2> {
    pub fn read(&self, buffer: &mut [F], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_rx(
                self.dev.borrow().device,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                std::ptr::null_mut(),
                timeout.as_millis() as u32,
            )
        };
        check_res!(res);
        Ok(())
    }

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
        unsafe {
            self.dev.set_sync_config::<NF>(config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev: self.dev,
            layout: self.layout,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRf2>, F, BladeRf2> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<BladeRf2>, NF, BladeRf2>> {
        unsafe {
            self.dev
                .as_ref()
                .set_sync_config::<NF>(config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev: self.dev.clone(),
            layout: self.layout,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

// //////////////////////////////////////////////////////////////////////////////////

impl<T: Borrow<BladeRfAny> + Clone, F: SampleFormat> RxSyncStream<T, F, BladeRfAny> {
    pub fn read(&self, buffer: &mut [F], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_rx(
                self.dev.borrow().device,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                std::ptr::null_mut(),
                timeout.as_millis() as u32,
            )
        };
        check_res!(res);
        Ok(())
    }

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
        unsafe {
            self.dev.set_sync_config::<NF>(config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev: self.dev,
            layout: self.layout,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRfAny>, F, BladeRfAny> {
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: &SyncConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<BladeRfAny>, NF, BladeRfAny>> {
        unsafe {
            self.dev
                .as_ref()
                .set_sync_config::<NF>(config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev: self.dev.clone(),
            layout: self.layout,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

impl<T: Borrow<D>, F: SampleFormat, D: BladeRF> Drop for RxSyncStream<T, F, D> {
    fn drop(&mut self) {
        // Ignore the results, just try disable both channels even if they don't exist on the dev.
        let _ = self.dev.borrow().set_enable_module(Channel::Rx0, false);
        let _ = self.dev.borrow().set_enable_module(Channel::Rx1, false);
    }
}

// Tx Streamers

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
