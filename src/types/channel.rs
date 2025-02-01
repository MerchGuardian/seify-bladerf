use enum_map::Enum;
use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Determined from the bladerf channel macros defined in
/// <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___c_h_a_n_n_e_l.html#ga832d79e0f128448d2258bd11a39bd45d>
#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum Channel {
    Rx0 = 0,
    Rx1 = 2,
    Tx0 = 1,
    Tx1 = 3,
}

impl Channel {
    pub fn is_rx(&self) -> bool {
        matches!(self, Channel::Rx0 | Channel::Rx1)
    }
    pub fn is_tx(&self) -> bool {
        matches!(self, Channel::Tx0 | Channel::Tx1)
    }
}

impl TryFrom<bladerf_channel> for Channel {
    type Error = Error;

    fn try_from(channel: bladerf_channel) -> Result<Self> {
        Self::from_repr(channel).ok_or_else(|| format!("Invalid bladerf channel: {channel}").into())
    }
}

#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
pub enum RxChannel {
    Rx0 = 0,
    Rx1 = 1,
}

impl From<RxChannel> for Channel {
    fn from(value: RxChannel) -> Self {
        match value {
            RxChannel::Rx0 => Channel::Rx0,
            RxChannel::Rx1 => Channel::Rx1,
        }
    }
}

#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
pub enum TxChannel {
    Tx0 = 0,
    Tx1 = 1,
}
