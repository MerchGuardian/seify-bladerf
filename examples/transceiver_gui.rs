use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use bladerf::{Channel, Format, GainMode, Loopback};
use num_complex::Complex;

#[derive(Clone, Debug)]
struct Config {
    frequency_hz: u64,
    sample_rate_hz: u32,
    bandwidth_hz: u32,
    num_buffers: u32,
    buffer_size: u32,
    num_transfers: u32,
    timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency_hz: 915_000_000,
            sample_rate_hz: 20_000_000,
            bandwidth_hz: 5_000_000,
            num_buffers: 4,
            buffer_size: 64 * 1024,
            num_transfers: 2,
            timeout: Duration::from_secs(5),
        }
    }
}

fn rx(device: Arc<bladerf::BladeRF>, c: Config) -> anyhow::Result<()> {
    let timeout = Duration::from_millis(250);
    let mut last_print = Instant::now();
    let mut sample_count = 0;
    let mut bytes = 0;
    let mut stream_power = 0;

    let mut samples = vec![Complex::<i16>::ZERO; c.buffer_size as usize];
    loop {
        let mut meta = None;
        device
            .sync_rx(&mut samples, meta.as_mut(), timeout)
            .context("Receive samples")?;

        sample_count += samples.len();
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
                "RX: {mib:.1}MiB / s, {sample_count:.1}M samples, average power {db_full_scale:.1}dBfs",
            );

            let s: Vec<_> = samples.iter().take(8).map(|s| (s.re, s.im)).collect();
            println!("  {s:?}",);
            last_print += Duration::from_secs(1);
            stream_power = 0;
            bytes = 0;
            sample_count = 0;
        }
    }
}

fn tx(device: Arc<bladerf::BladeRF>, c: Config) -> anyhow::Result<()> {
    let timeout = Duration::from_millis(250);
    let mut last_print = Instant::now();
    let mut sample_count = 0;
    let mut bytes = 0;

    let mut amplitude = 0.00;
    let mut samples = vec![Complex::<i16>::ZERO; c.buffer_size as usize];
    loop {
        let mut meta = None;
        device
            .sync_tx(&samples, meta.as_mut(), timeout)
            .context("Receive samples")?;

        sample_count += samples.len();
        bytes += samples.len() * std::mem::size_of_val(&samples[0]);

        let now = Instant::now();
        let elapsed = now.saturating_duration_since(last_print);
        if elapsed.as_secs() >= 1 {
            let mib = bytes as f64 / 1_000_000.0 / elapsed.as_secs_f64();
            println!("TX: {mib:.1}MiB / s, {sample_count:.1}M samples",);

            last_print += Duration::from_secs(1);
            bytes = 0;
            sample_count = 0;

            amplitude += 0.05;
            amplitude %= 1.0;
            for sample in &mut samples {
                sample.re = (amplitude * 2047.0) as i16;
                sample.im = (-amplitude * 2047.0) as i16;
            }

            println!("");
            println!("    AMPLITUDE: {amplitude}");
            println!("");
        }
    }
}

impl Config {
    fn write(&self, device: &bladerf::BladeRF, channel: Channel) -> anyhow::Result<()> {
        println!("Writing parameters for {channel:?}");

        device.set_sample_rate(channel, self.sample_rate_hz)?;
        device.set_frequency(channel, self.frequency_hz)?;
        device.set_bandwidth(channel, self.bandwidth_hz)?;
        if channel.is_rx() {
            device.set_gain(channel, 0)?;
            device.set_gain_mode(channel, GainMode::Default)?;
        }

        Ok(())
    }
}

fn setup(device: &bladerf::BladeRF, c: &Config) -> anyhow::Result<()> {
    device
        .load_fpga_from_env()
        .context("Failed to load FPGA bitstream")?;

    let _ = device
        .set_rx_mux(bladerf::RxMux::Baseband)
        .map_err(|e| println!("failed to set rx mux to baseband: {e:?}"));
    device.set_loopback(Loopback::None)?;

    println!("Setting device receive configuration");

    device.sync_config(
        Channel::Rx2,
        Format::Sc16Q11,
        c.num_buffers,
        c.buffer_size,
        c.num_transfers,
        c.timeout,
    )?;

    c.write(&device, Channel::Rx2)
        .context("Failed to write parameters for Rx2")?;

    device.enable_module(Channel::Rx2)?;

    println!("Setting device send configuration");

    device.sync_config(
        Channel::Tx1,
        Format::Sc16Q11,
        c.num_buffers,
        c.buffer_size,
        c.num_transfers,
        c.timeout,
    )?;

    c.write(&device, Channel::Tx1)
        .context("Failed to write parameters for Rx2")?;

    device.enable_module(Channel::Tx1)?;

    Ok(())
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

    let config = Config::default();

    setup(&device, &config).context("Failed to setup Blade RF")?;

    println!("Starting transmission");

    let device_rx = Arc::new(device);
    let device_tx = Arc::clone(&device_rx);

    let config_tx = config.clone();
    let config_rx = config;

    let receiver = std::thread::spawn(move || rx(device_rx, config_rx));
    std::thread::sleep(Duration::from_millis(500));
    let sender = std::thread::spawn(move || tx(device_tx, config_tx));

    receiver.join().unwrap().unwrap();
    sender.join().unwrap().unwrap();
    Ok(())
}
