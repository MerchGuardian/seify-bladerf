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

pub struct BladeRf1 {
    device: *mut bladerf,
    enabled_modules: Mutex<EnumMap<Channel, bool>>,
    format_sync: RwLock<Option<Format>>,
}

impl BladeRf1 {
    //     fn test(&mut self) {
    //         let mut en: &mut EnumMap<Channel, bool> = self.enabled_modules.get_mut();
    //     }
}

impl BladeRF for BladeRf1 {
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

impl Drop for BladeRf1 {
    fn drop(&mut self) {
        bladerf_drop(self);
    }
}
