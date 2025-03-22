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

use super::StreamConfig;

/// A synchronous stream from receiving samples from the BladeRF
///
/// This can be configured with a few different sample formats depending on your use-case.
///
/// Obtained from a call to [BladeRfAny::rx_streamer()] as well as a similar method on other devices.
/// ```no_run
/// use bladerf::{BladeRfAny, ComplexI12, ChannelLayoutRx, RxChannel, StreamConfig};
/// let dev = BladeRfAny::open_first().unwrap();
/// let conf = StreamConfig::default();
/// let layout = ChannelLayoutRx::SISO(RxChannel::Rx0);
///
/// let rx_stream = dev.rx_streamer::<ComplexI12>(conf, layout).unwrap();
/// ```
///
/// If the sample format needs to be changed, a call to [RxSyncStream::reconfigure()] can be made:
/// ```no_run
/// use bladerf::{BladeRfAny, ComplexI12, ChannelLayoutRx, RxChannel, StreamConfig, ComplexI8};
/// let dev = BladeRfAny::open_first().unwrap();
/// let conf = StreamConfig::default();
/// let layout = ChannelLayoutRx::SISO(RxChannel::Rx0);
///
/// let rx_stream_a = dev.rx_streamer::<ComplexI12>(conf, layout).unwrap();
///
/// let rx_stream_b = rx_stream_a.reconfigure::<ComplexI8>(conf, layout).unwrap();
/// ```
///
/// The methods for an [RxSyncStream] are a bit different for [BladeRf1] as they won't take the layout parameter.
#[derive(Debug)]
pub struct RxSyncStream<T: Borrow<D>, F: SampleFormat, D: BladeRF> {
    pub(crate) dev: T,
    pub(crate) layout: ChannelLayoutRx,
    pub(crate) config: StreamConfig,
    pub(crate) _devtype: PhantomData<D>,
    pub(crate) _format: PhantomData<F>,
}

impl<T: Borrow<D>, F: SampleFormat, D: BladeRF> RxSyncStream<T, F, D> {
    /// Reads IQ samples into a buffer of [[SampleFormat]].
    ///
    /// This method will error if a call to [RxSyncStream::enable()] as not been made.
    ///
    /// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_t_r_e_a_m_i_n_g___s_y_n_c.html#gacbe845827dd4ad717f3cbc812e66b204>
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

    /// # Safety
    /// Need to ensure multiple streamers are not configured since a reconfiguration of one can change the sample type leading to our of bounds memory accesses.
    pub(crate) unsafe fn new(
        dev: T,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<T, F, D>> {
        unsafe {
            dev.borrow().set_sync_config::<F>(&config, layout.into())?;
        }

        Ok(RxSyncStream {
            dev,
            layout,
            config,
            _devtype: PhantomData,
            _format: PhantomData,
        })
    }
}

impl<'a, F: SampleFormat, D: BladeRF> RxSyncStream<&'a D, F, D> {
    fn reconfigure_inner<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&'a D, NF, D>> {
        let dev = self.dev;
        // Drop needs to happen before constructing a new streamer since disabling voids the configuration and a new one need to be instatiated
        // Otherwise, a new RxSyncStream is created THEN the Drop trait is called calling disable and the stream immediately becomes invalid.
        drop(self);
        // Safety: the previous streamer is moved, and is dropped so we are save to construct a new one.
        unsafe { RxSyncStream::new(dev, config, layout) }
    }
}

