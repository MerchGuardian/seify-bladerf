use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use bladerf::{Channel, Format, GainMode, Loopback, Result};
use num_complex::Complex;

pub fn rx(device: &bladerf::BladeRF) -> anyhow::Result<()> {
    device
        .load_fpga_from_env()
        .context("Failed to load FPGA bitstream")?;

    let frequency_hz = 915_000_000;
    let sample_rate_hz = 20_000_000;
    let bandwidth_hz = 5_000_000;

    // TODO: Move this validation into the library
    let supported_freqs = device.get_frequency_range(Channel::Rx1).unwrap();
    let supported_sample_rates = device.get_sample_rate_range(Channel::Rx1).unwrap();
    let supported_bandwidths = device.get_bandwidth_range(Channel::Rx1).unwrap();
    assert!(
        supported_freqs.contains(frequency_hz),
        "{frequency_hz} not in {supported_freqs}"
    );
    assert!(
        supported_sample_rates.contains(sample_rate_hz),
        "{sample_rate_hz} not in {supported_sample_rates}"
    );
    assert!(
        supported_bandwidths.contains(bandwidth_hz),
        "{bandwidth_hz} not in {supported_bandwidths}"
    );

    let init_params = || -> Result<()> {
        device.set_frequency(Channel::Rx1, frequency_hz)?;

        // Fails here:
        // Maybe try to compile the same firmware as the host lib?
        device
            .set_sample_rate(Channel::Rx1, sample_rate_hz)
            .unwrap();
        device.set_bandwidth(Channel::Rx1, bandwidth_hz)?;
        // device.set_gain(Channel::Rx1, 0)?;
        // device.set_gain_mode(Channel::Rx1, GainMode::Default)?;

        // device.set_lna_gain(Channel::Rx1, 5_000_000)?;
        // device.set_rxg_gain1(Channel::Rx1, 5_000_000)?;
        // device.set_rxg_gain2(Channel::Rx1, 5_000_000)?;

        device.set_loopback(Loopback::None)?;
        Ok(())
    };

    println!("Setting device parameters");
    init_params().context("Failed to configure device parameters")?;

    let num_buffers = 4;
    let buffer_size = 64 * 1024;
    let num_transfers = 2;
    let timeout = Duration::from_secs(5);

    let set_config = || -> Result<()> {
        device.sync_config(
            Channel::Rx1,
            Format::Sc16Q11,
            num_buffers,
            buffer_size,
            num_transfers,
            timeout,
        )?;

        device.enable_module(Channel::Rx1)?;
        Ok(())
    };

    println!("Setting device configuration");
    set_config().context("Failed to set config")?;

    let mut samples = vec![Complex::<i16>::ZERO; buffer_size as usize];
    let mut meta = None;
    device
        .sync_rx(&mut samples, meta.as_mut(), timeout)
        .context("Receive samples")?;

    dbg!(meta);
    dbg!(&samples[..32]);

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    bladerf::set_log_level(bladerf::LogLevel::Debug);
    println!(
        "libbladerf version: {}",
        bladerf::version().context("Failed to obtain bladerf version")?
    );
    let device = bladerf::BladeRF::open_first().context("Failed to list BladeRF devices")?;
    let info = device.info().context("Failed to obtain device info")?;

    println!(
        "Device {}:\nSerial: {}\nManufacturer: {}\nProduct: {}\n",
        info.instance(),
        info.serial(),
        info.manufacturer(),
        info.product()
    );

    match rx(&device) {
        Ok(()) => Ok(()),
        Err(e) => {
            let dir = tempfile::TempDir::with_suffix("bladerf-fw-log")
                .expect("Failed to create tempfile");
            let mut path = PathBuf::from(dir.path());
            path.push("log.txt");
            if let Err(e) = device.get_fw_log(&path) {
                println!("Failed to download firmware log while responding to primary error. Error getting firmware log: {e:?}");
            } else {
                match std::fs::read_to_string(path) {
                    Ok(log) => {
                        if !log.is_empty() {
                            println!("Firmware log: \n{log}");
                        }
                    }
                    Err(e) => {
                        println!("Failed to read local firmware log while responding to primary error. Error getting firmware log: {e:?}");
                    }
                }
            }

            Err(e)
        }
    }
}
