use anyhow::Context;
use bladerf::*;

pub fn main() -> anyhow::Result<()> {
    println!(
        "libbladerf version: {}",
        bladerf::version().context("Failed to obtain bladerf version")?
    );
    let devices = get_device_list().context("Failed to list BladeRF devices")?;
    println!("Discovered {} devices", devices.len());

    for d in devices {
        println!(
            "Device {}:\n  Serial: {}\n  Manufacturer: {}\n  Product: {}\n",
            d.instance(),
            d.serial(),
            d.manufacturer(),
            d.product()
        );

        let dev = d.open().context("Failed to open device")?;

        dev.load_fpga_from_env()
            .context("Failed to load FPGA bitstream")?;

        let _ = print_device_info(&dev)
            .context("Failed to print device information")
            .map_err(|e| println!("{e:?}"));
        println!();
        let _ = print_loopback_info(&dev)
            .context("Failed to print loopback information")
            .map_err(|e| println!("{e:?}"));
        println!();
        let _ = print_sampling_info(&dev)
            .context("Failed to print sampling information")
            .map_err(|e| println!("{e:?}"));
        println!();

        for ch in [Channel::Rx1, Channel::Rx2, Channel::Tx1, Channel::Tx2] {
            let _ = print_channel_info(&dev, ch)
                .context(format!("Failed to print channel information for {:?}", ch))
                .map_err(|e| println!("{e:?}"));
        }
    }

    Ok(())
}

fn print_device_info(dev: &BladeRF) -> anyhow::Result<()> {
    let fw_version = dev
        .firmware_version()
        .context("Failed to retrieve firmware version")?;
    println!("  Firmware Version: {fw_version}");

    let fpga_version = dev
        .fpga_version()
        .context("Failed to retrieve FPGA version")?;
    println!("  FPGA Version: {fpga_version}");

    let fpga_size = dev
        .get_fpga_size()
        .context("Failed to retrieve FPGA size")?;
    println!("  FPGA Size: {:?}", fpga_size);

    let is_configured = dev
        .is_fpga_configured()
        .context("Failed to check if FPGA is configured")?;
    println!("  FPGA Configured: {}", is_configured);

    let serial = dev
        .get_serial()
        .context("Failed to retrieve serial number")?;
    println!("  Serial Number: {}", serial);

    Ok(())
}

fn print_channel_info(dev: &BladeRF, channel: Channel) -> anyhow::Result<()> {
    println!("  Channel {channel:?}");

    // frequency
    let freq = dev
        .get_frequency(channel)
        .context("Failed to retrieve frequency")?;
    println!("    Frequency: {freq} Hz");

    let frequency_range = dev
        .get_frequency_range(channel)
        .context("Failed to reterve frequency range")?;
    println!("    Bandwidth range: {frequency_range}");

    // bandwidth
    let bw = dev
        .get_bandwidth(channel)
        .context("Failed to retrieve bandwidth")?;
    println!("    Bandwidth: {bw} Hz");

    let bandwidth_range = dev
        .get_bandwidth_range(channel)
        .context("Failed to reterve bandwidth range")?;
    println!("    Bandwidth range: {bandwidth_range}");

    // sample rate
    let sample_rate = dev
        .get_sample_rate(channel)
        .context("Failed to reterve sample rate")?;
    println!("    Sample rate: {sample_rate} Hz");

    let sample_rate_range = dev
        .get_sample_rate_range(channel)
        .context("Failed to reterve sample rate range")?;
    println!("    Sample rate range: {sample_rate_range}");

    if channel.is_rx() {
        let gain = dev.get_gain(channel).context("Failed to retrieve gain")?;
        println!("    Gain: {gain} dB");

        let modes = dev
            .get_gain_modes(channel)
            .context("Failed to retrieve gain modes")?;
        println!("    Gain Modes:");
        for mode_info in modes {
            println!("      Mode: {} ({:?})", mode_info.name, mode_info.mode);
        }

        let stages = dev
            .get_gain_stages(channel)
            .context("Failed to retrieve gain stages")?;
        println!("    Gain Stages:");
        for stage in stages {
            println!("      Stage: {stage}");

            let gain = dev
                .get_gain_stage(channel, &stage)
                .context(format!("Failed to retrieve gain for stage {stage}"))?;
            println!("        Gain: {gain} dB");

            let range = dev
                .get_gain_stage_range(channel, &stage)
                .context(format!("Failed to retrieve gain range for stage {stage}"))?;
            println!(
                "        Range: min = {:.2} dB, max = {:.2} dB, step = {:.2} dB",
                range.min, range.max, range.step
            );
        }

        // let lpf_mode = dev
        //     .get_lpf_mode(channel)
        //     .context("Failed to retrieve LPF mode")?;
        // println!("    LPF Mode: {lpf_mode:?}");
    }

    println!();
    Ok(())
}

fn print_loopback_info(dev: &BladeRF) -> anyhow::Result<()> {
    let loopback_modes = dev
        .get_loopback_modes()
        .context("Failed to retrieve loopback modes")?;
    println!("  Supported Loopback Modes:");
    for mode_info in loopback_modes {
        println!(
            "    Mode: {} ({:?})",
            mode_info.name.as_deref().unwrap_or("Unknown"),
            mode_info.mode
        );
    }

    let current_loopback = dev
        .get_loopback()
        .context("Failed to retrieve current loopback mode")?;
    println!("  Current Loopback Mode: {:?}", current_loopback);

    Ok(())
}

fn print_sampling_info(dev: &BladeRF) -> anyhow::Result<()> {
    // let sampling = dev
    //     .get_sampling()
    //     .context("Failed to retrieve sampling mode")?;
    // println!("  Sampling Mode: {:?}", sampling);

    let rx_mux = dev.get_rx_mux().context("Failed to retrieve RX Mux mode")?;
    println!("  RX Mux Mode: {:?}", rx_mux);

    Ok(())
}