impl<F: SampleFormat, D: BladeRF> RxSyncStream<Arc<D>, F, D> {
    fn reconfigure_inner<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<D>, NF, D>> {
        let dev = self.dev.clone();
        // Drop needs to happen before constructing a new streamer since disabling voids the configuration and a new one need to be instatiated
        // Otherwise, a new RxSyncStream is created THEN the Drop trait is called calling disable and the stream immediately becomes invalid.
        drop(self);
        // Safety: the previous streamer is moved, and is dropped so we are save to construct a new one.
        unsafe { RxSyncStream::new(dev, config, layout) }
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
    /// Enables the stream (and the relevant hardware) so samples can be read.
    pub fn enable(&self) -> Result<()> {
        // Safety, should be find to do a reconfigure here, nothing changes about the config, we just need to do this because disable will uninitialize the config
        unsafe {
            self.dev
                .borrow()
                .set_sync_config::<F>(&self.config, self.layout.into())?;
        }
        self.dev.borrow().set_enable_module(Channel::Rx0, true)
    }

    /// Disables the stream (and the relevant hardware).
    pub fn disable(&self) -> Result<()> {
        self.dev.borrow().set_enable_module(Channel::Rx0, false)
    }
}

impl<'a, F: SampleFormat> RxSyncStream<&'a BladeRf1, F, BladeRf1> {
    /// Allows reconfiguring a stream to change either the [StreamConfig] or [SampleFormat]
    ///
    /// See the general [RxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
    ) -> Result<RxSyncStream<&'a BladeRf1, NF, BladeRf1>> {
        self.reconfigure_inner(config, ChannelLayoutRx::SISO(RxChannel::Rx0))
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRf1>, F, BladeRf1> {
    /// Allows reconfiguring a stream to change either the [StreamConfig] or [SampleFormat]
    ///
    /// See the general [RxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
    ) -> Result<RxSyncStream<Arc<BladeRf1>, NF, BladeRf1>> {
        self.reconfigure_inner(config, ChannelLayoutRx::SISO(RxChannel::Rx0))
    }
}

////////////////////////////////////////////////////////////////////////////////
// RX Stream Brf2

impl<T: Borrow<BladeRf2> + Clone, F: SampleFormat> RxSyncStream<T, F, BladeRf2> {
    /// Enables the stream (and the relevant hardware) so samples can be read.
    pub fn enable(&self) -> Result<()> {
        // Safety, should be find to do a reconfigure here, nothing changes about the config, we just need to do this because disable will uninitialize the config
        unsafe {
            self.dev
                .borrow()
                .set_sync_config::<F>(&self.config, self.layout.into())?;
        }

        match self.layout {
            ChannelLayoutRx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), true),
            ChannelLayoutRx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Rx0, true)?;
                self.dev.borrow().set_enable_module(Channel::Rx1, true)?;
                Ok(())
            }
        }
    }

    /// Disables the stream (and the relevant hardware).
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
    /// Allows reconfiguring a stream to change either the [StreamConfig]/[SampleFormat]/[ChannelLayoutRx]
    ///
    /// See the general [RxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&'a BladeRf2, NF, BladeRf2>> {
        self.reconfigure_inner(config, layout)
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRf2>, F, BladeRf2> {
    /// Allows reconfiguring a stream to change either the [StreamConfig]/[SampleFormat]/[ChannelLayoutRx]
    ///
    /// See the general [RxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<BladeRf2>, NF, BladeRf2>> {
        self.reconfigure_inner(config, layout)
    }
}

////////////////////////////////////////////////////////////////////////////////
// RX Stream BrfAny

impl<T: Borrow<BladeRfAny> + Clone, F: SampleFormat> RxSyncStream<T, F, BladeRfAny> {
    /// Enables the stream (and the relevant hardware) so samples can be read.
    pub fn enable(&self) -> Result<()> {
        // Safety, should be find to do a reconfigure here, nothing changes about the config, we just need to do this because disable will uninitialize the config
        unsafe {
            self.dev
                .borrow()
                .set_sync_config::<F>(&self.config, self.layout.into())?;
        }
        match self.layout {
            ChannelLayoutRx::SISO(ch) => self.dev.borrow().set_enable_module(ch.into(), true),
            ChannelLayoutRx::MIMO => {
                self.dev.borrow().set_enable_module(Channel::Rx0, true)?;
                self.dev.borrow().set_enable_module(Channel::Rx1, true)?;
                Ok(())
            }
        }
    }

    /// Disables the stream (and the relevant hardware).
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
    /// Allows reconfiguring a stream to change either the [StreamConfig]/[SampleFormat]/[ChannelLayoutRx]
    ///
    /// See the general [RxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<&'a BladeRfAny, NF, BladeRfAny>> {
        self.reconfigure_inner(config, layout)
    }
}

impl<F: SampleFormat> RxSyncStream<Arc<BladeRfAny>, F, BladeRfAny> {
    /// Allows reconfiguring a stream to change either the [StreamConfig]/[SampleFormat]/[ChannelLayoutRx]
    ///
    /// See the general [RxSyncStream] docs for usage example.
    pub fn reconfigure<NF: SampleFormat>(
        self,
        config: StreamConfig,
        layout: ChannelLayoutRx,
    ) -> Result<RxSyncStream<Arc<BladeRfAny>, NF, BladeRfAny>> {
        self.reconfigure_inner(config, layout)
    }
}
