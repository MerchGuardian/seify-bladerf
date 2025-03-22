use crate::{sys::*, BladeRF, Result};
use crate::{BladeRf1, Direction};

use super::xb_gpio_impls::Xb200Pins;
use super::{Xb200Filter, Xb200Path};

/// Structure to access functions related to the Xb200 Expansion board.
///
/// This struct can be obtained by a call to [BladeRf1::get_xb200()]
///
/// ```no_run
/// use bladerf::{BladeRf1, BladeRfAny};
/// let dev: BladeRf1 = BladeRfAny::open_first().unwrap().try_into().unwrap();
/// let xb200 = dev.get_xb200().unwrap();
/// ```
///
/// # Related Links on Nuand's Site
/// - [Product Page](https://www.nuand.com/product/hf-vhf-transverter/)
/// - [Getting Started Guide](https://github.com/Nuand/bladeRF/wiki/Getting-Started:-XB200-Transverter-Board)
/// - [RF Filterbank Performance](https://nuand.com/RF_filters.pdf)
pub struct Xb200<'a> {
    pub(crate) device: &'a BladeRf1,
    pub(crate) periph_taken: bool,
}

impl Xb200<'_> {
    /// Sets the filterbank to use on the given channel/direction: either [TX](Direction::TX) or [RX](Direction::RX)
    ///
    /// Relavent libbladerf docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___x_b.html#gabaa1f6ad3bf44503a217afea05f0273d>
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

    /// Gets the filterbank configured for use on the given channel/direction: either [TX](Direction::TX) or [RX](Direction::RX)
    ///
    /// Relevant libbladerf docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___x_b.html#gaba2e36d32776518c120d701b8336981b>
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

    /// Sets weather to [Mix](Xb200Path::Mix) or [Bypass](Xb200Path::Bypass) the XB200 board for the given channel/direction: either [TX](Direction::TX) or [RX](Direction::RX)
    ///
    /// Relevant libbladerf docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___x_b.html#ga8dcfc287d01256c476967fe4a964de25>
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

    /// Gets weather the given channel/direction is configured to [Mix](Xb200Path::Mix) or [Bypass](Xb200Path::Bypass) the signal.
    ///
    /// Relevant libbladerf docs: <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___x_b.html#gaa4948d0705c8b4de0b58cceff75f0194>
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

    /// Returns a struct to access the GPIO pins on the XB200
    ///
    /// Returns [None] if the pins have already been take.
    ///
    /// ```no_run
    /// use bladerf::{BladeRf1, BladeRfAny};
    /// let dev: BladeRf1 = BladeRfAny::open_first().unwrap().try_into().unwrap();
    /// let mut xb200 = dev.get_xb200().unwrap();
    /// let pins = xb200.take_periph().unwrap();
    /// let mut mypin = pins.j7_1.into_input().unwrap();
    ///
    /// // Method from this library
    /// let val = mypin.read();
    ///
    /// // Method from embedded hal
    /// use embedded_hal::digital::InputPin;
    /// let val2 = mypin.is_high();
    /// ```
    pub fn take_periph(&mut self) -> Option<Xb200Pins> {
        if self.periph_taken {
            None
        } else {
            self.periph_taken = true;
            Some(Xb200Pins::new(self.device))
        }
    }
}
