use std::{
    io::{stdout, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::Context;
use bladerf::{Channel, Format, GainMode, Loopback};
use crossterm::{
    cursor::{self},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use num_complex::Complex;
use parking_lot::Mutex;

// Configuration structure
#[derive(Clone, Debug)]
struct Config {
    frequency_hz: u64,
    sample_rate_hz: u32,
    bandwidth_hz: u32,
    num_buffers: u32,
    buffer_size: u32,
    num_transfers: u32,
    timeout: Duration,
    bit_rate: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency_hz: 915_000_000,
            sample_rate_hz: 2_000_000,
            bandwidth_hz: 500_000,
            num_buffers: 4,
            buffer_size: 64 * 1024,
            num_transfers: 2,
            timeout: Duration::from_secs(5),
            bit_rate: 100, // Default bit rate
        }
    }
}

// Constants for OOK modulation
static OOK_THRESHOLD: AtomicU64 = AtomicU64::new(300_000);
static RUNNING: AtomicBool = AtomicBool::new(true);

// UI State struct to hold all UI elements
struct UIState {
    text_output: String,
    hex_output: String,
    rx_stats: String,
    tx_stats: String,
    iq_samples: String,
    // Add any other UI elements you need
}

impl UIState {
    fn new() -> Self {
        Self {
            text_output: String::new(),
            hex_output: String::new(),
            rx_stats: String::new(),
            tx_stats: String::new(),
            iq_samples: String::new(),
        }
    }
}

fn rx(
    device: Arc<bladerf::BladeRF>,
    c: Config,
    ui_state: Arc<Mutex<UIState>>,
) -> anyhow::Result<()> {
    let timeout = Duration::from_millis(250);

    let samples_per_bit = c.sample_rate_hz / c.bit_rate;

    let mut samples = vec![Complex::<i16>::ZERO; c.buffer_size as usize];

    let mut last_print = Instant::now();
    let mut sample_count = 0;
    let mut bytes = 0;
    let mut stream_power = 0;

    let mut sample_buffer = Vec::new();
    let mut bits = Vec::new();

    while RUNNING.load(Ordering::Acquire) {
        let mut meta = None;
        device
            .sync_rx(&mut samples, meta.as_mut(), timeout)
            .context("Receive samples")?;

        // ===== Calculate power =====

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
            let rx_stats = format!(
                "RX: {mib:.1}MiB / s, {sample_count:.1}M samples, average power {db_full_scale:.1}dBfs",
            );

            let s: Vec<_> = samples.iter().take(8).map(|s| (s.re, s.im)).collect();
            let iq_samples = format!("  {s:?}");

            // Update UI state
            {
                let mut ui = ui_state.lock();
                ui.rx_stats = rx_stats;
                ui.iq_samples = iq_samples;
            }

            last_print = now;
            stream_power = 0;
            bytes = 0;
            sample_count = 0;
        }

        // ===== Decode OOK =====
        sample_buffer.extend_from_slice(&samples);

        // Process samples to extract bits
        while sample_buffer.len() >= samples_per_bit as usize {
            let bit_samples: Vec<Complex<i16>> =
                sample_buffer.drain(0..samples_per_bit as usize).collect();
            // Calculate average magnitude
            let mut sum_magnitude = 0u64;
            for s in &bit_samples {
                let magnitude = (s.re as i32).pow(2) + (s.im as i32).pow(2);
                sum_magnitude += magnitude as u64;
            }
            let avg_magnitude = sum_magnitude / bit_samples.len() as u64;

            let threshold = OOK_THRESHOLD.load(Ordering::Acquire);

            let bit = if avg_magnitude > threshold { 1 } else { 0 };
            bits.push(bit);

            // When we have 8 bits, convert to byte and update UI state
            if bits.len() >= 8 {
                let byte = bits_to_byte(&bits[0..8]);
                bits.drain(0..8);

                let mut ui = ui_state.lock();
                if byte.is_ascii() && !byte.is_ascii_control() {
                    ui.text_output.insert_str(0, &format!(" {} ", byte as char));
                } else {
                    ui.text_output.insert_str(0, "   ");
                }
                ui.hex_output.insert_str(0, &format!(" {:02X} ", byte));
                // Limit the length of the output strings
                let max_length = 80; // Adjust as needed
                if ui.text_output.len() > max_length {
                    ui.text_output.truncate(max_length);
                }
                if ui.hex_output.len() > max_length * 3 {
                    ui.hex_output.truncate(max_length * 3);
                }
            }
        }
    }

    Ok(())
}

// Helper function to convert 8 bits to a byte
fn bits_to_byte(bits: &[u8]) -> u8 {
    bits.iter()
        .enumerate()
        .fold(0u8, |acc, (i, &b)| acc | (b << (7 - i)))
}

