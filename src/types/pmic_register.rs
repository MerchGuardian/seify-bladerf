use strum::FromRepr;

use crate::{sys::*, Error, Result};

#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq)]
#[repr(i32)]
pub enum PmicRegister {
    // Configuration register (uint16_t)
    // TODO: Allow returning u16 value
    // Shunt voltage (float)
    VoltageShunt = bladerf_pmic_register_BLADERF_PMIC_VOLTAGE_SHUNT as i32,
    /// Bus voltage (float)
    VoltageBus = bladerf_pmic_register_BLADERF_PMIC_VOLTAGE_BUS as i32,
    /// Load power (float)
    Power = bladerf_pmic_register_BLADERF_PMIC_POWER as i32,
    /// Load current (float)
    Current = bladerf_pmic_register_BLADERF_PMIC_CURRENT as i32,
    // Calibration (uint16_t)
    // TODO: Allow returning u16 value
    //bladerf_pmic_register_BLADERF_PMIC_CALIBRATION as i32,
}

impl std::fmt::Display for PmicRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PmicRegister::VoltageShunt => "Voltage (V, Shunt)",
            PmicRegister::VoltageBus => "Voltage (V, Bus)",
            PmicRegister::Power => "Power (W)",
            PmicRegister::Current => "Current (A)",
        };
        f.write_str(s)
    }
}

impl TryFrom<bladerf_pmic_register> for PmicRegister {
    type Error = Error;

    fn try_from(value: bladerf_pmic_register) -> Result<Self> {
        Self::from_repr(value as i32)
            .ok_or_else(|| Error::msg(format!("Invalid PmicRegister value: {value}")))
    }
}
