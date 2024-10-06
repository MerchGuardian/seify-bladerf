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
      in with pkgs; {
        devShells = {
          default = pkgs.mkShell {
            LIBCLANG_PATH="${llvmPackages_14.clang.cc.lib}/lib";
            BLADERF_INCLUDE_PATH="${libbladeRF}/include";
            RUSTFLAGS="-L ${libbladeRF}/lib";

            packages = [
              rust-pkgs
            ];
          };
        };
      });
}
