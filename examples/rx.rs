use std::time::Duration;

use anyhow::Context;
use bladerf::{Channel, Format};
use num_complex::Complex;

pub fn main() -> anyhow::Result<()> {
    println!(
        "libbladerf version: {}",
        bladerf::version().context("Failed to obtain bladerf version")?
    );
    let device = bladerf::BladeRF::open_first().context("Failed to list BladeRF devices")?;
    let info = device.info().context("Failed to obtain device info")?;

    println!(
        "Device {}:\nSerial: {}\nManufacturer: {}\nProduct: {}\n",
        info.instance(),
        info.serial(),
        info.manufacturer(),
        info.product()
    );

    device
        .load_fpga_from_env()
        .context("Failed to load FPGA bitstream")?;

    device
        .enable_module(Channel::Rx1)
        .context("Enable module")?;

    let num_buffers = 4;
    let buffer_size = 64 * 1024;
    let num_transfers = 2;
    let timeout = Duration::from_millis(500);
    device
        .sync_config(
            Channel::Rx1,
            Format::Sc16Q11,
            num_buffers,
            buffer_size,
            num_transfers,
            timeout,
        )
        .context("Failed to sync config")?;

    let mut samples = vec![Complex::<i16>::ZERO; buffer_size as usize];
    let mut meta = None;
    device
        .sync_rx(&mut samples, meta.as_mut(), timeout)
        .context("Receive samples")?;

    dbg!(meta);
    dbg!(&samples[..32]);

    Ok(())
}
