# seify-bladerf

Rust bindings to [nuand](https://www.nuand.com/)'s [libbladerf](https://github.com/Nuand/bladeRF/tree/master/host/libraries/libbladeRF) for the HackRF family of devices.

## Status

Most methods are implemented with the exception of:
- Async transfers
- Corrections
- Calibration
- Expansion boards

Supports libbladerf 2.5.0.

Only the functions used by the examples have been thoroughly tested, so your mileage may vary.
So far only tested on BladeRF 2.0 micro.

## Requirements

Enter a nix shell with:
```
nix develop
```
This will bring in cargo, rustc, libclang, etc. and will automatically download and set `BLADERF_RS_FPGA_BITSTREAM_PATH`, `BLADERF_RS_FX3_FIRMWARE_PATH`, based on the versions locked in [./bladerf.nix](bladerf.nix).
`BLADERF_INCLUDE_PATH` will also be set to a nix store path to facilitate running bindgen in [build.rs](libbladerf-sys/build.rs).

