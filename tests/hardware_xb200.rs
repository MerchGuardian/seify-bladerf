#![cfg(feature = "hwtest_xb200")]

use std::{thread::sleep, time::Duration};

use bladerf::{
    expansion_boards::{Xb200Filter, Xb200Path},
    BladeRf1, BladeRfAny, Direction, Result,
};
use embedded_hal::digital::OutputPin;
use serial_test::serial;

#[test]
#[serial]
fn get_set_filterbank() -> Result<()> {
    let device: BladeRf1 = BladeRfAny::open_first()?.try_into()?;

    let xb200 = device.get_xb200()?;

    let filter_to_set = Xb200Filter::Custom;
    xb200.set_filterbank(Direction::RX, filter_to_set)?;
    let current_filter = xb200.get_filterbank(Direction::RX)?;
    assert_eq!(current_filter, filter_to_set);

    let filter_to_set = Xb200Filter::MHz50;
    xb200.set_filterbank(Direction::RX, filter_to_set)?;
    let current_filter = xb200.get_filterbank(Direction::RX)?;
    assert_eq!(current_filter, filter_to_set);

    Ok(())
}

#[test]
#[serial]
fn get_set_path() -> Result<()> {
    let device: BladeRf1 = BladeRfAny::open_first()?.try_into()?;

    let xb200 = device.get_xb200()?;

    let path_to_set = Xb200Path::Bypass;
    xb200.set_path(Direction::RX, path_to_set)?;
    let current_path = xb200.get_path(Direction::RX)?;
    assert_eq!(current_path, path_to_set);

    let path_to_set = Xb200Path::Bypass;
    xb200.set_path(Direction::RX, path_to_set)?;
    let current_path = xb200.get_path(Direction::RX)?;
    assert_eq!(current_path, path_to_set);

    Ok(())
}

#[test]
#[serial]
fn take_pins() -> Result<()> {
    let device: BladeRf1 = BladeRfAny::open_first()?.try_into()?;

    let mut xb200 = device.get_xb200()?;

    let pins = xb200.take_periph().unwrap();

    // J7_2 is the only pin that really seems to function. Some of the other seems to have odd behavior.
    let mut test_pin = pins.j7_2.into_output()?;

    test_pin.set_high()?;
    sleep(Duration::from_secs(2));
    test_pin.set_low()?;
    sleep(Duration::from_secs(2));

    Ok(())
}
