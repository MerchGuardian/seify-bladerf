// Allow clippy::unnecessary_cast since the cast is needed for when bindgen runs on windows. The enum variants get cast to i32 on windows.
#![allow(clippy::unnecessary_cast)]
use enum_map::Enum;
use strum::FromRepr;

use crate::{sys::*, Error, Result};

use super::{RxChannel, TxChannel};

#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
#[repr(u32)]
pub enum ChannelLayout {
    RxSISO = bladerf_channel_layout_BLADERF_RX_X1 as u32,
    RxMIMO = bladerf_channel_layout_BLADERF_RX_X2 as u32,
    TxSISO = bladerf_channel_layout_BLADERF_TX_X1 as u32,
    TxMIMO = bladerf_channel_layout_BLADERF_TX_X2 as u32,
}

impl ChannelLayout {
    pub fn is_rx(&self) -> bool {
        matches!(self, ChannelLayout::RxSISO | ChannelLayout::RxMIMO)
    }
    pub fn is_tx(&self) -> bool {
        matches!(self, ChannelLayout::TxSISO | ChannelLayout::TxMIMO)
    }
    pub fn is_siso(&self) -> bool {
        matches!(self, ChannelLayout::TxSISO | ChannelLayout::RxSISO)
    }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelLayoutRx {
    SISO(RxChannel),
    MIMO,
}

impl ChannelLayoutRx {
    pub fn is_mimo(&self) -> bool {
        matches!(self, Self::MIMO)
    }

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelLayoutTx {
    SISO(TxChannel),
    MIMO,
}

impl ChannelLayoutTx {
    pub fn is_mimo(&self) -> bool {
        matches!(self, Self::MIMO)
    }

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