fn tx(
    device: Arc<bladerf::BladeRF>,
    c: Config,
    ui_state: Arc<Mutex<UIState>>,
) -> anyhow::Result<()> {
    let timeout = Duration::from_millis(250);

    let samples_per_bit = c.sample_rate_hz / c.bit_rate;

    // Define preamble and sync word
    let preamble = [1, 0, 1, 0, 1, 0, 1, 0, // 0xAA
        1, 0, 1, 0, 1, 0, 1, 0];
    let sync_word = [0, 1, 0, 1, 0, 1, 0, 1];

    // Define the bitstream to send (e.g., "Hello")
    let message = b"Hello";
    let mut message_bits = Vec::new();
    for &byte in message.iter() {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            message_bits.push(bit);
        }
    }

    // Combine preamble, sync word, and message
    let bitstream = [/*preamble, sync_word,*/ message_bits].concat();
    let bitstream_len = bitstream.len();
    let mut bit_index = 0;

    let mut samples = vec![Complex::<i16>::ZERO; c.buffer_size as usize];

    let mut last_print = Instant::now();
    let mut sample_count = 0;
    let mut bytes = 0;

    while RUNNING.load(Ordering::Acquire) {
        // Fill the samples buffer according to the bitstream
        let mut sample_pos = 0;
        while sample_pos < samples.len() {
            let bit = bitstream[bit_index % bitstream_len];
            bit_index += 1;

            // For the current bit, generate samples_per_bit samples
            let end = std::cmp::min(sample_pos + samples_per_bit as usize, samples.len());
            let amplitude = if bit == 1 { 2047 } else { 0 };
            for s in &mut samples[sample_pos..end] {
                s.re = amplitude;
                s.im = 0;
            }
            sample_pos = end;
        }

        // Transmit the samples
        let mut meta = None;
        device
            .sync_tx(&samples, meta.as_mut(), timeout)
            .context("Transmit samples")?;

        sample_count += samples.len();
        bytes += samples.len() * std::mem::size_of_val(&samples[0]);

        let now = Instant::now();
        let elapsed = now.saturating_duration_since(last_print);
        if elapsed.as_secs() >= 1 {
            let mib = bytes as f64 / 1_000_000.0 / elapsed.as_secs_f64();
            let tx_stats = format!("TX: {mib:.1}MiB / s, {sample_count:.1}M samples");

            // Update UI state
            {
                let mut ui = ui_state.lock();
                ui.tx_stats = tx_stats;
            }

            last_print = now;
            bytes = 0;
            sample_count = 0;
        }
    }

    Ok(())
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

    c.write(device, Channel::Rx2)
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

    c.write(device, Channel::Tx1)
        .context("Failed to write parameters for Tx1")?;

    device.enable_module(Channel::Tx1)?;

    Ok(())
}

fn tui_app() -> anyhow::Result<()> {
    IS_RAW_MODE.store(true, Ordering::Release);
    // Setup terminal
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    std::panic::set_hook(Box::new(|info| {
        let (file, line) = {
            if let Some(location) = info.location() {
                (location.file(), location.line())
            } else {
                ("<unknown>", 0)
            }
        };

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        execute!(
            std::io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )
        .unwrap();
        terminal::disable_raw_mode().unwrap();
        println!("Panic: {msg}, at {file}:{line}");
        RUNNING.store(false, Ordering::Release);
    }));

    // Create UI state and share it across threads
    let ui_state = Arc::new(Mutex::new(UIState::new()));

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
    let _ = device
        .device_reset()
        .map_err(|e| println!("Failed to reset device: {e:?}"));

    let start = Instant::now();
    let device = 'outer: loop {
        for info in bladerf::get_device_list().unwrap_or_default() {
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
        thread::sleep(Duration::from_millis(50));
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

    println!("Device ready, starting tasks");

    let device_rx = Arc::new(device);
    let device_tx = Arc::clone(&device_rx);

    let config_tx = config.clone();
    let config_rx = config;

    let ui_state_rx = Arc::clone(&ui_state);
    let ui_state_tx = Arc::clone(&ui_state);
    let ui_state_input = Arc::clone(&ui_state);

    let receiver = thread::spawn(move || rx(device_rx, config_rx, ui_state_rx));
    let sender = thread::spawn(move || tx(device_tx, config_tx, ui_state_tx));

    let mut threshold_textbox = String::new();

    // Main loop to display UI
    while RUNNING.load(Ordering::Acquire) {
        if receiver.is_finished() || sender.is_finished() {
            RUNNING.store(false, Ordering::Release);
            break;
        }

        // Render
        {
            let ui = ui_state.lock();

            use std::io::{Write, stdout};
            use crossterm::{QueueableCommand, cursor};

            let mut stdout = stdout();

            stdout.queue(cursor::MoveTo(0, 0))?
            .queue(Clear(ClearType::All)).unwrap();

            let current_threshold = OOK_THRESHOLD.load(Ordering::Acquire);
            // Render UI elements
            let lines = [
                "Press 'q' to exit.".to_string(),
                format!("Current Threshold: {} - {threshold_textbox}", current_threshold),
                format!("TX Stats: {}", ui.tx_stats),
                format!("RX Stats: {}", ui.rx_stats),
                format!("IQ Samples: {}", ui.iq_samples),
                "\nReceived Text:".to_string(),
                ui.text_output.to_string(),
                "\nReceived Hex:".to_string(),
                ui.hex_output.to_string(),
            ];
            for (i, line) in lines.into_iter().enumerate() {
                stdout.queue(cursor::MoveTo(0, i as u16))?
                .queue(Print(line))?;
            }
            stdout.flush().unwrap();
        }

        // update
        loop {
            if !event::poll(Duration::from_millis(5)).unwrap() {
                break;
            }
            while let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char('q') => {
                        RUNNING.store(false, Ordering::Release);
                    }
                    KeyCode::Enter => {
                        if let Ok(threshold) = threshold_textbox.parse::<u64>() {
                            OOK_THRESHOLD.store(threshold, Ordering::Release);
                        }
                        threshold_textbox.clear();
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        threshold_textbox.push(c);
                    }
                    _ => {}
                }
            }
        }
    
    }
    RUNNING.store(false, Ordering::Release);

    // Wait for threads to finish
    receiver.join().unwrap()?;
    sender.join().unwrap()?;

    // Cleanup
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, cursor::Show)?;
    terminal::disable_raw_mode()?;
    IS_RAW_MODE.store(false, Ordering::Release);

    Ok(())
}

static IS_RAW_MODE: AtomicBool = AtomicBool::new(false);

pub fn main() -> anyhow::Result<()> {
    bladerf::set_log_level(bladerf::LogLevel::Info);
    bladerf::set_usb_reset_on_open(true);

    

    tui_app()
}

