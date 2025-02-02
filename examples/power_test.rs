use anyhow::Ok;
use bladerf::{BladeRF, BladeRf2, BladeRfAny, Channel, ChannelLayoutRx, PmicRegister, SyncConfig};
use num_complex::Complex;
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    let dev = BladeRfAny::open_first()?;
    let dev: BladeRf2 = dev.try_into().expect("Expected bladerf 2.0");

    // ========== Test Matrix ==========

    let frequencies = [
        87_000_000u64,
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

    // Length to sample each configuration for
    let sample_period = Duration::from_secs(15);

    const NUM_SAMPLES: usize = 16_384;
    let timeout = Duration::from_secs(3);

    // ========== Main Loop ==========
    for frequency in frequencies {
        for channel_set in &channels {
            for channel in [Channel::Rx0, Channel::Rx1, Channel::Tx0, Channel::Tx1] {
                dev.set_frequency(channel, frequency)
                    .expect("Failed to set frequency");

                dev.set_sample_rate(channel, sample_rate)
                    .expect("Failed to set sample rate");
            }

            let rx0 = channel_set.contains(&Channel::Rx0);
            let rx1 = channel_set.contains(&Channel::Rx1);

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
                let config = SyncConfig::new(8, NUM_SAMPLES as u32, 5, timeout.as_millis() as u32)?;
                let receiver = dev
                    .rx_streamer::<Complex<i16>>(&config, layout)
                    .expect("Rx streamer");
                receiver.enable().expect("Enable rx steramer");
                Some(receiver)
            } else {
                None
            };

            println!();
            println!(
                "Starting Sample for freq: {}Mhz. Channels active: {channel_set:?}",
                frequency as f32 / 1_000_000.0
            );
            let mut buf = [Complex::<i16>::ZERO; NUM_SAMPLES];

            let mut samples_read = 0;

            let mut last_print = Instant::now();
            let start = Instant::now();
            while start.elapsed() < sample_period {
                if let Some(rx) = receiver.as_mut() {
                    rx.read(&mut buf, timeout).expect("Read samples");
                    samples_read += NUM_SAMPLES;
                }

                if last_print.elapsed().as_secs() > 1 {
                    last_print = Instant::now();
                    let temp = dev.get_rfic_temperature()?;
                    let voltage_bus = dev.get_pmic_register(PmicRegister::VoltageBus)?;
                    let voltage_shunt = dev.get_pmic_register(PmicRegister::VoltageShunt)?;
                    let power = dev.get_pmic_register(PmicRegister::Power)?;
                    let current = dev.get_pmic_register(PmicRegister::Current)?;
                    println!(
                        "{temp:.1}C, Voltage Bus: {voltage_bus:.2}V, Voltage Shunt: {voltage_shunt:.2}V"
                    );
                    println!("Current: {current:.2}A, power: {power:.2}W");
                }
            }

            println!(
                "Read {samples_read} samples {}M samples/sec",
                samples_read as f32 / start.elapsed().as_secs_f32() / 1_000_000.0
            );
        }
    }

    Ok(())
}
