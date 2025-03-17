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
use crate::ChannelLayoutTx;
use crate::Result;
use crate::SampleFormat;
use crate::TxChannel;

use super::StreamConfig;

/// A synchronous stream from transmitting samples with the BladeRF
///
/// This can be configured with a few different sample formats depending on your use-case.
///
/// Obtained from a call to [BladeRfAny::tx_streamer()] as well as a similar method on other devices.
/// ```no_run
/// use bladerf::{BladeRfAny, ComplexI12, ChannelLayoutTx, TxChannel, SyncConfig};
/// let dev = BladeRfAny::open_first().unwrap();
/// let conf = SyncConfig::default();
/// let layout = ChannelLayoutTx::SISO(TxChannel::Tx0);
///
/// let tx_stream = dev.tx_streamer::<ComplexI12>(conf, layout).unwrap();
/// ```
///
/// If the sample format needs to be changed, a call to [TxSyncStream::reconfigure()] can be made:
/// ```no_run
/// use bladerf::{BladeRfAny, ComplexI12, ChannelLayoutTx, TxChannel, SyncConfig, ComplexI8};
/// let dev = BladeRfAny::open_first().unwrap();
/// let conf = SyncConfig::default();
/// let layout = ChannelLayoutTx::SISO(TxChannel::Tx0);
///
/// let tx_stream_a = dev.tx_streamer::<ComplexI12>(conf, layout).unwrap();
///
/// let tx_stream_b = tx_stream_a.reconfigure::<ComplexI8>(conf, layout).unwrap();
/// ```
///
/// The methods for an [TxSyncStream] are a bit different for [BladeRf1] as they won't take the layout parameter.
#[derive(Debug)]
pub struct TxSyncStream<T: Borrow<D>, F: SampleFormat, D: BladeRF> {
    pub(crate) dev: T,
    pub(crate) layout: ChannelLayoutTx,
    pub(crate) config: StreamConfig,
    pub(crate) _devtype: PhantomData<D>,
    pub(crate) _format: PhantomData<F>,
}

impl<T: Borrow<D>, F: SampleFormat, D: BladeRF> TxSyncStream<T, F, D> {
    /// Writes IQ samples from a buffer of [[SampleFormat]].
    ///
    /// This method will error if a call to [TxSyncStream::enable()] as not been made.
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_t_r_e_a_m_i_n_g___s_y_n_c.html#ga9717092f3390080ed70f6dfb874a1dea>
    pub fn write(&self, buffer: &[F], timeout: Duration) -> Result<()> {
        let res = unsafe {
            sys::bladerf_sync_tx(
                self.dev.borrow().get_device_ptr(),
                buffer.as_ptr() as *const _,
                buffer.len() as u32,
                std::ptr::null_mut(),
                timeout.as_millis() as u32,
            )
        };
        check_res!(res);
        Ok(())
    }

