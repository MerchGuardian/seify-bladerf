use enum_map::Enum;
use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Channel {
    Rx1 = bladerf_channel_layout_BLADERF_RX_X1 as i32,
    Rx2 = bladerf_channel_layout_BLADERF_RX_X2 as i32,
    Tx1 = bladerf_channel_layout_BLADERF_TX_X1 as i32,
    Tx2 = bladerf_channel_layout_BLADERF_TX_X2 as i32,
}

impl Channel {
    pub fn is_rx(&self) -> bool {
        matches!(self, Channel::Rx1 | Channel::Rx2)
    }
    pub fn is_tx(&self) -> bool {
        matches!(self, Channel::Tx1 | Channel::Tx2)
    }
}

impl TryFrom<bladerf_channel> for Channel {
    type Error = Error;

    fn try_from(channel: bladerf_channel) -> Result<Self> {
        Self::from_repr(channel).ok_or_else(|| format!("Invalid bladerf channel: {channel}").into())
    }
}
