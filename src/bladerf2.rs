use crate::{bladerf_drop, error::*, sys::*, types::*, BladeRF};
use crate::{HardwareVariant, Unknown};
use enum_map::EnumMap;
use ffi::{c_char, c_void, CStr, CString};
use log::warn;
use marker::PhantomData;
use mem::ManuallyDrop;
use parking_lot::lock_api::MutexGuard;
use parking_lot::Mutex;
use path::Path;
use std::*;
use sync::RwLock;
use time::Duration;

unsafe impl Send for BladeRf2 {}
unsafe impl Sync for BladeRf2 {}

pub struct BladeRf2 {
    device: *mut bladerf,
    enabled_modules: Mutex<EnumMap<Channel, bool>>,
    format_sync: RwLock<Option<Format>>,
}

impl BladeRf2 {
    pub fn get_bias_tee(&self, channel: Channel) -> Result<bool> {
        let mut enable = false;
        let res =
            unsafe { bladerf_get_bias_tee(self.device, channel as bladerf_channel, &mut enable) };
        check_res!(res);
        Ok(enable)
    }

    pub fn set_bias_tee(&self, channel: Channel, enable: bool) -> Result<()> {
        let res = unsafe { bladerf_set_bias_tee(self.device, channel as bladerf_channel, enable) };
        check_res!(res);
        Ok(())
    }
}

impl BladeRF for BladeRf2 {
    fn get_device_ptr(&self) -> *mut bladerf {
        self.device
    }

    fn get_enabled_modules(&self) -> MutexGuard<'_, parking_lot::RawMutex, EnumMap<Channel, bool>> {
        self.enabled_modules.lock()
    }

    // fn get_enabled_modules_mut(&mut self) -> &mut EnumMap<Channel, bool> {
    //     self.enabled_modules.get_mut()
    // }
}

impl Drop for BladeRf2 {
    fn drop(&mut self) {
        bladerf_drop(self);
    }
}
