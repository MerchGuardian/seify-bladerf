use std::marker::PhantomData;
use std::time::Duration;

use libbladerf_sys as sys;

use crate::BladeRF;
use crate::BladeRf1;
use crate::Channel;
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
    pub(crate) dev: &'a BladeRf1,
    pub(crate) _format: PhantomData<T>,
    pub(crate) _device: PhantomData<D>,
}

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

    pub fn enable(&self) -> Result<()> {
        let res = unsafe { sys::bladerf_enable_module(self.dev.device, Channel::Rx0 as i32, true) };
        check_res!(res);
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        let res =
            unsafe { sys::bladerf_enable_module(self.dev.device, Channel::Rx0 as i32, false) };
        check_res!(res);
        Ok(())
    }
}

impl<'a, T: SampleFormat, D: BladeRF> Drop for RxSyncStream<'a, T, D> {
    fn drop(&mut self) {
        todo!("Do we need to disable the module here?");
    }
}

pub struct TxSyncStream<'a, T: SampleFormat, D: BladeRF> {
    pub(crate) dev: &'a BladeRf1,
    pub(crate) _format: PhantomData<T>,
    pub(crate) _device: PhantomData<D>,
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

    pub fn enable(&self) -> Result<()> {
        let res = unsafe { sys::bladerf_enable_module(self.dev.device, Channel::Tx0 as i32, true) };
        check_res!(res);
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        let res =
            unsafe { sys::bladerf_enable_module(self.dev.device, Channel::Tx0 as i32, false) };
        check_res!(res);
        Ok(())
    }
}

impl<'a, T: SampleFormat, D: BladeRF> Drop for TxSyncStream<'a, T, D> {
    fn drop(&mut self) {
        todo!("Do we need to disable the module here?");
    }
}
