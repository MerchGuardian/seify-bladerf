use bladerf::*;
use std::*;

pub fn main() {
    let devices = BladeRF::get_device_list().unwrap();

    println!("Discovered {} devices", devices.len());

    for d in devices {
        println!("Device: {} Serial: {}", d.instance(), d.serial());
    }
}
