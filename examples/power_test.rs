use anyhow::Ok;

use bladerf::{
    BladeRF, BladeRf2, BladeRfAny, Channel, ChannelLayoutRx, ChannelLayoutTx, PmicRegister,
    SyncConfig,
};
use clap::Parser;
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

use base64;
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

#[derive(Serialize, Deserialize, Clone)]
struct Parameters {
    frequency: u64,
    channel_set: Vec<Channel>,
}

#[derive(Serialize)]
struct Measurement {
    timestamp: u128,
    temperature: f32,
    voltage_bus: f32,
    voltage_shunt: f32,
    power: f32,
    current: f32,
}

fn perform_sampling(
    dev: &mut BladeRf2,
    hyper: &HyperParameters,
    params: &Parameters,
) -> anyhow::Result<Vec<Measurement>> {
    // Set frequency and sample rate for each channel using hyper parameters.
    for channel in [Channel::Rx0, Channel::Rx1, Channel::Tx0, Channel::Tx1] {
        dev.set_frequency(channel, params.frequency)
            .expect("Failed to set frequency");
        dev.set_sample_rate(channel, hyper.sample_rate)
            .expect("Failed to set sample rate");
    }

    // Determine active channels from the parameters.
    let rx0 = params.channel_set.contains(&Channel::Rx0);
    let rx1 = params.channel_set.contains(&Channel::Rx1);
    let tx0 = params.channel_set.contains(&Channel::Tx0);
    let tx1 = params.channel_set.contains(&Channel::Tx1);

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
        let mut rx_streamer = dev
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
        let mut tx_streamer = dev
            .tx_streamer::<Complex<i16>>(&config, layout)
            .expect("Tx streamer");
        tx_streamer.enable().expect("Enable tx streamer");
        Some(tx_streamer)
    } else {
        None
    };

    println!();
    println!(
        "Starting Sample for freq: {} MHz. Channels active: {:?}",
        params.frequency as f32 / 1_000_000.0,
        params.channel_set
    );

    // Prepare buffers.
    let mut rx_buf = vec![Complex::<i16>::ZERO; hyper.num_samples];
    let tx_buf = vec![Complex::<i16>::new(0b1111_1111_1111, 0); hyper.num_samples];

    let mut samples_read = 0;
    let mut last_print = Instant::now();
    let start = Instant::now();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // Use a scoped thread so that we can safely borrow non-'static data.
    let measurements = std::thread::scope(|s| {
        // Spawn the tx thread.
        let tx_handle = s.spawn(|| {
            let mut samples_written = 0;
            while running_clone.load(Ordering::Acquire) {
                if let Some(ref mut tx) = sender {
                    tx.write(&tx_buf, hyper.timeout).expect("Write samples");
                    samples_written += hyper.num_samples;
                }
            }
            samples_written
        });

        // Main loop: perform rx sampling and log power data at 10Hz.
        let mut measurements: Vec<Measurement> = Vec::new();
        while start.elapsed() < hyper.sample_period {
            if let Some(ref mut rx) = receiver {
                rx.read(&mut rx_buf, hyper.timeout)
                    .expect("Read samples");
                samples_read += hyper.num_samples;
            }

            if last_print.elapsed() > Duration::from_millis(100) {
                last_print = Instant::now();
                let temperature = dev.get_rfic_temperature().expect("Temp error");
                let voltage_bus = dev
                    .get_pmic_register(PmicRegister::VoltageBus)
                    .expect("VoltageBus error");
                let voltage_shunt = dev
                    .get_pmic_register(PmicRegister::VoltageShunt)
                    .expect("VoltageShunt error");
                let power = dev.get_pmic_register(PmicRegister::Power).expect("Power error");
                let current = dev.get_pmic_register(PmicRegister::Current).expect("Current error");
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time error")
                    .as_millis();
                measurements.push(Measurement {
                    timestamp,
                    temperature,
                    voltage_bus,
                    voltage_shunt,
                    power,
                    current,
                });
                println!(
                    "{:.1}C, Voltage Bus: {:.2}V, Voltage Shunt: {:.2}V",
                    temperature, voltage_bus, voltage_shunt
                );
                println!("Current: {:.2}A, power: {:.2}W", current, power);
            }
        }
        running.store(false, Ordering::Release);
        if let Some(ref mut rx) = receiver {
            rx.disable().expect("Failed to disable receiver");
        }
        let samples_written = tx_handle.join().unwrap();

        println!(
            "Read {} samples, wrote {}. Throughput: {:.2}M samples/sec",
            samples_read,
            samples_written,
            (samples_read + samples_written) as f32 / start.elapsed().as_secs_f32() / 1_000_000.0
        );
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
            anyhow::bail!("Output directory '{}' is not empty", args.output_dir.display());
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
        /*225_000_000,*/ 550_000_000,
        1_500_000_000,
        3_000_000_000,
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

    let hyper_params = HyperParameters {
        sample_rate: 5_000_000,
        sample_period: Duration::from_secs(15),
        num_samples: 16_384,
        timeout: Duration::from_secs(3),
        num_buffers: 8,
        num_transfers: 5,
    };

    // ========== Main Loop ==========
    for frequency in frequencies {
        for channel_set in &channels {
            // Create a parameters struct for the inner loop.
            let params = Parameters {
                frequency,
                channel_set: channel_set.clone(),
            };

            let measurements = perform_sampling(&mut dev, &hyper_params, &params)?;

            // Create a CSV file containing the vector of measurement data.
            // The filename is the Base64-encoded JSON serialization of the parameters.
            let params_serialized = serde_json::to_string(&params)?;
            let filename = format!("{}.csv", base64::encode(&params_serialized));
            let file_path = args.output_dir.join(&filename);
            let mut file = File::create(&file_path)?;
            writeln!(
                file,
                "timestamp,temperature,voltage_bus,voltage_shunt,power,current"
            )?;
            for m in measurements {
                writeln!(
                    file,
                    "{},{:.1},{:.2},{:.2},{:.2},{:.2}",
                    m.timestamp, m.temperature, m.voltage_bus, m.voltage_shunt, m.power, m.current
                )?;
            }
            println!("Saved measurements to {}", file_path.display());
        }
    }

    Ok(())
}

