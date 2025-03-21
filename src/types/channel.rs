use enum_map::Enum;
use strum::FromRepr;

use crate::{sys::*, Error, Result};

/// Represents the channels that can be used and configured on the BladeRF
///
/// You can convert to this type from [TxChannel] or [RxChannel] using [From] and [Into].
///
/// # Example
/// ```no_run
/// # use bladerf::{RxChannel, Channel};
/// // Creating directly.
/// let channel = Channel::Rx1;
///
/// // Converting from an RxChannel
/// let rx_channel = RxChannel::Rx0;
/// let channel: Channel = rx_channel.into();
/// ```
///
/// Determined from the `libbladerf` channel macros.
/// Relevant `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___c_h_a_n_n_e_l.html#ga832d79e0f128448d2258bd11a39bd45d>
#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
#[repr(i32)]
#[allow(missing_docs)]
pub enum Channel {
    Rx0 = 0,
    Rx1 = 2,
    Tx0 = 1,
    Tx1 = 3,
}

impl Channel {
    /// Checks if the given [Channel] is a receive channel.
    pub fn is_rx(&self) -> bool {
        matches!(self, Channel::Rx0 | Channel::Rx1)
    }

    /// Checks if the given [Channel] is a transmit channel.
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

/// Represents the receive channels that can be used on configured on the BladeRF
///
/// This type exists to better enforce the variants that are needed for certain configuration.
#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum RxChannel {
    Rx0,
    Rx1,
}

impl From<RxChannel> for Channel {
    fn from(value: RxChannel) -> Self {
        match value {
            RxChannel::Rx0 => Channel::Rx0,
            RxChannel::Rx1 => Channel::Rx1,
        }
    }
}

/// Represents the transmit channels that can be used on configured on the BladeRF
///
/// This type exists to better enforce the variants that are needed for certain configuration.
#[derive(Copy, Clone, Debug, Enum, FromRepr, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum TxChannel {
    Tx0,
    Tx1,
}

impl From<TxChannel> for Channel {
    fn from(value: TxChannel) -> Self {
        match value {
            TxChannel::Tx0 => Channel::Tx0,
            TxChannel::Tx1 => Channel::Tx1,
        }
    }
}
