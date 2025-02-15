use crate::{sys::*, BladeRF, Result};
use crate::{BladeRf1, Direction};

use super::xb200_gpio::Xb200Pins;
use super::{Xb200Filter, Xb200Path};

pub struct Xb200<'a> {
    pub(crate) device: &'a BladeRf1,
    pub(crate) periph_taken: bool,
}

impl Xb200<'_> {
    pub fn set_filterbank(&self, direction: Direction, filter: Xb200Filter) -> Result<()> {
        let res = unsafe {
            bladerf_xb200_set_filterbank(
                self.device.get_device_ptr(),
                direction as bladerf_channel,
                filter as bladerf_xb200_filter,
            )
        };

        check_res!(res);
        Ok(())
    }

    pub fn get_filterbank(&self, path: Direction) -> Result<Xb200Filter> {
        let mut filter = bladerf_xb200_filter_BLADERF_XB200_CUSTOM;
        let res = unsafe {
            bladerf_xb200_get_filterbank(
                self.device.get_device_ptr(),
                path as bladerf_channel,
                &mut filter,
            )
        };

        check_res!(res);
        filter.try_into()
    }

    pub fn set_path(&self, direction: Direction, path: Xb200Path) -> Result<()> {
        let res = unsafe {
            bladerf_xb200_set_path(
                self.device.get_device_ptr(),
                direction as bladerf_channel,
                path as bladerf_xb200_path,
            )
        };

        check_res!(res);
        Ok(())
    }

    pub fn get_path(&self, direction: Direction) -> Result<Xb200Path> {
        let mut path = bladerf_xb200_path_BLADERF_XB200_BYPASS;
        let res = unsafe {
            bladerf_xb200_get_path(
                self.device.get_device_ptr(),
                direction as bladerf_channel,
                &mut path,
            )
        };

        check_res!(res);
        path.try_into()
    }

    pub fn take_periph(&mut self) -> Option<Xb200Pins> {
        if self.periph_taken {
            None
        } else {
            self.periph_taken = true;
            Some(Xb200Pins::new(self.device))
        }
    }
}
