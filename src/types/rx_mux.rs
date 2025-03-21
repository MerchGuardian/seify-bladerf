use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// RX Mux Modes
///
/// These describe where the samples are sourced from for the RX FIFOs in the FPGA.
/// This is typically done for testing purposes.
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___r_e_c_e_i_v_e___m_u_x.html#gae7706e9b73a8ba4e9d6eaa74018aa114>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum RxMux {
    /// An invalid rx mux mode selection
    Invalid = bladerf_rx_mux_BLADERF_RX_MUX_INVALID,
    /// The default mode of operation where baseband IQ samples are read in.
    Baseband = bladerf_rx_mux_BLADERF_RX_MUX_BASEBAND,
    /// The I and Q channels are 12 bit counters where the I channel counts up and the Q channel counts down.
    Counter12bit = bladerf_rx_mux_BLADERF_RX_MUX_12BIT_COUNTER,
    /// What would normally be an 32 bit IQ sample (i16 for I and i16 for Q) is instead a 32 bit counter
    Counter32bit = bladerf_rx_mux_BLADERF_RX_MUX_32BIT_COUNTER,
    /// The baseband TX IQ samples that the user writes are "Looped Back"
    DigitalLoopback = bladerf_rx_mux_BLADERF_RX_MUX_DIGITAL_LOOPBACK,
}

impl TryFrom<bladerf_rx_mux> for RxMux {
    type Error = Error;

    fn try_from(value: bladerf_rx_mux) -> Result<Self> {
        Self::from_repr(value).ok_or_else(|| Error::msg(format!("Invalid RxMux value: {value}")))
    }
}
