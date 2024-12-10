use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum RxMux {
    Invalid = bladerf_rx_mux_BLADERF_RX_MUX_INVALID,
    Baseband = bladerf_rx_mux_BLADERF_RX_MUX_BASEBAND,
    Counter12bit = bladerf_rx_mux_BLADERF_RX_MUX_12BIT_COUNTER,
    Counter32bit = bladerf_rx_mux_BLADERF_RX_MUX_32BIT_COUNTER,
    DigitalLoopback = bladerf_rx_mux_BLADERF_RX_MUX_DIGITAL_LOOPBACK,
}

impl TryFrom<bladerf_rx_mux> for RxMux {
    type Error = Error;

    fn try_from(value: bladerf_rx_mux) -> Result<Self> {
        Self::from_repr(value).ok_or_else(|| Error::msg(format!("Invalid RxMux value: {value}")))
    }
}
