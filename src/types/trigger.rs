use strum::FromRepr;

use crate::{sys::*, Error, Result};

use super::Channel;

/// Trigger role
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TriggerRole {
    Invalid = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_INVALID,
    Disabled = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_DISABLED,
    Master = bladerf_trigger_role_BLADERF_TRIGGER_ROLE_MASTER,
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
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum TriggerSignal {
    Invalid = bladerf_trigger_signal_BLADERF_TRIGGER_INVALID,
    J71_4 = bladerf_trigger_signal_BLADERF_TRIGGER_J71_4,
    J51_1 = bladerf_trigger_signal_BLADERF_TRIGGER_J51_1,
    MiniExp1 = bladerf_trigger_signal_BLADERF_TRIGGER_MINI_EXP_1,
    User0 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_0,
    User1 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_1,
    User2 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_2,
    User3 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_3,
    User4 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_4,
    User5 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_5,
    User6 = bladerf_trigger_signal_BLADERF_TRIGGER_USER_6,
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
pub struct Trigger {
    pub channel: Channel,
    pub role: TriggerRole,
    pub signal: TriggerSignal,
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
