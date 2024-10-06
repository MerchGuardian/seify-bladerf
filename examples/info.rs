use bladerf::*;

pub fn main() -> anyhow::Result<()> {
    let devices = get_device_list().context("Failed to list BladeRF devices")?;
    println!("Discovered {} devices", devices.len());

    // Updates firmware to latest
    let do_firmware_update = true;
    let desired_firmware_version = Version {
        major: 2,
        minor: 5,
        patch: 0,
        describe: None,
    };

    for d in devices {
        println!(
            "Device {}:\n  Serial: {}\n  Manufacturer: {}\n  Product: {}\n",
            d.instance(),
            d.serial(),
            d.manufacturer(),
            d.product()
        );

        let dev = d.open().expect("Failed to open device");
        dev.load_fpga_from_env()
            .expect("Failed to load fpga bitstream");

        print_device_info(&dev);
        print_loopback_info(&dev);
        print_sampling_info(&dev);
        println!();

        for ch in [Channel::Rx1, Channel::Rx2, Channel::Tx1, Channel::Tx2] {
            print_channel_info(&dev, ch);
        }

        if do_firmware_update {
            if let Ok(current) = dev.firmware_version() {
                dbg!(&current, &desired_firmware_version);
                if current < desired_firmware_version {
                    println!("Firmware out of date. Updating from {current} to {desired_firmware_version}");
                }
            }
        }
    }

    Ok(())
}

fn print_device_info(dev: &BladeRF) -> anyhow::Result<()> {
    if let Ok(fw_version) = dev.firmware_version() {
        println!("  Firmware Version: {fw_version}");
    }

    if let Ok(fpga_version) = dev.fpga_version() {
        println!("  FPGA Version: {fpga_version}",);
    }

    if let Ok(fpga_size) = dev.get_fpga_size() {
        println!("  FPGA Size: {:?}", fpga_size);
    }

    if let Ok(is_configured) = dev.is_fpga_configured() {
        println!("  FPGA Configured: {}", is_configured);
    }

    if let Ok(serial) = dev.get_serial() {
        println!("  Serial Number: {}", serial);
    }

    Ok(())
}

fn print_channel_info(dev: &BladeRF, channel: Channel) -> anyhow::Result<()> {
    println!("  Channel {channel:?}");
    if let Ok(freq) = dev.get_frequency(channel) {
        println!("    Frequency: {freq} Hz");
    }

    if let Ok(bw) = dev.get_bandwidth(channel) {
        println!("    Bandwidth: {bw} Hz");
    }

    if let Ok(lpf_mode) = dev.get_lpf_mode(channel) {
        println!("    LPF Mode: {lpf_mode:?}");
    }

    if let Ok(gain) = dev.get_gain(channel) {
        println!("    Gain: {gain} dB");
    }

    println!("");
    if let Ok(modes) = dev.get_gain_modes(channel) {
        println!("    Gain Modes:");
        for mode_info in modes {
            println!("      Mode: {} ({:?})", mode_info.name, mode_info.mode);
        }
    }

    if let Ok(stages) = dev.get_gain_stages(channel) {
        println!("    Gain Stages:");
        for stage in stages {
            println!("      Stage: {stage}");

            if let Ok(gain) = dev.get_gain_stage(channel, &stage) {
                println!("        Gain: {gain} dB");
            }

            if let Ok(range) = dev.get_gain_stage_range(channel, &stage) {
                println!(
                    "        Range: min = {:.2} dB, max = {:.2} dB, step = {:.2} dB",
                    range.min, range.max, range.step
                );
            }
        }
    }
    println!();

    Ok(())
}

fn print_loopback_info(dev: &BladeRF) -> anyhow::Result<()> {
    if let Ok(loopback_modes) = dev.get_loopback_modes() {
        println!("  Supported Loopback Modes:");
        for mode_info in loopback_modes {
            println!(
                "    Mode: {} ({:?})",
                mode_info.name.as_deref().unwrap_or("Unknown"),
                mode_info.mode
            );
        }
    }

    if let Ok(current_loopback) = dev.get_loopback() {
        println!("  Current Loopback Mode: {:?}", current_loopback);
    }

    Ok(())
}

fn print_sampling_info(dev: &BladeRF) -> anyhow::Result<()> {
    if let Ok(sampling) = dev.get_sampling() {
        println!("  Sampling Mode: {:?}", sampling);
    }

    if let Ok(rx_mux) = dev.get_rx_mux() {
        println!("  RX Mux Mode: {:?}", rx_mux);
    }

    Ok(())
}
