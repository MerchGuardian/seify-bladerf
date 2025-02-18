use std::marker::PhantomData;

use crate::{BladeRF, Error, Result};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, PinState};
use libbladerf_sys as sys;

#[macro_export]
macro_rules! bladerf_gpio {
    ($struct_name:ident<$dev:ty>, $( $physical_name:ident = $pin_id:literal ),+) => {
        use $crate::expansion_boards::xb_gpio::{Disabled, XbGpioPin};

        pub struct $struct_name<'a> {
            $(pub $physical_name: XbGpioPin<'a, Disabled, $dev>,)+
        }

        impl $struct_name<'_> {
            pub(crate) fn new(dev: &$dev) -> $struct_name {
                $struct_name {
                    $($physical_name: XbGpioPin::<Disabled, $dev>::new($pin_id, dev),)+
                }
            }
        }
    };
}

pub struct Disabled;
pub struct Input;
pub struct Output;

const fn pin_to_bitmask(pin: u8) -> u32 {
    1 << (pin - 1)
}

pub struct XbGpioPin<'a, T, D: BladeRF> {
    pin: u8,
    device: &'a D,
    _direction: PhantomData<T>,
}

impl<'a, T, D: BladeRF> XbGpioPin<'a, T, D> {
    pub(crate) fn new(pin: u8, device: &'a D) -> XbGpioPin<'a, Disabled, D> {
        XbGpioPin {
            pin,
            device,
            _direction: PhantomData,
        }
    }
    pub fn into_input(self) -> Result<XbGpioPin<'a, Input, D>> {
        gpio_dir_masked_write(self.device, pin_to_bitmask(self.pin), 0)?;
        Ok(XbGpioPin {
            pin: self.pin,
            device: self.device,
            _direction: PhantomData,
        })
    }

    pub fn into_output(self) -> Result<XbGpioPin<'a, Output, D>> {
        gpio_dir_masked_write(self.device, pin_to_bitmask(self.pin), u32::MAX)?;
        Ok(XbGpioPin {
            pin: self.pin,
            device: self.device,
            _direction: PhantomData,
        })
    }
}

impl<D: BladeRF> XbGpioPin<'_, Input, D> {
    pub fn read(&self) -> Result<PinState> {
        let state_raw = gpio_read(self.device)?;
        if ((state_raw >> (self.pin - 1)) & 1) == 1 {
            Ok(PinState::High)
        } else {
            Ok(PinState::Low)
        }
    }
}

impl<D: BladeRF> XbGpioPin<'_, Output, D> {
    pub fn write(&self, state: PinState) -> Result<()> {
        let mask = pin_to_bitmask(self.pin);
        match state {
            PinState::High => gpio_masked_write(self.device, mask, u32::MAX),
            PinState::Low => gpio_masked_write(self.device, mask, 0),
        }
    }
}

impl<T, D: BladeRF> ErrorType for XbGpioPin<'_, T, D> {
    type Error = Error;
}

impl<D: BladeRF> InputPin for XbGpioPin<'_, Input, D> {
    fn is_high(&mut self) -> std::result::Result<bool, Self::Error> {
        match self.read()? {
            PinState::High => Ok(true),
            PinState::Low => Ok(false),
        }
    }

    fn is_low(&mut self) -> std::result::Result<bool, Self::Error> {
        match self.read()? {
            PinState::High => Ok(false),
            PinState::Low => Ok(true),
        }
    }
}

impl<D: BladeRF> OutputPin for XbGpioPin<'_, Output, D> {
    fn set_low(&mut self) -> std::result::Result<(), Self::Error> {
        self.write(PinState::Low)
    }

    fn set_high(&mut self) -> std::result::Result<(), Self::Error> {
        self.write(PinState::High)
    }
}

fn gpio_read<D: BladeRF>(dev: &D) -> Result<u32> {
    let mut val = 0;
    let result = unsafe { sys::bladerf_expansion_gpio_read(dev.get_device_ptr(), &mut val) };
    check_res!(result);
    Ok(val)
}

fn _gpio_write<D: BladeRF>(dev: &D, val: u32) -> Result<()> {
    let result = unsafe { sys::bladerf_expansion_gpio_write(dev.get_device_ptr(), val) };
    check_res!(result);
    Ok(())
}

fn gpio_masked_write<D: BladeRF>(dev: &D, mask: u32, value: u32) -> Result<()> {
    let result =
        unsafe { sys::bladerf_expansion_gpio_masked_write(dev.get_device_ptr(), mask, value) };
    check_res!(result);
    Ok(())
}

fn _gpio_dir_read<D: BladeRF>(dev: &D) -> Result<u32> {
    let mut dir = 0;
    let result = unsafe { sys::bladerf_expansion_gpio_dir_read(dev.get_device_ptr(), &mut dir) };
    check_res!(result);
    Ok(dir)
}

fn _gpio_dir_write<D: BladeRF>(dev: &D, outputs: u32) -> Result<()> {
    let result = unsafe { sys::bladerf_expansion_gpio_dir_write(dev.get_device_ptr(), outputs) };
    check_res!(result);
    Ok(())
}

fn gpio_dir_masked_write<D: BladeRF>(dev: &D, mask: u32, outputs: u32) -> Result<()> {
    let result = unsafe {
        sys::bladerf_expansion_gpio_dir_masked_write(dev.get_device_ptr(), mask, outputs)
    };
    check_res!(result);
    Ok(())
}
