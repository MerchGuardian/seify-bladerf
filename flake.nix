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
          overlays = [
            (import rust-overlay)
          ];
          inherit system;
        };
        rust-pkgs = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
        xa4-bitstream = pkgs.fetchurl {
          # See: https://www.nuand.com/fpga_images/
          url = "https://www.nuand.com/fpga/v0.15.3/hostedxA4.rbf";
          # nix hash to-sri --type sha256 eaa415650b6b00e75f1e28c2a860d8f90577b1f4648b2f7b88a65745f464a6db
          sha256 = "sha256-6qQVZQtrAOdfHijCqGDY+QV3sfRkiy97iKZXRfRkpts=";
        };
        fx3-firmware = pkgs.fetchurl {
          # See: https://www.nuand.com/fx3_images/
          # Annoyingly there is no explicit url for 2.5.0, so one day he will update and the hash check will fail...
          url = "https://www.nuand.com/fx3/bladeRF_fw_latest.img";
          sha256 = "sha256-Zw0cp6ocYAfrCZADUcOqmX5L4xbbwKL8FTKpCNAswKk=";
        };
      in with pkgs; {
        devShells = {
          default = pkgs.mkShell {
            LIBCLANG_PATH="${llvmPackages_14.clang.cc.lib}/lib";
            BLADERF_INCLUDE_PATH="${libbladeRF}/include";
            RUSTFLAGS="-L ${libbladeRF}/lib";
            BLADERF_RS_FPGA_BITSTREAM_PATH="${xa4-bitstream}";
            BLADERF_RS_FX3_FIRMWARE_PATH="${fx3-firmware}";

            packages = [
              rust-pkgs
            ];
          };
        };
      });
}
