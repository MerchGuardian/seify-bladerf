// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use enum_map::Enum;
use strum::FromRepr;

use crate::{sys::*, Error, Result};

use super::{RxChannel, TxChannel};

/// Specifies the layout of the stream.
///
/// SISO - Single In / Single Out
/// MIMO - Multiple In / Multiple Out
///
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___c_h_a_n_n_e_l.html#ga832d79e0f128448d2258bd11a39bd45d>
#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum ChannelLayout {
    /// x1 RX (SISO)
    RxSISO = bladerf_channel_layout_BLADERF_RX_X1 as u32,
    /// x2 RX (MIMO)
    RxMIMO = bladerf_channel_layout_BLADERF_RX_X2 as u32,
    /// x1 TX (SISO)
    TxSISO = bladerf_channel_layout_BLADERF_TX_X1 as u32,
    /// x2 TX (MIMO)
    TxMIMO = bladerf_channel_layout_BLADERF_TX_X2 as u32,
}

impl ChannelLayout {
    /// Tests if the given layout is for the receive channel(s)
    pub fn is_rx(&self) -> bool {
        matches!(self, ChannelLayout::RxSISO | ChannelLayout::RxMIMO)
    }

    /// Tests if the given layout is for the transmit channel(s)
    pub fn is_tx(&self) -> bool {
        matches!(self, ChannelLayout::TxSISO | ChannelLayout::TxMIMO)
    }

    /// Tests if the given layout is Single In / Single Out
    pub fn is_siso(&self) -> bool {
        matches!(self, ChannelLayout::TxSISO | ChannelLayout::RxSISO)
    }

    /// Tests if the given layout is Multiple In / Multiple Out
    pub fn is_mimo(&self) -> bool {
        matches!(self, ChannelLayout::RxMIMO | ChannelLayout::TxMIMO)
    }
}

impl TryFrom<bladerf_channel_layout> for ChannelLayout {
    type Error = Error;

    fn try_from(channel: bladerf_channel_layout) -> Result<Self> {
        Self::from_repr(channel as u32)
            .ok_or_else(|| format!("Invalid bladerf channel: {channel}").into())
    }
}

/// A channel layour restricted to receive channels.
///
/// Allows specifying the rx channel used for SISO.
///
/// Can be converted into a [ChannelLayout]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelLayoutRx {
    /// A SISO channel layout for the given [RxChannel]
    SISO(RxChannel),
    /// A MIMO channel layout for the receive channels
    MIMO,
}

impl ChannelLayoutRx {
    /// Tests if the layout is a MIMO configuration
    pub fn is_mimo(&self) -> bool {
        matches!(self, Self::MIMO)
    }

    /// Tests if the layout is a SISO configuration
    pub fn is_siso(&self) -> bool {
        matches!(self, Self::SISO(_))
    }
}

impl From<ChannelLayoutRx> for ChannelLayout {
    fn from(value: ChannelLayoutRx) -> Self {
        match value {
            ChannelLayoutRx::SISO(_) => ChannelLayout::RxSISO,
            ChannelLayoutRx::MIMO => ChannelLayout::RxMIMO,
        }
    }
}

/// A channel layour restricted to transmit channels.
///
/// Allows specifying the tx channel used for SISO.
///
/// Can be converted into a [ChannelLayout]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelLayoutTx {
    /// A SISO channel layout for the given [TxChannel]
    SISO(TxChannel),
    /// A MIMO channel layout for the transmit channels
    MIMO,
}

impl ChannelLayoutTx {
    /// Tests if the layout is a MIMO configuration
    pub fn is_mimo(&self) -> bool {
        matches!(self, Self::MIMO)
    }

    /// Tests if the layout is a SISO configuration
    pub fn is_siso(&self) -> bool {
        matches!(self, Self::SISO(_))
    }
}

impl From<ChannelLayoutTx> for ChannelLayout {
    fn from(value: ChannelLayoutTx) -> Self {
        match value {
            ChannelLayoutTx::SISO(_) => ChannelLayout::TxSISO,
            ChannelLayoutTx::MIMO => ChannelLayout::TxMIMO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_conversions() {
        let rx_layout = ChannelLayoutRx::MIMO;
        let layout_rx_conv: ChannelLayout = rx_layout.into();
        assert_eq!(layout_rx_conv, ChannelLayout::RxMIMO);

        let tx_layout = ChannelLayoutTx::MIMO;
        let layout_tx_conv: ChannelLayout = tx_layout.into();
        assert_eq!(layout_tx_conv, ChannelLayout::TxMIMO);
    }
}
