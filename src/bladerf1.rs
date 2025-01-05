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

unsafe impl Send for BladeRf1 {}
unsafe impl Sync for BladeRf1 {}

impl BladeRf1 {
    pub fn set_txvga2(&self, gain: i32) -> Result<()> {
        let res = unsafe { bladerf_set_txvga2(self.device, gain) };

        check_res!(res);
        Ok(())
    }

    pub fn set_sampling(&self, sampling: Sampling) -> Result<()> {
        let res = unsafe { bladerf_set_sampling(self.device, sampling as bladerf_sampling) };
        check_res!(res);
        Ok(())
    }

    pub fn get_sampling(&self) -> Result<Sampling> {
        let mut sampling = bladerf_sampling_BLADERF_SAMPLING_UNKNOWN;
        let res = unsafe { bladerf_get_sampling(self.device, &mut sampling) };
        check_res!(res);
        Sampling::try_from(sampling)
    }

    pub fn set_lpf_mode(&self, channel: Channel, lpf_mode: LPFMode) -> Result<()> {
        let res = unsafe {
            bladerf_set_lpf_mode(
                self.device,
                channel as bladerf_channel,
                lpf_mode as bladerf_lpf_mode,
            )
        };
        check_res!(res);
        Ok(())
    }

    pub fn get_lpf_mode(&self, channel: Channel) -> Result<LPFMode> {
        let mut lpf_mode = bladerf_lpf_mode_BLADERF_LPF_NORMAL;
        let res =
            unsafe { bladerf_get_lpf_mode(self.device, channel as bladerf_channel, &mut lpf_mode) };
        check_res!(res);
        LPFMode::try_from(lpf_mode)
    }

    pub fn set_smb_mode(&self, mode: SmbMode) -> Result<()> {
        let res = unsafe { bladerf_set_smb_mode(self.device, mode as bladerf_smb_mode) };
        check_res!(res);
        Ok(())
    }

    pub fn get_smb_mode(&self) -> Result<SmbMode> {
        let mut mode = bladerf_smb_mode_BLADERF_SMB_MODE_INVALID;
        let res = unsafe { bladerf_get_smb_mode(self.device, &mut mode) };
        check_res!(res);
        SmbMode::try_from(mode)
    }

    pub fn set_rational_smb_frequency(&self, frequency: RationalRate) -> Result<RationalRate> {
        let mut actual_freq = bladerf_rational_rate {
            integer: 0,
            num: 0,
            den: 0,
        };
        // Despite frequency being passes as a &mut reference, the value is not actually mutated, so no need to pass it back to the user.
        let res = unsafe {
            bladerf_set_rational_smb_frequency(self.device, &mut frequency.into(), &mut actual_freq)
        };
        check_res!(res);
        Ok(actual_freq.into())
    }

    pub fn get_rational_smb_frequency(&self) -> Result<RationalRate> {
        let mut freq = bladerf_rational_rate {
            integer: 0,
            num: 0,
            den: 0,
        };
        let res = unsafe { bladerf_get_rational_smb_frequency(self.device, &mut freq) };
        check_res!(res);
        Ok(freq.into())
    }

    pub fn set_smb_frequency(&self, frequency: u32) -> Result<u32> {
        let mut actual_freq = 0;
        let res = unsafe { bladerf_set_smb_frequency(self.device, frequency, &mut actual_freq) };
        check_res!(res);
        Ok(actual_freq)
    }

    pub fn get_smb_frequency(&self) -> Result<u32> {
        let mut freq = 0;
        let res = unsafe { bladerf_get_smb_frequency(self.device, &mut freq) };
        check_res!(res);
        Ok(freq)
    }
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
