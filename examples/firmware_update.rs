use std::time::Instant;

use anyhow::Context;
use bladerf::*;

pub fn main() -> anyhow::Result<()> {
    println!(
        "libbladerf version: {}",
        bladerf::version().context("Failed to obtain bladerf version")?
    );
    let devices = get_device_list().context("Failed to list BladeRF devices")?;
    println!("Discovered {} devices", devices.len());

    let var_name = "BLADERF_RS_FX3_FIRMWARE_PATH";
    let firmware_path = std::env::var(var_name).map_err(|e| {
        Error::msg(format!(
            "Failed to read env var {var_name}: {e:?}. Are you running from a nix shell?"
        ))
    })?;

    let desired_firmware_version = Version {
        major: 2,
        minor: 4,
        patch: 0,
        describe: None,
    };

    'device_loop: for info in devices {
        println!(
            "\nDevice {}:\n  Serial: {}\n  Manufacturer: {}\n  Product: {}\n",
            info.instance(),
            info.serial(),
            info.manufacturer(),
            info.product()
        );

        let Ok(dev) = info
            .open()
            .map_err(|e| println!("Failed to open device: {e:?}"))
        else {
            continue;
        };

        if let Ok(current) = dev
            .firmware_version()
            .map_err(|e| println!("Failed to obtain firmware version: {e:?}"))
        {
            if current == desired_firmware_version {
                println!("Device firmware up to date");
            } else if current < desired_firmware_version {
                println!(
                    "Firmware out of date. Updating from {current} to {desired_firmware_version}"
                );
                println!("Continue? y/n");
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                if line.trim() != "y" {
                    println!("Aborting firmware update");
                    continue;
                }
                println!("Erasing saved fpga version");
                if let Err(e) = dev.erase_stored_fpga() {
                    println!("Failed to erase stored fpga, continuning anyway: {e:?}");
                }

                // do firmware update
                if let Err(e) = dev.flash_firmware(&firmware_path) {
                    println!("Failed to flash firmware: {e:?}");
                    continue;
                }
                if let Err(e) = dev.device_reset() {
                    println!("Failed to reset device after firmware update: {e:?}");
                    continue;
                }

                let start = Instant::now();
                // Try to reconnect and verify firmware version for up to 5 seconds
                while start.elapsed().as_secs() < 5 {
                    let Ok(dev) = BladeRfAny::open_with_devinfo(&info) else {
                        continue;
                    };
                    let Ok(new_version) = dev
                        .firmware_version()
                        .map_err(|e| println!("Failed to read firmware version: {e:?}"))
                    else {
                        continue;
                    };

                    if new_version == desired_firmware_version {
                        println!("Firmware updated to {new_version} successfully");
                        println!();
                    } else {
                        println!("Firmware update failed. Version after flashing: {new_version}");
                        let path = "blade_fw_log.txt";
                        if let Err(e) = dev.get_fw_log(path) {
                            println!("Failed to download firmware log: {e:?}");
                        } else {
                            println!("Saved firmware log to {path}");
                        }
                    }
                    continue 'device_loop;
                }
                println!("Unable to communicate with device after 5 seconds. Manual firmware update / power cycle may be needed");

                //
            } else {
                println!("Firmware version on device is ahead of known version?! Current: {current} desired: {desired_firmware_version}");
            }
        }
    }
    Ok(())
}
