use strum::FromRepr;

use crate::{sys::*, Error, Result};

use super::Channel;

/// Represents the role of a device in a trigger chain
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html#gae0ce25426c2eba648a28fa07a3436acc>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TriggerRole {
    /// Invalid Role
    Invalid = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_INVALID,
    /// Triggering functionality is disabled on this device. Samples are not gated and the trigger signal is an input.
    Disabled = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_DISABLED,
    /// This device is the trigger master. Its trigger signal will be an output and this device will determine when all devices shall trigger.
    Master = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_MASTER,
    /// This device is the trigger slave. This device's trigger signal will be an input and this devices will wait for the master's trigger signal assertion.
    Slave = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_SLAVE,
}

impl TryFrom<bladerf_trigger_role> for TriggerRole {
    type Error = Error;

    fn try_from(value: bladerf_trigger_role) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid TriggerRole value: {value}")))
    }
}

/// Trigger signal selection
///
/// This selects pin or signal used for the trigger.
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___t_r_i_g.html#gaebcb881ab6a5f975aaabfd87586f248d>
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TriggerSignal {
    /// Invalid selection
    Invalid = bladerf_trigger_signal_BLADERF_TRIGGER_INVALID,
    /// J71 pin 4, mini_exp\[1] on x40/x115
    J71_4 = bladerf_trigger_signal_BLADERF_TRIGGER_J71_4,
    /// J51 pin 1, mini_exp\[1] on xA4/xA5/xA9
    J51_1 = bladerf_trigger_signal_BLADERF_TRIGGER_J51_1,
    /// mini_exp\[1], hardware-independent
    MiniExp1 = bladerf_trigger_signal_BLADERF_TRIGGER_MINI_EXP_1,
    /// Reserved for user SW/HW customizations
    User0 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_0,
    /// Reserved for user SW/HW customizations
    User1 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_1,
    /// Reserved for user SW/HW customizations
    User2 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_2,
    /// Reserved for user SW/HW customizations
    User3 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_3,
    /// Reserved for user SW/HW customizations
    User4 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_4,
    /// Reserved for user SW/HW customizations
    User5 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_5,
    /// Reserved for user SW/HW customizations
    User6 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_6,
    /// Reserved for user SW/HW customizations
    User7 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_7,
}

impl TryFrom<bladerf_trigger_signal> for TriggerSignal {
    type Error = Error;

    fn try_from(value: bladerf_trigger_signal) -> Result<Self> {
        Self::from_repr(value)
            .ok_or_else(|| Error::msg(format!("Invalid TriggerSignal value: {value}")))
    }
}

/// Trigger configuration
///
/// Related `libbladerf` docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/structbladerf__trigger.html>
pub struct Trigger {
    /// The channel associated with this trigger
    pub channel: Channel,
    /// The role of the device in this trigger chain
    pub role: TriggerRole,
    /// The pin/signal being used
    pub signal: TriggerSignal,
    /// Reserved for future use, should be set to zero.
    pub options: u64,
}

impl TryFrom<bladerf_trigger> for Trigger {
    type Error = Error;

    fn try_from(t: bladerf_trigger) -> Result<Self> {
        Ok(Self {
            channel: t.channel.try_into()?,
            role: t.role.try_into()?,
            signal: t.signal.try_into()?,
            options: t.options,
        })
    }
}
