use anyhow::{Context, Ok};
use bladerf::{BladeRF, BladeRfAny, ChannelLayoutRx, RxChannel, SyncConfig};
use indicatif::{ProgressBar, ProgressStyle};
use num_complex::Complex;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::mpsc::TryRecvError,
    time::Duration,
};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug)]
enum CliChannel {
    Ch0,
    Ch1,
}

const SAMPLES_PER_BLOCK: u32 = 8192;

/// Simple program to receive samples from a bladeRF and write them to a file.
/// The output file will be a binary file containing interleaved I and Q samples
/// where each sample is a 16-bit little endian signed integer.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    outfile: PathBuf,

    #[arg(short, long)]
    device: Option<String>,

    #[arg(short, long)]
    frequency: u64,

    #[arg(short, long)]
    samplerate: u32,

    /// The channel to use. Defaults to Ch0.
    #[arg(short, long)]
    channel: Option<CliChannel>,

    /// How long to recieve samples for in seconds. If not provided, will run indefinitely.
    #[arg(long)]
    duration: Option<f32>,

    #[arg(long)]
    noprogress: bool,
}

fn complex_i16_to_u8(arr: &[Complex<i16>]) -> &[u8] {
    let len = std::mem::size_of_val(arr);
    let ptr = arr.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    pretty_env_logger::init();

    log::debug!("Args: {:#?}", args);

    let dev = if let Some(device) = args.device {
        log::debug!("Opening device with device identifier: {}", device);
        BladeRfAny::open_identifier(&device).with_context(|| "Cannot Open Device")?
    } else {
        log::debug!("Opening first device");
        BladeRfAny::open_first().with_context(|| "Cannot Open Device")?
    };

    let channel = args
        .channel
        .map(|c| match c {
            CliChannel::Ch0 => RxChannel::Rx0,
            CliChannel::Ch1 => RxChannel::Rx1,
        })
        .unwrap_or(RxChannel::Rx0);

    log::debug!("Configuring channel {:?}", channel);

    dev.set_frequency(channel.into(), args.frequency)
        .with_context(|| {
            format!(
                "Unable to set frequency ({}) on the given channel ({:?}).",
                args.frequency, channel
            )
        })?;

    log::debug!("Frequency set to {}", args.frequency);

    dev.set_sample_rate(channel.into(), args.samplerate)
        .with_context(|| {
            format!(
                "Unable to set sample rate ({}) on the given channel ({:?}).",
                args.samplerate, channel
            )
        })?;

    log::debug!("Sample rate set to {}", args.samplerate);

    let config = SyncConfig::new(16, SAMPLES_PER_BLOCK, 8, 3500)
        .with_context(|| "Cannot Create Sync Config")?;
    let layout = ChannelLayoutRx::SISO(channel);
    let reciever = dev
        .rx_streamer::<Complex<i16>>(&config, layout)
        .with_context(|| "Cannot Get Streamer")?;

    let file = File::create(args.outfile).with_context(|| "Cannot Open Output File")?;
    let mut file_buf = BufWriter::new(file);
    let mut buffer = [Complex::new(0_i16, 0); SAMPLES_PER_BLOCK as usize];

    log::debug!("Opened file for writing");

    reciever.enable().with_context(|| "Cannot Enable Stream")?;

    log::debug!("Stream enabled");

    let (ctrlc_tx, ctrlc_rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(());
    })
    .with_context(|| "Cannot Set Ctrl-C Handler")?;

    log::info!("Starting to receive samples");

    let bar_style = ProgressStyle::with_template(
        "{spinner:.blue} [{elapsed_precise}] {binary_bytes} written to disk.",
    )
    .unwrap();
    let progress = ProgressBar::no_length().with_style(bar_style);

    let mut reciever_inner = || -> anyhow::Result<()> {
        reciever
            .read(&mut buffer, Duration::from_secs(1))
            .with_context(|| "Cannot Read Samples")?;

        let data = complex_i16_to_u8(&buffer);

        file_buf
            .write_all(data)
            .with_context(|| "Could not write to file")?;

        if !args.noprogress {
            progress.inc(SAMPLES_PER_BLOCK as u64 * size_of::<Complex<i16>>() as u64);
        }

        Ok(())
    };

    match args.duration {
        Some(duration) => {
            let buffer_read_count_limit = {
                let sample_count = args.samplerate as f64 * duration as f64;
                let samples_per_block = SAMPLES_PER_BLOCK as f64;
                (sample_count / samples_per_block) as u64
            };

            for _ in 0..buffer_read_count_limit {
                reciever_inner()?;
                match ctrlc_rx.try_recv() {
                    std::result::Result::Ok(_) => break,
                    Err(TryRecvError::Disconnected) => break,
                    _ => {}
                }
            }
        }
        None => loop {
            reciever_inner()?;
            match ctrlc_rx.try_recv() {
                std::result::Result::Ok(_) => break,
                Err(TryRecvError::Disconnected) => break,
                _ => {}
            }
        },
    }

    log::info!("Finished receiving samples");

    file_buf.flush().with_context(|| "Cannot Flush File")?;
    let file = file_buf.into_inner().with_context(|| "Cannot Get File")?;
    file.sync_all().with_context(|| "Cannot Sync File")?;

    Ok(())
}
