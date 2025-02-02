use anyhow::{Context, Ok};
use bladerf::{BladeRF, BladeRfAny, Channel, SyncConfig};
use num_complex::Complex;
use std::{
    fs::File,
    io::{BufWriter, Write},
    time::Duration,
};

fn complex_i16_to_u8(arr: &[Complex<i16>]) -> &[u8] {
    let len = std::mem::size_of_val(arr);
    let ptr = arr.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

fn main() -> anyhow::Result<()> {
    let dev = BladeRfAny::open_first()?;

    // ========== Test Matrix ==========

    let frequencies = [
        50_000_000u64,
        /*225_000_000.0,*/ 550_000_000,
        1_500_000_000,
        3_000_000_000,
    ];

    let channels = [
        // Each single
        vec![Channel::Rx0],
        vec![Channel::Rx1],
        vec![Channel::Tx0],
        vec![Channel::Tx1],
        //
        vec![Channel::Rx0, Channel::Rx1],
        vec![Channel::Tx0, Channel::Tx1],
        // 2x MIMO
        vec![Channel::Rx0, Channel::Rx1, Channel::Tx0, Channel::Tx1],
    ];

    let sample_rate = 5_000_000;

    // ========== Main Loop ==========
    for frequency in frequencies {
        for channel_set in &channels {
            for channel in [Channel::Rx0, Channel::Rx1, Channel::Tx0, Channel::Tx1] {
                let is_enabled = channel_set.contains(&channel);

                dev.set_frequency(channel, frequency)
                    .expect("Failed to set frequency");

                dev.set_sample_rate(channel, sample_rate)
                    .expect("Failed to set sample rate");

                let config = SyncConfig::new(16, 8192, 8, 3500)?;

                let reciever = dev.rx_streamer::<Complex<i16>>(&config, false)?;
            }
        }
    }

    Ok(())
}
