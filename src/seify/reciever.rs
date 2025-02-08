use std::time::Duration;

use crate::{brf_ci16_to_cf32, stream::RxSyncStream, BladeRfAny};
use num_complex::Complex;
use num_traits::Zero;
use seify::RxStreamer;

impl RxStreamer for RxSyncStream<'_, Complex<i16>, BladeRfAny> {
    fn mtu(&self) -> Result<usize, seify::Error> {
        todo!()
    }

    fn activate_at(&mut self, time_ns: Option<i64>) -> Result<(), seify::Error> {
        if time_ns.is_none() {
            self.enable()?;
        } else {
            todo!();
        };
        Ok(())
    }

    fn deactivate_at(&mut self, time_ns: Option<i64>) -> Result<(), seify::Error> {
        if time_ns.is_none() {
            self.disable()?;
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
        let mut ci16_buffer = vec![Complex::zero(); 1024 * 8];
        Self::read(
            self,
            ci16_buffer.as_mut_slice(),
            Duration::from_micros(timeout_us as u64),
        )?;

        for (cf32_samp, ci16_samp) in buffers[0].iter_mut().zip(ci16_buffer.iter()) {
            *cf32_samp = brf_ci16_to_cf32(*ci16_samp);
        }
        Ok(ci16_buffer.len())
    }

    fn activate(&mut self) -> Result<(), seify::Error> {
        self.activate_at(None)
    }

    fn deactivate(&mut self) -> Result<(), seify::Error> {
        self.deactivate_at(None)
    }
}
