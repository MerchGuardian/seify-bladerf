use std::marker::PhantomData;
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
// use crate::Format;
use crate::Result;
use crate::SampleFormat;

pub struct SyncConfig {
    // pub(crate) format: Format,
    pub(crate) num_buffers: u32,
    pub(crate) buffer_size: u32,
    pub(crate) num_transfers: u32,
    pub(crate) stream_timeout: u32,
}

impl SyncConfig {
    pub fn new(
        // format: Format,
        num_buffers: u32,
        buffer_size: u32,
        num_transfers: u32,
        stream_timeout: u32,
    ) -> Result<Self> {
        if buffer_size % 1024 != 0 {
            Err(Error::msg("Buffer size must be a multiple of 1024"))
        } else if num_buffers <= num_transfers {
            Err(Error::msg(
                "Number of buffers must be greater than number of transfers",
            ))
        } else {
            Ok(Self {
                // format,
                num_buffers,
                buffer_size,
                num_transfers,
                stream_timeout,
            })
        }
    }
}

pub struct RxSyncStream<'a, T: SampleFormat, D: BladeRF> {
    pub(crate) dev: &'a D,
    pub(crate) layout: ChannelLayoutRx,
    pub(crate) _format: PhantomData<T>,
}

// RX Stream Brf1

impl<'a, T: SampleFormat> RxSyncStream<'a, T, BladeRf1> {
    pub fn read(&self, buffer: &mut [T], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_rx(
                self.dev.device,
                buffer.as_mut_ptr() as *mut _,
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
    ) -> Result<RxSyncStream<'a, NF, BladeRf1>> {
        unsafe {
            self.dev
                .set_sync_config::<NF>(config, ChannelLayout::RxSISO)?;
        }
        Ok(RxSyncStream {
            dev: self.dev,
            layout: self.layout,
            _format: PhantomData,
        })
    }

    pub fn enable(&self) -> Result<()> {
        unsafe { self.dev.set_enable_module(Channel::Rx0, true) }
    }

    pub fn disable(&self) -> Result<()> {
        unsafe { self.dev.set_enable_module(Channel::Rx0, false) }
    }
}

impl<'a, T: SampleFormat> RxSyncStream<'a, T, BladeRf2> {
    pub fn read(&self, buffer: &mut [T], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_rx(
                self.dev.device,
                buffer.as_mut_ptr() as *mut _,
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
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<'a, NF, BladeRf2>> {
        unsafe {
            self.dev.set_sync_config::<NF>(config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev: self.dev,
            layout: self.layout,
            _format: PhantomData,
        })
    }

    pub fn enable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutRx::SISO(ch) => unsafe { self.dev.set_enable_module(ch.into(), true) },
            ChannelLayoutRx::MIMO => {
                unsafe { self.dev.set_enable_module(Channel::Rx0, true) }?;
                unsafe { self.dev.set_enable_module(Channel::Rx1, true) }?;
                Ok(())
            }
        }
    }

    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutRx::SISO(ch) => unsafe { self.dev.set_enable_module(ch.into(), false) },
            ChannelLayoutRx::MIMO => {
                unsafe { self.dev.set_enable_module(Channel::Rx0, false) }?;
                unsafe { self.dev.set_enable_module(Channel::Rx1, false) }?;
                Ok(())
            }
        }
    }
}

impl<'a, T: SampleFormat> RxSyncStream<'a, T, BladeRfAny> {
    pub fn read(&self, buffer: &mut [T], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_rx(
                self.dev.device,
                buffer.as_mut_ptr() as *mut _,
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
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<'a, NF, BladeRfAny>> {
        unsafe {
            self.dev.set_sync_config::<NF>(config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev: self.dev,
            layout: self.layout,
            _format: PhantomData,
        })
    }

    pub fn enable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutRx::SISO(ch) => unsafe { self.dev.set_enable_module(ch.into(), true) },
            ChannelLayoutRx::MIMO => {
                unsafe { self.dev.set_enable_module(Channel::Rx0, true) }?;
                unsafe { self.dev.set_enable_module(Channel::Rx1, true) }?;
                Ok(())
            }
        }
    }

    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutRx::SISO(ch) => unsafe { self.dev.set_enable_module(ch.into(), false) },
            ChannelLayoutRx::MIMO => {
                unsafe { self.dev.set_enable_module(Channel::Rx0, false) }?;
                unsafe { self.dev.set_enable_module(Channel::Rx1, false) }?;
                Ok(())
            }
        }
    }
}

impl<T: SampleFormat, D: BladeRF> Drop for RxSyncStream<'_, T, D> {
    fn drop(&mut self) {
        unsafe {
            // Ignore the results, just try disable both channels even if they don't exist on the dev.
            let _ = self.dev.set_enable_module(Channel::Rx0, false);
            let _ = self.dev.set_enable_module(Channel::Rx1, false);
        }
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
        unsafe { self.dev.set_enable_module(Channel::Tx0, true) }
    }

    pub fn disable(&self) -> Result<()> {
        unsafe { self.dev.set_enable_module(Channel::Tx0, false) }
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
            ChannelLayoutTx::SISO(ch) => unsafe { self.dev.set_enable_module(ch.into(), true) },
            ChannelLayoutTx::MIMO => {
                unsafe { self.dev.set_enable_module(Channel::Tx0, true) }?;
                unsafe { self.dev.set_enable_module(Channel::Tx1, true) }?;
                Ok(())
            }
        }
    }

    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutTx::SISO(ch) => unsafe { self.dev.set_enable_module(ch.into(), false) },
            ChannelLayoutTx::MIMO => {
                unsafe { self.dev.set_enable_module(Channel::Tx0, false) }?;
                unsafe { self.dev.set_enable_module(Channel::Tx1, false) }?;
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
            ChannelLayoutTx::SISO(ch) => unsafe { self.dev.set_enable_module(ch.into(), true) },
            ChannelLayoutTx::MIMO => {
                unsafe { self.dev.set_enable_module(Channel::Tx0, true) }?;
                unsafe { self.dev.set_enable_module(Channel::Tx1, true) }?;
                Ok(())
            }
        }
    }

    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutTx::SISO(ch) => unsafe { self.dev.set_enable_module(ch.into(), false) },
            ChannelLayoutTx::MIMO => {
                unsafe { self.dev.set_enable_module(Channel::Tx0, false) }?;
                unsafe { self.dev.set_enable_module(Channel::Tx1, false) }?;
                Ok(())
            }
        }
    }
}

impl<T: SampleFormat, D: BladeRF> Drop for TxSyncStream<'_, T, D> {
    fn drop(&mut self) {
        unsafe {
            // Ignore the results, just try disable both channels even if they don't exist on the dev.
            let _ = self.dev.set_enable_module(Channel::Tx0, false);
            let _ = self.dev.set_enable_module(Channel::Tx1, false);
        }
    }
}
