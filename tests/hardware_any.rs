#![cfg(feature = "hwtest_any")]

use std::{thread, time::Duration};

use bladerf::{
    BladeRF, BladeRfAny, ChannelLayoutRx, ComplexI12, ComplexI16, Error, Result, RxChannel,
    StreamConfig,
};
use serial_test::serial;

#[test]
#[serial]
fn list_devices() -> Result<()> {
    let devices = bladerf::get_device_list()?;
    println!("Discovered devices: {:?}", devices.len());
    Ok(())
}

#[test]
#[serial]
fn open_first_device() -> Result<()> {
    let _device = BladeRfAny::open_first()?;
    Ok(())
}

#[test]
#[serial]
fn open_with_devinfo() -> Result<()> {
    let devices = bladerf::get_device_list()?;
    let device = devices.first().ok_or(Error::Nodev)?;
    let _device = BladeRfAny::open_with_devinfo(device)?;
    Ok(())
}

#[test]
#[serial]
fn print_info() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    println!("{:#?}", device.info());
    Ok(())
}

#[test]
#[serial]
fn print_serial() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    println!("{:?}", device.get_serial());
    Ok(())
}

#[test]
#[serial]
fn is_fpga_configured() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let is_configured = device.is_fpga_configured()?;
    println!("FPGA Configured: {:?}", is_configured);
    Ok(())
}

#[test]
#[serial]
fn print_fpga_size() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let fpga_size = device.get_fpga_size()?;
    println!("FPGA Size: {:?}", fpga_size);
    Ok(())
}

#[test]
#[serial]
fn print_firmware_version() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let version = device.get_firmware_version()?;
    println!("{:?}", version);
    Ok(())
}

#[test]
#[serial]
fn print_fpga_version() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let version = device.get_fpga_version()?;
    println!("FPGA Version: {:?}", version);
    Ok(())
}

// TODO Provide way to select a sample rate from the list of supported rates gor a given device.
// and just use the higher level configure module function
#[test]
#[serial]
fn get_set_sample_rate() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let actual_rate = device.set_sample_rate(bladerf::Channel::Rx0, 1_000_000)?;
    let getter_rate = device.get_sample_rate(bladerf::Channel::Rx0)?;
    assert_eq!(actual_rate, getter_rate);

    let actual_rate = device.set_sample_rate(bladerf::Channel::Rx0, 2_000_000)?;
    let getter_rate = device.get_sample_rate(bladerf::Channel::Rx0)?;
    assert_eq!(actual_rate, getter_rate);
    Ok(())
}

#[test]
#[serial]
fn get_set_rx_mux() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let set_mux = bladerf::RxMux::DigitalLoopback;
    device.set_rx_mux(set_mux)?;
    let getter_mux = device.get_rx_mux()?;
    assert_eq!(set_mux, getter_mux);

    let set_mux = bladerf::RxMux::Baseband;
    device.set_rx_mux(set_mux)?;
    let getter_mux = device.get_rx_mux()?;
    assert_eq!(set_mux, getter_mux);
    Ok(())
}

// TODO Provide way to select bandwidths from the list of supported bws for a given device.
// and just use the higher level configure module function
#[test]
#[serial]
fn get_set_bandwidth() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let actual_bandwidth = device.set_bandwidth(bladerf::Channel::Rx0, 1_000_000)?;
    let getter_bandwidth = device.get_bandwidth(bladerf::Channel::Rx0)?;
    assert_eq!(actual_bandwidth, getter_bandwidth);

    let actual_bandwidth = device.set_bandwidth(bladerf::Channel::Rx0, 2_000_000)?;
    let getter_bandwidth = device.get_bandwidth(bladerf::Channel::Rx0)?;
    assert_eq!(actual_bandwidth, getter_bandwidth);
    Ok(())
}

// TODO Provide way to select frequencies from the list of supported frequencies for a given device.
// and just use the higher level configure module function
#[test]
#[serial]
fn get_set_frequency() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let set_frequency = 433_500_000;
    device.set_frequency(bladerf::Channel::Rx0, set_frequency)?;
    let getter_frequency = device.get_frequency(bladerf::Channel::Rx0)?;
    assert_eq!(set_frequency, getter_frequency);

    let set_frequency = 915_000_000;
    device.set_frequency(bladerf::Channel::Rx0, set_frequency)?;
    let getter_frequency = device.get_frequency(bladerf::Channel::Rx0)?;
    assert_eq!(set_frequency, getter_frequency);
    Ok(())
}

#[test]
#[serial]
fn get_set_loopback() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let set_loopback = bladerf::Loopback::Firmware;
    device.set_loopback(set_loopback)?;
    let getter_loopback = device.get_loopback()?;
    assert_eq!(set_loopback, getter_loopback);

    let set_loopback = bladerf::Loopback::None;
    device.set_loopback(set_loopback)?;
    let getter_loopback = device.get_loopback()?;
    assert_eq!(set_loopback, getter_loopback);
    Ok(())
}

// TODO Provide way to select gains from the list of supported gains for a given device.
// and just use the higher level configure module function
#[test]
#[serial]
fn get_set_gain() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let set_gain = 20;
    device.set_gain(bladerf::Channel::Rx0, set_gain)?;
    let getter_gain = device.get_gain(bladerf::Channel::Rx0)?;
    assert_eq!(set_gain, getter_gain);

    let set_gain = 40;
    device.set_gain(bladerf::Channel::Rx0, set_gain)?;
    let getter_gain = device.get_gain(bladerf::Channel::Rx0)?;
    assert_eq!(set_gain, getter_gain);
    Ok(())
}

#[test]
#[serial]
fn device_reset() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let old_serial = device.get_serial()?;
    device.device_reset()?;

    thread::sleep(Duration::from_secs(3));
    let new_device = BladeRfAny::open_first()?;
    let new_serial = new_device.get_serial()?;

    assert_eq!(old_serial, new_serial);
    Ok(())
}

#[test]
#[serial]
fn get_board_name() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let board_name = device.get_board_name();
    println!("{:?}", board_name);
    Ok(())
}

#[test]
#[serial]
fn rx_streamer_toggle_enabled() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let rx_streamer = device.rx_streamer::<ComplexI16>(
        StreamConfig::default(),
        ChannelLayoutRx::SISO(RxChannel::Rx0),
    )?;

    // Make sure that we can enable, disable and reenable again as well as read some samples.
    rx_streamer.enable()?;
    rx_streamer.disable()?;
    rx_streamer.enable()?;

    rx_streamer.read(&mut [ComplexI16::ZERO; 1024], Duration::from_secs(1))?;

    Ok(())
}

#[test]
#[serial]
fn rx_streamer_reconfigure() -> Result<()> {
    let device = BladeRfAny::open_first()?;
    let rx_streamer = device.rx_streamer::<ComplexI16>(
        StreamConfig::default(),
        ChannelLayoutRx::SISO(RxChannel::Rx0),
    )?;

    rx_streamer.enable()?;

    let new_rxstreamer = rx_streamer.reconfigure::<ComplexI12>(
        StreamConfig::default(),
        ChannelLayoutRx::SISO(RxChannel::Rx0),
    )?;

    new_rxstreamer.enable()?;
    new_rxstreamer.disable()?;

    Ok(())
}
