use crate::BladeRf1;

// J7_1   = 10
// J7_2   = 11
// J7_5   = 08
// J7_6   = 09
// J13_1  = 17
// J13_2  = 18
// J16_1  = 31
// J16_2  = 32
// J16_3  = 19
// J16_4  = 20
// J16_5  = 21
// J16_6  = 24
use super::xb_gpio::{self, Disabled, XbGpioPin};
pub struct Xb200Pins<'a> {
    pub j7_1: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j7_2: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j7_5: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j7_6: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j13_1: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j13_2: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j16_1: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j16_2: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j16_3: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j16_4: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j16_5: XbGpioPin<'a, Disabled, BladeRf1>,
    pub j16_6: XbGpioPin<'a, Disabled, BladeRf1>,
}

impl Xb200Pins<'_> {
    pub(crate) fn new(dev: &BladeRf1) -> Xb200Pins {
        Xb200Pins {
            j7_1: XbGpioPin::<Disabled, BladeRf1>::new(10, dev),
            j7_2: XbGpioPin::<Disabled, BladeRf1>::new(11, dev),
            j7_5: XbGpioPin::<Disabled, BladeRf1>::new(8, dev),
            j7_6: XbGpioPin::<Disabled, BladeRf1>::new(9, dev),
            j13_1: XbGpioPin::<Disabled, BladeRf1>::new(17, dev),
            j13_2: XbGpioPin::<Disabled, BladeRf1>::new(18, dev),
            j16_1: XbGpioPin::<Disabled, BladeRf1>::new(31, dev),
            j16_2: XbGpioPin::<Disabled, BladeRf1>::new(32, dev),
            j16_3: XbGpioPin::<Disabled, BladeRf1>::new(19, dev),
            j16_4: XbGpioPin::<Disabled, BladeRf1>::new(20, dev),
            j16_5: XbGpioPin::<Disabled, BladeRf1>::new(21, dev),
            j16_6: XbGpioPin::<Disabled, BladeRf1>::new(24, dev),
        }
    }
}
