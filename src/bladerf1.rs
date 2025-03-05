use crate::expansion_boards::Xb200;
use crate::streamers::{RxSyncStream, SyncConfig, TxSyncStream};
use crate::{error::*, sys::*, types::*, BladeRF, BladeRfAny};
use mem::ManuallyDrop;
use std::*;
use sync::atomic::{AtomicBool, Ordering};

pub struct BladeRf1 {
    pub(crate) device: *mut bladerf,
    rx_stream_configured: AtomicBool,
    tx_stream_configured: AtomicBool,
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

    pub fn tx_streamer<T: SampleFormat>(
        &self,
        config: &SyncConfig,
    ) -> Result<TxSyncStream<&Self, T, BladeRf1>> {
        // TODO: Decide Ordering
        self.tx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an TX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { TxSyncStream::new(self, config, ChannelLayoutTx::SISO(TxChannel::Tx0)) }
    }

    pub fn rx_streamer<T: SampleFormat>(
        &self,
        config: SyncConfig,
    ) -> Result<RxSyncStream<&Self, T, BladeRf1>> {
        // TODO: Decide Ordering
        self.rx_stream_configured
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_err| {
                Error::Msg("Already have an RX stream open".to_owned().into_boxed_str())
            })?;

        // Safety: we check to make sure no other streamers are configured
        unsafe { RxSyncStream::new(self, config, ChannelLayoutRx::SISO(RxChannel::Rx0)) }
    }

    fn expansion_attach(&self, module: ExpansionModule) -> Result<()> {
        let res = unsafe { bladerf_expansion_attach(self.device, module as bladerf_xb) };
        check_res!(res);
        Ok(())
    }

    pub fn get_attached_expansion(&self) -> Result<ExpansionModule> {
        let mut module = bladerf_xb_BLADERF_XB_NONE;
        let res = unsafe { bladerf_expansion_get_attached(self.device, &mut module) };
        check_res!(res);
        ExpansionModule::try_from(module)
    }

    pub fn get_xb200(&self) -> Result<Xb200> {
        self.expansion_attach(ExpansionModule::Xb200)?;
        Ok(Xb200 {
            device: self,
            periph_taken: false,
        })
    }
}

impl TryFrom<BladeRfAny> for BladeRf1 {
    type Error = Error;

    fn try_from(value: BladeRfAny) -> std::result::Result<Self, Self::Error> {
        if value.get_board_name() == "bladerf1" {
            let old_dev = ManuallyDrop::new(value);

            let new_dev = BladeRf1 {
                device: old_dev.device,
                rx_stream_configured: AtomicBool::new(false),
                tx_stream_configured: AtomicBool::new(false),
            };

            Ok(new_dev)
        } else {
            Err(Error::Unsupported)
        }
    }
}

impl BladeRF for BladeRf1 {
    fn get_device_ptr(&self) -> *mut bladerf {
        self.device
    }
}

impl Drop for BladeRf1 {
    fn drop(&mut self) {
        unsafe { self.close() };
    }
}
