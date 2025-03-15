use crate::{bladerf_gpio, BladeRf1};

bladerf_gpio! {Xb200Pins<BladeRf1>,
    j7_1   = 10,
    j7_2   = 11,
    j7_5   = 8,
    j7_6   = 9,
    j13_1  = 17,
    j13_2  = 18,
    j16_1  = 31,
    j16_2  = 32,
    j16_3  = 19,
    j16_4  = 20,
    j16_5  = 21,
    j16_6  = 24
}
