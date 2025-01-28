use std::time::Duration;

use crate::{brf_ci16_to_cf32, stream::RxSyncStream, BladeRF, BladeRfAny, SampleFormat};
use num_complex::Complex;
use num_traits::Zero;
use seify::RxStreamer;

impl<'a> RxStreamer for RxSyncStream<'a, Complex<i16>, BladeRfAny> {
    fn mtu(&self) -> Result<usize, seify::Error> {
        todo!()
    }

    fn activate_at(&mut self, time_ns: Option<i64>) -> Result<(), seify::Error> {
        if time_ns.is_none() {
            unsafe {
                self.dev
                    .set_enable_module(crate::Channel::Rx0, true)
                    .unwrap()
            };
        } else {
            todo!();
        };
        Ok(())
    }

    fn deactivate_at(&mut self, time_ns: Option<i64>) -> Result<(), seify::Error> {
        if time_ns.is_none() {
            unsafe {
                self.dev
                    .set_enable_module(crate::Channel::Rx0, false)
                    .unwrap()
            };
        } else {
            todo!();
        };
        Ok(())
    }

    fn read(
        &mut self,
        buffers: &mut [&mut [num_complex::Complex32]],
        timeout_us: i64,
    ) -> Result<usize, seify::Error> {
        let mut ci16_buffer = [Complex::zero(); 1024 * 8];
        // let mut bufs = [&mut ci16_buffer];
        Self::read(
            self,
            ci16_buffer.as_mut_slice(),
            Duration::from_millis(3500),
        )
        .map_err(|e| seify::Error::DeviceError)?;

        for (cf32_samp, ci16_samp) in buffers[0].iter_mut().zip(ci16_buffer.iter()) {
            *cf32_samp = brf_ci16_to_cf32(*ci16_samp);
        }
        Ok(1024 * 8)
    }

    fn activate(&mut self) -> Result<(), seify::Error> {
        self.activate_at(None)
    }

    fn deactivate(&mut self) -> Result<(), seify::Error> {
        self.deactivate_at(None)
    }
}
