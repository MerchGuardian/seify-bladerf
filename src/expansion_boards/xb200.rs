use crate::{sys::*, BladeRF, Error, Result};
use crate::{BladeRf1, Direction};

use super::Xb200Filter;

pub struct Xb200<'a> {
    pub(crate) device: &'a BladeRf1,
}

impl Xb200<'_> {
    pub fn set_filterbank(&self, path: Direction, filter: Xb200Filter) -> Result<()> {
        let res = unsafe {
            bladerf_xb200_set_filterbank(
                self.device.get_device_ptr(),
                path as bladerf_channel,
                filter as bladerf_xb200_filter,
            )
        };

        check_res!(res);
        Ok(())
    }
}