    /// # Safety
    /// Need to ensure multiple streamers are not configured since a reconfiguration of one can change the sample type leading to our of bounds memory accesses.
    pub(crate) unsafe fn new(
        dev: T,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<T, F, D>> {
        unsafe {
            dev.borrow().set_sync_config::<F>(&config, layout.into())?;
        }

        Ok(TxSyncStream {
            dev,
            layout,
            config,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

impl<'a, F: SampleFormat, D: BladeRF> TxSyncStream<&'a D, F, D> {
    fn reconfigure_inner<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<&'a D, NF, D>> {
        // Safety: the previous streamer is moved, and is dropped so we are save to construct a new one.
        unsafe { TxSyncStream::new(self.dev, config, layout) }
    }
}

impl<F: SampleFormat, D: BladeRF> TxSyncStream<Arc<D>, F, D> {
    fn reconfigure_inner<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<Arc<D>, NF, D>> {
        // Safety: the previous streamer is moved, and is dropped so we are save to construct a new one.
        unsafe { TxSyncStream::new(self.dev.clone(), config, layout) }
    }
}

impl<T: Borrow<D>, F: SampleFormat, D: BladeRF> Drop for TxSyncStream<T, F, D> {
    fn drop(&mut self) {
        // Ignore the results, just try disable both channels even if they don't exist on the dev.
        let _ = self.dev.borrow().set_enable_module(Channel::Tx0, false);
        let _ = self.dev.borrow().set_enable_module(Channel::Tx1, false);
    }
}

////////////////////////////////////////////////////////////////////////////////
// RX Stream Brf1

impl<T: Borrow<BladeRf1>, F: SampleFormat> TxSyncStream<T, F, BladeRf1> {
    /// Enables the stream (and the relevant hardware) so samples can be written.
    pub fn enable(&self) -> Result<()> {
        // Safety, should be find to do a reconfigure here, nothing changes about the config, we just need to do this because disable will uninitialize the config
        unsafe {
            self.dev
                .borrow()
                .set_sync_config::<F>(&self.config, self.layout.into())?;
        }
        self.dev.borrow().set_enable_module(Channel::Tx0, true)
    }

    /// Disables the stream (and the relevant hardware).
    pub fn disable(&self) -> Result<()> {
        self.dev.borrow().set_enable_module(Channel::Tx0, false)
    }
}

impl<'a, F: SampleFormat> TxSyncStream<&'a BladeRf1, F, BladeRf1> {
    /// Allows reconfiguring a stream to change either the [SyncConfig] or [SampleFormat]
    ///
    /// See the general [TxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
    ) -> Result<TxSyncStream<&'a BladeRf1, NF, BladeRf1>> {
        self.reconfigure_inner(config, ChannelLayoutTx::SISO(TxChannel::Tx0))
    }
}

impl<F: SampleFormat> TxSyncStream<Arc<BladeRf1>, F, BladeRf1> {
    /// Allows reconfiguring a stream to change either the [SyncConfig] or [SampleFormat]
    ///
    /// See the general [TxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
    ) -> Result<TxSyncStream<Arc<BladeRf1>, NF, BladeRf1>> {
        self.reconfigure_inner(config, ChannelLayoutTx::SISO(TxChannel::Tx0))
    }
}

////////////////////////////////////////////////////////////////////////////////
// RX Stream Brf2

impl<T: Borrow<BladeRf2> + Clone, F: SampleFormat> TxSyncStream<T, F, BladeRf2> {
    /// Enables the stream (and the relevant hardware) so samples can be written.
    pub fn enable(&self) -> Result<()> {
        // Safety, should be find to do a reconfigure here, nothing changes about the config, we just need to do this because disable will uninitialize the config
        unsafe {
            self.dev
                .borrow()
                .set_sync_config::<F>(&self.config, self.layout.into())?;
        }
        match self.layout {
            ChannelLayoutTx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), true),
            ChannelLayoutTx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Tx0, true)?;
                self.dev.borrow().set_enable_module(Channel::Tx1, true)?;
                Ok(())
            }
        }
    }

    /// Disables the stream (and the relevant hardware).
    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutTx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), false),
            ChannelLayoutTx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Tx0, false)?;
                self.dev.borrow().set_enable_module(Channel::Tx1, false)?;
                Ok(())
            }
        }
    }
}

impl<'a, F: SampleFormat> TxSyncStream<&'a BladeRf2, F, BladeRf2> {
    /// Allows reconfiguring a stream to change either the [SyncConfig]/[SampleFormat]/[ChannelLayoutTx]
    ///
    /// See the general [TxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<&'a BladeRf2, NF, BladeRf2>> {
        self.reconfigure_inner(config, layout)
    }
}

impl<F: SampleFormat> TxSyncStream<Arc<BladeRf2>, F, BladeRf2> {
    /// Allows reconfiguring a stream to change either the [SyncConfig]/[SampleFormat]/[ChannelLayoutTx]
    ///
    /// See the general [TxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<Arc<BladeRf2>, NF, BladeRf2>> {
        self.reconfigure_inner(config, layout)
    }
}

////////////////////////////////////////////////////////////////////////////////
// RX Stream BrfAny

impl<T: Borrow<BladeRfAny> + Clone, F: SampleFormat> TxSyncStream<T, F, BladeRfAny> {
    /// Enables the stream (and the relevant hardware) so samples can be written.
    pub fn enable(&self) -> Result<()> {
        // Safety, should be find to do a reconfigure here, nothing changes about the config, we just need to do this because disable will uninitialize the config
        unsafe {
            self.dev
                .borrow()
                .set_sync_config::<F>(&self.config, self.layout.into())?;
        }
        match self.layout {
            ChannelLayoutTx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), true),
            ChannelLayoutTx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Tx0, true)?;
                self.dev.borrow().set_enable_module(Channel::Tx1, true)?;
                Ok(())
            }
        }
    }

    /// Disables the stream (and the relevant hardware).
    pub fn disable(&self) -> Result<()> {
        match self.layout {
            ChannelLayoutTx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), false),
            ChannelLayoutTx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Tx0, false)?;
                self.dev.borrow().set_enable_module(Channel::Tx1, false)?;
                Ok(())
            }
        }
    }
}

impl<'a, F: SampleFormat> TxSyncStream<&'a BladeRfAny, F, BladeRfAny> {
    /// Allows reconfiguring a stream to change either the [SyncConfig]/[SampleFormat]/[ChannelLayoutTx]
    ///
    /// See the general [TxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<&'a BladeRfAny, NF, BladeRfAny>> {
        self.reconfigure_inner(config, layout)
    }
}

impl<F: SampleFormat> TxSyncStream<Arc<BladeRfAny>, F, BladeRfAny> {
    /// Allows reconfiguring a stream to change either the [SyncConfig]/[SampleFormat]/[ChannelLayoutTx]
    ///
    /// See the general [TxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutTx,
    ) -> Result<TxSyncStream<Arc<BladeRfAny>, NF, BladeRfAny>> {
        self.reconfigure_inner(config, layout)
    }
}
