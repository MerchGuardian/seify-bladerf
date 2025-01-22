use anyhow::{Context, Ok};
use bladerf::{BladeRF, BladeRfAny, Channel, SyncConfig};
use log::info;
use num_complex::Complex;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Duration,
};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug)]
enum CliChannel {
    Ch0,
    Ch1,
}

/// Simple program to greet a person
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

    #[arg(short, long)]
    channel: Option<CliChannel>,

    #[arg(long)]
    duration: Option<f32>,
}

fn complex_i16_to_u8(arr: &[Complex<i16>]) -> &[u8] {
    let len = std::mem::size_of_val(arr);
    let ptr = arr.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let dev = if let Some(device) = args.device {
        BladeRfAny::open_identifier(&device)?
    } else {
        BladeRfAny::open_first()?
    };

    let channel = args
        .channel
        .map(|c| match c {
            CliChannel::Ch0 => Channel::Rx0,
            CliChannel::Ch1 => Channel::Rx1,
        })
        .unwrap_or(Channel::Rx0);

    println!("Hi");

    dev.set_frequency(channel, args.frequency)
        .with_context(|| {
            format!(
                "Unable to set frequency ({}) on the given channel ({:?}).",
                args.frequency, channel
            )
        })?;

    dev.set_sample_rate(channel, args.samplerate)
        .with_context(|| {
            format!(
                "Unable to set sample rate ({}) on the given channel ({:?}).",
                args.samplerate, channel
            )
        })?;

    println!("Hi2");

    let config = SyncConfig::new(16, 8192, 8, 3500)?;

    let reciever = dev.rx_streamer::<Complex<i16>>(&config, false)?;

    let mut file = File::create(args.outfile)?;

    let mut file_buf = BufWriter::new(&mut file);

    let buffer_read_count_limit = {
        let sample_count = args.samplerate as f64 * args.duration.unwrap() as f64;
        let samples_per_block = 8192 as f64;
        (sample_count / samples_per_block) as u64
    };

    println!("Hi3 {buffer_read_count_limit}");

    reciever.enable(channel)?;

    println!("Hi4");

    // let tmp_vec = Vec::with_capacity(8192);

    for _ in 0..buffer_read_count_limit {
        let mut buffer = [Complex::new(0_i16, 0); 8192];
        // let mut buffer = [inner_buffer.as_mut_slice(); 1];

        reciever.read(&mut buffer, Duration::from_secs(1))?;
        println!("RX");

        let data = complex_i16_to_u8(&buffer);

        // println!("D: {:?}", inner_buffer);

        println!("RX2");

        // let data = [0, 0, 0, 0, 0_u8];

        file_buf
            .write_all(data)
            .with_context(|| "Could not write to file")?;
    }

    println!("Fin");

    Ok(())
}
