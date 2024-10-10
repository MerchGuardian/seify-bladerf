use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Context;
use bladerf::{BladeRF, Channel, Format, GainMode, Loopback};
use eframe::egui::{self, Slider};
use eframe::App;
use egui_plot::{Line, Plot, PlotPoints};
use num_complex::Complex;

const DBFS_SAMPLE_RATE_HZ: f64 = 10.0;
// Duration in seconds for the sliding plot window
const PLOT_DURATION: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
struct Config {
    frequency_hz: u64,
    sample_rate_hz: u32,
    bandwidth_hz: u32,
    num_buffers: u32,
    buffer_size: u32,
    num_transfers: u32,
    timeout: Duration,
    amplitude: f64,
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
            amplitude: 0.5,
        }
    }
}

struct BladeRFApp {
    device: Arc<BladeRF>,
    config: Config,
    running: Arc<AtomicBool>,
    /// Stores (time (s), dBFS) pairs
    rx_power_data: Arc<Mutex<Vec<(f64, f64)>>>,
    tx_thread: Option<thread::JoinHandle<()>>,
    rx_thread: Option<thread::JoinHandle<()>>,
    rx_stats: Arc<Mutex<String>>,
    tx_stats: Arc<Mutex<String>>,
}

impl BladeRFApp {
    fn new(device: BladeRF) -> Self {
        Self {
            device: Arc::new(device),
            config: Config::default(),
            running: Arc::new(AtomicBool::new(false)),
            rx_power_data: Arc::new(Mutex::new(vec![])),
            tx_thread: None,
            rx_thread: None,
            rx_stats: Arc::new(Mutex::new(String::new())),
            tx_stats: Arc::new(Mutex::new(String::new())),
        }
    }

    fn start(&mut self) {
        let device_tx = Arc::clone(&self.device);
        let device_rx = Arc::clone(&self.device);
        let config_tx = self.config.clone();
        let config_rx = self.config.clone();
        let running_tx = Arc::clone(&self.running);
        let running_rx = Arc::clone(&self.running);
        let power_data = Arc::clone(&self.rx_power_data);
        let rx_stats = Arc::clone(&self.rx_stats);
        let tx_stats = Arc::clone(&self.tx_stats);

        if let Err(e) = setup(&self.device, &self.config) {
            println!("Setup failed: {e:?}");
            return;
        }

        self.running.store(true, Ordering::SeqCst);

        self.tx_thread = Some(thread::spawn(move || {
            tx_loop(device_tx, config_tx, running_tx, tx_stats);
        }));

        self.rx_thread = Some(thread::spawn(move || {
            rx_loop(device_rx, config_rx, running_rx, power_data, rx_stats);
        }));
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.rx_thread.take() {
            handle.join().unwrap();
        }
        if let Some(handle) = self.tx_thread.take() {
            handle.join().unwrap();
        }
    }
}

fn setup(device: &BladeRF, c: &Config) -> anyhow::Result<()> {
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

    c.write(device, Channel::Rx2)
        .context("Failed to write parameters for Rx2")?;

    println!("Setting device send configuration");

    device.sync_config(
        Channel::Tx1,
        Format::Sc16Q11,
        c.num_buffers,
        c.buffer_size,
        c.num_transfers,
        c.timeout,
    )?;

    c.write(device, Channel::Tx1)
        .context("Failed to write parameters for Tx1")?;

    device.enable_module(Channel::Tx1).unwrap();
    device.enable_module(Channel::Rx2).unwrap();

    Ok(())
}

