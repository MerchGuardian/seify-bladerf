use crate::{bladerf_drop, error::*, sys::*, types::*, BladeRF, BladeRfAny};
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

impl TryFrom<BladeRfAny> for BladeRf2 {
    type Error = Error;

    fn try_from(value: BladeRfAny) -> std::result::Result<Self, Self::Error> {
        if value.get_board_name() == "bladerf2" {
            let old_dev = ManuallyDrop::new(value);

            // Use `std::ptr::read` to move non-Copy fields out of the ManuallyDrop wrapper
            // SAFETY:
            // Being a rust reference, the following hold.
            // 1. each field is valid for reads
            // 2. each field is guaranteed to be aligned
            // 3. each field is properly initialized
            // Further
            // 4. Each field is read exactly once and then not dropped, therefore no double objects are created
            let enabled_modules = unsafe { std::ptr::read(&old_dev.enabled_modules) };
            let format_sync = unsafe { std::ptr::read(&old_dev.format_sync) };

            // let test = (*old_dev).enabled_modules;
            let new_dev = BladeRf2 {
                device: old_dev.device,
                enabled_modules,
                format_sync,
            };
            Ok(new_dev)
        } else {
            Err(Error::Unsupported)
        }
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
