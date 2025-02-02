use anyhow::Ok;
use bladerf::{
    BladeRF, BladeRf2, BladeRfAny, Channel, ChannelLayoutRx, ChannelLayoutTx, GainMode,
    PmicRegister, SyncConfig,
};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use num_complex::Complex;
use std::{
    fs,
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};

use bs58;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Directory to store CSV results. Must not exist or be empty.
    #[clap(short, long, default_value = "results")]
    output_dir: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct HyperParameters {
    sample_rate: u32,
    sample_period: Duration,
    num_samples: usize,
    timeout: Duration,
    num_buffers: u32,
    num_transfers: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Parameters {
    frequency: u64,
    channel_set: Vec<Channel>,
    rx_gain: i32,
    tx_gain: i32,
    external_bias_tee: bool,
    external_lna: bool,
}

#[derive(Serialize)]
struct Measurement {
    /// seconds since Unix epoch.
    timestamp: f64,
    temperature: f32,
    voltage_bus: f32,
    voltage_shunt: f32,
    power: f32,
    current: f32,
}

/// Performs a measurement run for the given configuration, updating the provided global
/// progress bar with the elapsed time
fn perform_sampling(
    dev: &mut BladeRf2,
    hyper: &HyperParameters,
    params: &Parameters,
    global_pb: &ProgressBar,
) -> anyhow::Result<Vec<Measurement>> {
    // Set frequency and sample rate for each channel using hyper parameters.
    for channel in [Channel::Rx0, Channel::Rx1, Channel::Tx0, Channel::Tx1] {
        dev.set_frequency(channel, params.frequency)?;
        dev.set_sample_rate(channel, hyper.sample_rate)?;

        dev.set_gain_mode(channel, GainMode::Manual)?;
    }

    let rx0 = params.channel_set.contains(&Channel::Rx0);
    let rx1 = params.channel_set.contains(&Channel::Rx1);
    let tx0 = params.channel_set.contains(&Channel::Tx0);
    let tx1 = params.channel_set.contains(&Channel::Tx1);

    dev.set_bias_tee(Channel::Rx0, params.external_lna && rx0)?;
    dev.set_bias_tee(Channel::Rx1, params.external_lna && rx1)?;
    dev.set_bias_tee(Channel::Tx0, params.external_bias_tee && tx0)?;
    dev.set_bias_tee(Channel::Tx1, params.external_bias_tee && tx1)?;

    dev.set_gain(Channel::Rx0, params.rx_gain as i32)?;
    dev.set_gain(Channel::Rx1, params.rx_gain as i32)?;
    dev.set_gain(Channel::Tx0, params.tx_gain as i32)?;
    dev.set_gain(Channel::Tx1, params.tx_gain as i32)?;

    // Setup receiver if needed.
    let mut receiver = if rx0 || rx1 {
        let layout = if rx0 && rx1 {
            ChannelLayoutRx::MIMO
        } else if rx0 {
            ChannelLayoutRx::SISO(bladerf::RxChannel::Rx0)
        } else if rx1 {
            ChannelLayoutRx::SISO(bladerf::RxChannel::Rx1)
        } else {
            unreachable!();
        };
        let config = SyncConfig::new(
            hyper.num_buffers,
            hyper.num_samples as u32,
            hyper.num_transfers,
            hyper.timeout.as_millis() as u32,
        )?;
        let rx_streamer = dev
            .rx_streamer::<Complex<i16>>(&config, layout)
            .expect("Rx streamer");
        rx_streamer.enable().expect("Enable rx streamer");
        Some(rx_streamer)
    } else {
        None
    };

    // Setup sender if needed.
    let mut sender = if tx0 || tx1 {
        let layout = if tx0 && tx1 {
            ChannelLayoutTx::MIMO
        } else if tx0 {
            ChannelLayoutTx::SISO(bladerf::TxChannel::Tx0)
        } else if tx1 {
            ChannelLayoutTx::SISO(bladerf::TxChannel::Tx1)
        } else {
            unreachable!();
        };
        let config = SyncConfig::new(
            hyper.num_buffers,
            hyper.num_samples as u32,
            hyper.num_transfers,
            hyper.timeout.as_millis() as u32,
        )?;
        let tx_streamer = dev
            .tx_streamer::<Complex<i16>>(&config, layout)
            .expect("Tx streamer");
        tx_streamer.enable().expect("Enable tx streamer");
        Some(tx_streamer)
    } else {
        None
    };

    println!("Sampling {params:#?}");

    // Prepare buffers.
    let mut rx_buf = vec![Complex::<i16>::ZERO; hyper.num_samples];
    let tx_buf = vec![Complex::<i16>::new(0b1111_1111_1111, 0); hyper.num_samples];

    let mut samples_read = 0;
    let start = Instant::now();
    let mut last_update = start;

    let running = Arc::new(AtomicBool::new(true));
    // Move the clone of `running` outside the thread spawn.
    let running_clone = Arc::clone(&running);

    // Use a scoped thread so that we can safely borrow non-'static data.
    let measurements = std::thread::scope(|s| {
        // Spawn the TX thread using the cloned running flag.
        let tx_handle = s.spawn(move || {
            let mut samples_written = 0;
            while running_clone.load(Ordering::Acquire) {
                if let Some(ref mut tx) = sender {
                    tx.write(&tx_buf, hyper.timeout).expect("Write samples");
                    samples_written += hyper.num_samples;
                }
            }
            samples_written
        });

        // Main loop: perform RX sampling and log power data at 10Hz.
        let mut measurements: Vec<Measurement> = Vec::new();
        while start.elapsed() < hyper.sample_period {
            if let Some(ref mut rx) = receiver {
                rx.read(&mut rx_buf, hyper.timeout).expect("Read samples");
                samples_read += hyper.num_samples;
            }

            if last_update.elapsed() > Duration::from_millis(100) {
                let now = Instant::now();
                let dt = now.duration_since(last_update);
                // Update the global progress bar with the elapsed time in this measurement run.
                global_pb.inc(dt.as_millis() as u64);
                last_update = now;

                // Take measurements.
                let temperature = dev.get_rfic_temperature().expect("Temp error");
                let voltage_bus = dev
                    .get_pmic_register(PmicRegister::VoltageBus)
                    .expect("VoltageBus error");
                let voltage_shunt = dev
                    .get_pmic_register(PmicRegister::VoltageShunt)
                    .expect("VoltageShunt error");
                let power = dev
                    .get_pmic_register(PmicRegister::Power)
                    .expect("Power error");
                let current = dev
                    .get_pmic_register(PmicRegister::Current)
                    .expect("Current error");
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time error")
                    .as_secs_f64();
                measurements.push(Measurement {
                    timestamp,
                    temperature,
                    voltage_bus,
                    voltage_shunt,
                    power,
                    current,
                });
                // Update the progress bar message with current measurement values.
                let progress = (start.elapsed().as_millis() as f64
                    / hyper.sample_period.as_millis() as f64)
                    * 100.0;
                global_pb.set_message(format!(
                    "{:.1}% - Temp: {:.1}C, VBus: {:.2}V, VShunt: {:.2}V, Power: {:.2}W, Curr: {:.2}A",
                    progress, temperature, voltage_bus, voltage_shunt, power, current
                ));
            }
        }
        running.store(false, Ordering::Release);
        if let Some(ref mut rx) = receiver {
            rx.disable().expect("Failed to disable receiver");
        }
        let samples_written = tx_handle.join().unwrap();
        let throughput =
            (samples_read + samples_written) as f32 / start.elapsed().as_secs_f32() / 1_000_000.0;
        let summary = format!(
            "Read {} samples, wrote {}. Throughput: {:.2}M samples/sec",
            samples_read, samples_written, throughput
        );
        global_pb.println(&summary);
        measurements
    });

    Ok(measurements)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Check if the output directory exists.
    if args.output_dir.exists() {
        // If it exists, ensure it is empty.
        if fs::read_dir(&args.output_dir)?.next().is_some() {
            anyhow::bail!(
                "Output directory '{}' is not empty",
                args.output_dir.display()
            );
        }
    } else {
        // Create the directory if it does not exist.
        fs::create_dir_all(&args.output_dir)?;
    }

    println!("Opening device");
    let dev_any = BladeRfAny::open_first()?;
    let mut dev: BladeRf2 = dev_any.try_into().unwrap();

    // ========== Test Matrix ==========
    let frequencies = [
        87_000_000u64,
        // 225_000_000,
        // 550_000_000,
        1_500_000_000,
        // 3_000_000_000,
    ];

    let channels = [
        // Each single.
        vec![Channel::Rx0],
        vec![Channel::Rx1],
        vec![Channel::Tx0],
        vec![Channel::Tx1],
        // Dual channels.
        vec![Channel::Rx0, Channel::Rx1],
        vec![Channel::Tx0, Channel::Tx1],
        // 2x MIMO.
        vec![Channel::Rx0, Channel::Rx1, Channel::Tx0, Channel::Tx1],
    ];

    let amp_config = [
        // (false, false), (false, true), (true, false), (true, true)
        (false, false),
    ];

    // Gain RX1 overall:   60 dB (Range: [-15, 60])
    //             full:   71 dB (Range: [-4, 71])
    // Gain RX2 overall:   60 dB (Range: [-15, 60])
    //             full:   71 dB (Range: [-4, 71])
    // Gain TX1 overall:   56 dB (Range: [-23.75, 66])
    //              dsa:  -90 dB (Range: [-89.75, 0])
    // Gain TX2 overall:   56 dB (Range: [-23.75, 66])
    //              dsa:  -90 dB (Range: [-89.75, 0])

    let rx_gains = [0, 40, 77];
    let tx_gains = [-89, -45, 0];

    let hyper_params = HyperParameters {
        sample_rate: 5_000_000,
        sample_period: Duration::from_secs(15),
        num_samples: 16_384,
        timeout: Duration::from_secs(3),
        num_buffers: 8,
        num_transfers: 5,
    };

    // Calculate the total expected measurement time in milliseconds.
    let total_configs = (frequencies.len() * channels.len() * rx_gains.len()) as u64;
    let total_time_ms = total_configs * hyper_params.sample_period.as_millis() as u64;

    let warmup_time = Duration::from_secs(120);
    let warmup_pb = ProgressBar::new(warmup_time.as_millis() as u64);
    warmup_pb.set_style(
        ProgressStyle::with_template("{percent:>3}% {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    println!("Warming up radio for {}s", warmup_time.as_secs());
    let _ = perform_sampling(
        &mut dev,
        &HyperParameters {
            sample_period: warmup_time,
            sample_rate: hyper_params.sample_rate,
            num_samples: hyper_params.num_samples,
            timeout: hyper_params.timeout,
            num_buffers: hyper_params.num_buffers,
            num_transfers: hyper_params.num_transfers,
        },
        &Parameters {
            frequency: 815_000_000,
            channel_set: vec![Channel::Rx0, Channel::Rx1, Channel::Tx0, Channel::Tx1],
            rx_gain: 70,
            tx_gain: 0,
            external_bias_tee: false,
            external_lna: false,
        },
        &warmup_pb,
    )?;

    // Create a global progress bar for the entire run.
    let global_pb = ProgressBar::new(total_time_ms);
    global_pb.set_style(
        ProgressStyle::with_template("{percent:>3}% [{bar:40.cyan/blue}] {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // ========== Main Loop ==========
    for frequency in frequencies {
        for (rx_gain, tx_gain) in rx_gains.into_iter().zip(tx_gains) {
            for channel_set in &channels {
                for (external_lna, external_bias_tee) in amp_config {
                    // Create a parameters struct for the inner loop, including gain and external amp settings.
                    let params = Parameters {
                        frequency,
                        channel_set: channel_set.clone(),
                        external_bias_tee,
                        external_lna,
                        rx_gain,
                        tx_gain,
                    };

                    let measurements =
                        perform_sampling(&mut dev, &hyper_params, &params, &global_pb)?;

                    // Create a CSV file containing the vector of measurement data.
                    // The filename is the Base58-encoded JSON serialization of the parameters.
                    let params_serialized = serde_json::to_string(&params)?;
                    let filename =
                        format!("{}.csv", bs58::encode(&params_serialized).into_string());
                    let file_path = args.output_dir.join(&filename);
                    let mut file = File::create(&file_path)?;
                    writeln!(
                        file,
                        "timestamp,temperature,voltage_bus,voltage_shunt,power,current"
                    )?;
                    for m in measurements {
                        writeln!(
                            file,
                            "{:.6},{:.1},{:.2},{:.2},{:.2},{:.2}",
                            m.timestamp,
                            m.temperature,
                            m.voltage_bus,
                            m.voltage_shunt,
                            m.power,
                            m.current
                        )?;
                    }
                    println!("Saved measurements to {}", file_path.display());
                }
            }
        }
    }

    global_pb.finish_with_message("All measurements complete");
    Ok(())
}
