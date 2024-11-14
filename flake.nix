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
        rust-pkgs = pkgs.rust-bin.nightly.latest.default.override {
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
          owner = "MerchGuardian";
          repo = "bladeRF";
          rev = "67f5d683b53196f761ec3cb9f84bdd3f6c96d49a";
          sha256 = "sha256-PvOlMLhTAiJDwLMToTh4AVXdEgLkp/FO/uV651hgGWc=";
          fetchSubmodules = true;
        };
        bladerf = import ./bladerf.nix {
          inherit bladerf-src;
          inherit (pkgs) fetchurl fetchFromGitHub fetchpatch libbladeRF symlinkJoin;
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
          libbladerf-android = android.libbladerf;
          libusb-android = android.libusb;
        };

        devShells = {
          default = pkgs.mkShell {
            
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
