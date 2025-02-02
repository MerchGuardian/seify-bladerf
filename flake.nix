{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-24.05";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
          overlays = [
            (import rust-overlay)
          ];
        };
        rust-pkgs = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
        ndkVersion = "25.1.8937393";
        androidPackages = pkgs.androidenv.composeAndroidPackages {
          includeNDK = true;
          ndkVersions = [ ndkVersion ];
          abiVersions = [ "arm64-v8a" ];
        };
        androidSdk = androidPackages.androidsdk;
        ndk = "${androidSdk}/libexec/android-sdk/ndk/${ndkVersion}";
        bladerf-src = pkgs.fetchFromGitHub {
          owner = "Nuand";
          repo = "bladeRF";
          rev = "fe3304d75967c88ab4f17ff37cb5daf8ff53d3e1";
          sha256 = "sha256-fr9snjWjHees/3VIq8Gyao1ppnzsaFs7WEu1Un8xdu0=";
          fetchSubmodules = true;
        };
        bladerf = import ./bladerf.nix {
          inherit bladerf-src;
          inherit (pkgs) lib fetchurl fetchpatch symlinkJoin ncurses stdenv pkg-config cmake git doxygen help2man tecla libusb1 udev;
        };
        android = import ./android.nix {
          inherit ndk bladerf-src;
          inherit (pkgs) stdenvNoCC autoreconfHook fetchFromGitHub libtool lib cmake pkg-config git;
        };
      in with pkgs; {
        packages = {
          libbladerf = bladerf.libbladerf;
          xa4-bitstream = bladerf.xa4-bitstream;
          fx3-firmware = bladerf.fx3-firmware;
        };

        devShells = {
          default = pkgs.mkShell {
            # Include native inputs so we can use this shell to dev on bladeRF c code
            inputsFrom = [ bladerf.libbladerf ];

            shellHook = ''
              export LIBCLANG_PATH="${llvmPackages_14.clang.cc.lib}/lib";

              export BLADERF_INCLUDE_PATH="${bladerf.libbladerf}/include";
              export BLADERF_RS_FPGA_BITSTREAM_PATH="${bladerf.xa4-bitstream}";
              export BLADERF_RS_FX3_FIRMWARE_PATH="${bladerf.fx3-firmware}";
              export RUSTFLAGS="-L ${bladerf.libbladerf}/lib";
              export PATH="${bladerf.libbladerf}/bin:$PATH";
            '';

            packages = [
              rust-pkgs
            ];
          };
        };
      });
}
