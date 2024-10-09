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
        bladerf = import ./bladerf.nix {
          inherit (pkgs) fetchurl fetchFromGitHub fetchpatch libbladeRF;
        };
      in with pkgs; {
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