impl Config {
    fn write(&self, device: &BladeRF, channel: Channel) -> anyhow::Result<()> {
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

fn tx_loop(
    device: Arc<BladeRF>,
    config: Config,
    running: Arc<AtomicBool>,
    tx_stats: Arc<Mutex<String>>,
) {
    let mut samples = vec![Complex::<i16>::ZERO; config.buffer_size as usize];
    let timeout = Duration::from_millis(250);

    while running.load(Ordering::SeqCst) {
        let sample_val = (config.amplitude * 2047.0) as i16;
        for sample in &mut samples {
            sample.re = sample_val;
            sample.im = -sample_val;
        }

        if let Err(e) = device.sync_tx(&samples, None, timeout) {
            println!("TX Error: {e:?}");
            break;
        }

        let mib = samples.len() as f64 * std::mem::size_of_val(&samples[0]) as f64 / 1_000_000.0;
        *tx_stats.lock().unwrap() = format!("TX: {mib:.1}MiB/s");
    }

    device.disable_module(Channel::Tx1).unwrap();
}

fn rx_loop(
    device: Arc<BladeRF>,
    config: Config,
    running: Arc<AtomicBool>,
    power_data: Arc<Mutex<Vec<(f64, f64)>>>,
    rx_stats: Arc<Mutex<String>>,
) {
    let mut samples = vec![Complex::<i16>::ZERO; config.buffer_size as usize];
    let mut stream_power = 0;
    let mut bytes = 0;
    let mut sample_count = 0;
    let timeout = Duration::from_millis(250);
    let mut last_sample_time = Instant::now();

    while running.load(Ordering::SeqCst) {
        if let Err(e) = device.sync_rx(&mut samples, None, timeout) {
            println!("RX Error: {e:?}");
            break;
        }

        sample_count += samples.len();
        bytes += samples.len() * std::mem::size_of_val(&samples[0]);
        for sample in &samples {
            stream_power += (sample.re as i64 * sample.re as i64) as u64;
            stream_power += (sample.im as i64 * sample.im as i64) as u64;
        }

        let elapsed = last_sample_time.elapsed();
        if elapsed.as_secs_f64() >= 1.0 / DBFS_SAMPLE_RATE_HZ {
            let full_scale_ratio = stream_power as f64 / (bytes * 2047 * 2047) as f64;
            let db_full_scale = 10.0 * full_scale_ratio.log10() + 3.0;
            let timestamp = last_sample_time.elapsed().as_secs_f64();

            {
                let mut data = power_data.lock().unwrap();
                data.push((timestamp, db_full_scale));

                while let Some(&(t, _)) = data.first() {
                    if timestamp - t > PLOT_DURATION.as_secs_f64() {
                        data.remove(0);
                    } else {
                        break;
                    }
                }
            }

            let mib = bytes as f64 / 1_000_000.0 / elapsed.as_secs_f64();
            *rx_stats.lock().unwrap() = format!(
                "RX: {mib:.1}MiB/s, {sample_count:.1}M samples, average power {db_full_scale:.1}dBfs"
            );

            last_sample_time = Instant::now();
            stream_power = 0;
            bytes = 0;
            sample_count = 0;
        }
    }

    device.disable_module(Channel::Rx2).unwrap();
}

impl App for BladeRFApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("BladeRF Control Panel");

            ui.collapsing("Configuration", |ui| {
                ui.add(
                    Slider::new(&mut self.config.frequency_hz, 850_000_000..=950_000_000)
                        .text("Frequency (Hz)"),
                );
                ui.add(
                    Slider::new(&mut self.config.sample_rate_hz, 1_000_000..=40_000_000)
                        .text("Sample Rate (Hz)"),
                );
                ui.add(
                    Slider::new(&mut self.config.bandwidth_hz, 1_000_000..=10_000_000)
                        .text("Bandwidth (Hz)"),
                );
                ui.add(Slider::new(&mut self.config.amplitude, 0.0..=1.0).text("Amplitude"));
            });

            if self.running.load(Ordering::SeqCst) {
                if ui.button("Stop").clicked() {
                    self.stop();
                }
            } else {
                if ui.button("Start").clicked() {
                    self.start();
                }
            }

            ui.label(format!("RX Stats: {}", self.rx_stats.lock().unwrap()));
            ui.label(format!("TX Stats: {}", self.tx_stats.lock().unwrap()));

            Plot::new("RX dBFS").view_aspect(2.0).show(ui, |plot_ui| {
                let power_data = self.rx_power_data.lock().unwrap();
                let points: PlotPoints = power_data.iter().map(|&(t, v)| [t, v]).collect();
                plot_ui.line(Line::new(points));
            });
        });
    }
}

fn main() -> anyhow::Result<()> {
    bladerf::set_log_level(bladerf::LogLevel::Info);
    bladerf::set_usb_reset_on_open(true);

    let device = bladerf::BladeRF::open_first().context("Failed to list BladeRF devices")?;
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "BladeRF Control Panel",
        options,
        Box::new(|_cc| Ok(Box::new(BladeRFApp::new(device)))),
    )
    .unwrap();

    Ok(())
}
