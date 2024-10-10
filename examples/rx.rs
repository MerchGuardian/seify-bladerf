use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

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
        let _ = device
            .set_sample_rate(Channel::Rx1, sample_rate_hz)
            .map_err(|e| println!("Failed to set sample rate: {e:?}"));
        device.set_bandwidth(Channel::Rx1, bandwidth_hz)?;
        device.set_gain(Channel::Rx1, 0)?;
        device.set_gain_mode(Channel::Rx1, GainMode::Default)?;

        let _ = device
            .set_rx_mux(bladerf::RxMux::Baseband)
            .map_err(|e| println!("failed to set rx mux to baseband: {e:?}"));

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

    let mut last_print = Instant::now();
    let mut bytes = 0;
    let mut stream_power = 0;

    let mut samples = vec![Complex::<i16>::ZERO; buffer_size as usize];
    loop {
        let mut meta = None;
        device
            .sync_rx(&mut samples, meta.as_mut(), timeout)
            .context("Receive samples")?;

        bytes += samples.len() * std::mem::size_of_val(&samples[0]);
        for sample in &samples {
            stream_power += (sample.re as i64 * sample.re as i64) as u64;
            stream_power += (sample.im as i64 * sample.im as i64) as u64;
        }

        let now = Instant::now();
        let elapsed = now.saturating_duration_since(last_print);
        if elapsed.as_secs() >= 1 {
            let full_scale_ratio = stream_power as f64 / (bytes * 2047 * 2047) as f64;
            let db_full_scale = 10.0 * full_scale_ratio.log10() + 3.0;

            let mib = bytes as f64 / 1_000_000.0 / elapsed.as_secs_f64();
            println!(
                "{mib:.1}MiB / s, {:.1}M samples, average power {db_full_scale:.1}dBfs",
                bytes as f64 / 2.0 / 1_000_000.0,
            );

            let s: Vec<_> = samples.iter().take(10).map(|s| (s.re, s.im)).collect();
            println!("  {s:?}",);
            last_print += Duration::from_secs(1);
            stream_power = 0;
            bytes = 0;
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    bladerf::set_log_level(bladerf::LogLevel::Info);
    bladerf::set_usb_reset_on_open(true);

    println!(
        "libbladerf version: {}",
        bladerf::version().context("Failed to obtain bladerf version")?
    );
    let device = bladerf::BladeRF::open_first().context("Failed to list BladeRF devices")?;
    println!("Found: {:?}", device.info().map(|i| i.serial()));
    let serial_number = device
        .get_serial()
        .context("Failed to get bladerf serial number")?;

    println!("Resetting device...");
    // Work around cleanup issue: If trying to re-open device without resetting / power cycling,
    // setting the sample rate fails and we get a strange error: Calibration TIMEOUT (0x16, 0x80)
    let _ = device
        .device_reset()
        .map_err(|e| println!("Failed to reset device: {e:?}"));

    let start = Instant::now();
    let device = 'outer: loop {
        for info in bladerf::get_device_list().unwrap_or(vec![]) {
            println!("Found: {:?}", info.serial());
            if info.serial() == serial_number {
                if let Ok(dev) = info.open() {
                    break 'outer dev;
                }
            }
        }
        if start.elapsed().as_secs() > 2 {
            anyhow::bail!("Failed to open device after two seconds");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    println!("Opened device");
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
