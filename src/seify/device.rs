use num_complex::Complex;
use seify::DeviceTrait;

use crate::{BladeRF, BladeRfAny, RxSyncStream, TxSyncStream};

impl DeviceTrait for BladeRfAny {
    type RxStreamer = RxSyncStream<'_, Complex<i16>, BladeRfAny>;

    type TxStreamer = TxSyncStream<'_, Complex<i16>, BladeRfAny>;

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        todo!()
    }

    fn driver(&self) -> seify::Driver {
        todo!()
    }

    fn id(&self) -> Result<String, seify::Error> {
        self.get_serial().map_err(|e| e.into())
    }

    fn info(&self) -> Result<seify::Args, seify::Error> {
        todo!()
    }

    fn num_channels(&self, direction: seify::Direction) -> Result<usize, seify::Error> {
        todo!()
    }

    fn full_duplex(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<bool, seify::Error> {
        todo!()
    }

    fn rx_streamer(
        &self,
        channels: &[usize],
        args: seify::Args,
    ) -> Result<Self::RxStreamer, seify::Error> {
        todo!()
    }

    fn tx_streamer(
        &self,
        channels: &[usize],
        args: seify::Args,
    ) -> Result<Self::TxStreamer, seify::Error> {
        todo!()
    }

    fn antennas(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<Vec<String>, seify::Error> {
        todo!()
    }

    fn antenna(&self, direction: seify::Direction, channel: usize) -> Result<String, seify::Error> {
        todo!()
    }

    fn set_antenna(
        &self,
        direction: seify::Direction,
        channel: usize,
        name: &str,
    ) -> Result<(), seify::Error> {
        todo!()
    }

    fn supports_agc(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<bool, seify::Error> {
        todo!()
    }

    fn enable_agc(
        &self,
        direction: seify::Direction,
        channel: usize,
        agc: bool,
    ) -> Result<(), seify::Error> {
        todo!()
    }

    fn agc(&self, direction: seify::Direction, channel: usize) -> Result<bool, seify::Error> {
        todo!()
    }

    fn gain_elements(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<Vec<String>, seify::Error> {
        todo!()
    }

    fn set_gain(
        &self,
        direction: seify::Direction,
        channel: usize,
        gain: f64,
    ) -> Result<(), seify::Error> {
        todo!()
    }

    fn gain(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<Option<f64>, seify::Error> {
        todo!()
    }

    fn gain_range(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<seify::Range, seify::Error> {
        todo!()
    }

    fn set_gain_element(
        &self,
        direction: seify::Direction,
        channel: usize,
        name: &str,
        gain: f64,
    ) -> Result<(), seify::Error> {
        todo!()
    }

    fn gain_element(
        &self,
        direction: seify::Direction,
        channel: usize,
        name: &str,
    ) -> Result<Option<f64>, seify::Error> {
        todo!()
    }

    fn gain_element_range(
        &self,
        direction: seify::Direction,
        channel: usize,
        name: &str,
    ) -> Result<seify::Range, seify::Error> {
        todo!()
    }

    fn frequency_range(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<seify::Range, seify::Error> {
        todo!()
    }

    fn frequency(&self, direction: seify::Direction, channel: usize) -> Result<f64, seify::Error> {
        todo!()
    }

    fn set_frequency(
        &self,
        direction: seify::Direction,
        channel: usize,
        frequency: f64,
        args: seify::Args,
    ) -> Result<(), seify::Error> {
        todo!()
    }

    fn frequency_components(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<Vec<String>, seify::Error> {
        todo!()
    }

    fn component_frequency_range(
        &self,
        direction: seify::Direction,
        channel: usize,
        name: &str,
    ) -> Result<seify::Range, seify::Error> {
        todo!()
    }

    fn component_frequency(
        &self,
        direction: seify::Direction,
        channel: usize,
        name: &str,
    ) -> Result<f64, seify::Error> {
        todo!()
    }

    fn set_component_frequency(
        &self,
        direction: seify::Direction,
        channel: usize,
        name: &str,
        frequency: f64,
    ) -> Result<(), seify::Error> {
        todo!()
    }

    fn sample_rate(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<f64, seify::Error> {
        todo!()
    }

    fn set_sample_rate(
        &self,
        direction: seify::Direction,
        channel: usize,
        rate: f64,
    ) -> Result<(), seify::Error> {
        todo!()
    }

    fn get_sample_rate_range(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<seify::Range, seify::Error> {
        todo!()
    }

    fn bandwidth(&self, direction: seify::Direction, channel: usize) -> Result<f64, seify::Error> {
        todo!()
    }

    fn set_bandwidth(
        &self,
        direction: seify::Direction,
        channel: usize,
        bw: f64,
    ) -> Result<(), seify::Error> {
        todo!()
    }

    fn get_bandwidth_range(
        &self,
        direction: seify::Direction,
        channel: usize,
    ) -> Result<seify::Range, seify::Error> {
        todo!()
    }
}
