use anyhow::{Context, Ok};
use bladerf::{BladeRF, BladeRfAny, ChannelLayoutRx, RxChannel, SyncConfig};
use num_complex::Complex;
use seify::RxStreamer;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug)]
enum CliChannel {
    Ch0,
    Ch1,
}

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

    #[arg(short, long, default_value = "CliChannel::Ch0")]
    channel: CliChannel,

    #[arg(long)]
    duration: Option<f32>,
}

fn complex_f32_to_u8(arr: &[Complex<f32>]) -> &[u8] {
    let len = std::mem::size_of_val(arr);
    let ptr = arr.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

fn rx_seify<T: RxStreamer>(mut rx_stream: T, limit: u64, fileuf: &mut BufWriter<&mut File>) {
    rx_stream.activate().unwrap();

    for _ in 0..limit {
        let mut buffer = [Complex::new(0_f32, 0.0); 8192];
        let mut buffs = [buffer.as_mut_slice()];

        rx_stream.read(&mut buffs, 3500000).unwrap();

        let data = complex_f32_to_u8(&buffer);

        fileuf
            .write_all(data)
            .with_context(|| "Could not write to file")
            .unwrap();
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let dev = if let Some(device) = args.device {
        BladeRfAny::open_identifier(&device)?
    } else {
        BladeRfAny::open_first()?
    };

    let channel = match args.channel {
        CliChannel::Ch0 => RxChannel::Rx0,
        CliChannel::Ch1 => RxChannel::Rx1,
    };

    dev.set_frequency(channel.into(), args.frequency)
        .with_context(|| {
            format!(
                "Unable to set frequency ({}) on the given channel ({:?}).",
                args.frequency, channel
            )
        })?;

    dev.set_sample_rate(channel.into(), args.samplerate)
        .with_context(|| {
            format!(
                "Unable to set sample rate ({}) on the given channel ({:?}).",
                args.samplerate, channel
            )
        })?;

    let config = SyncConfig::new(16, 8192, 8, 3500)?;

    let reciever = dev.rx_streamer::<Complex<i16>>(&config, ChannelLayoutRx::SISO(channel))?;

    let mut file = File::create(args.outfile)?;

    let mut file_buf = BufWriter::new(&mut file);

    let buffer_read_count_limit = {
        let sample_count = args.samplerate as f64 * args.duration.unwrap() as f64;
        let samples_per_block = 8192.0;
        (sample_count / samples_per_block) as u64
    };

    rx_seify(reciever, buffer_read_count_limit, &mut file_buf);

    Ok(())
}
