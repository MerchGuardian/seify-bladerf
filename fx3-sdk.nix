{ lib, requireFile, mkDerivation, autoPatchelfHook, stdenv, gnutar, libusb1 }:
let
  version = "1.3.5";
  name = "ezusbfx3sdk_${version}_Linux_x32-x64";
in mkDerivation {
  inherit name;

  src = requireFile {
    url = "hhttps://softwaretools.infineon.com/tools/com.ifx.tb.tool.ezusbfx3sdk";
    name = "${name}.tar.gz";
    sha256 = "sha256-yokb+JgYAC/abGV//oDxCnpFL2a69TiBuzSQoRHmTZA=";
  };

  nativeBuildInputs = [
    # autoPatchelfHook
  ];

  buildInputs = [
    stdenv.cc.cc.lib
  ];

  unpackPhase = ''
    mkdir ./tmp
    ${gnutar}/bin/tar -xzf $src -C ./tmp
    ${gnutar}/bin/tar -xzf ./tmp/fx3_firmware_linux.tar.gz -C ./tmp
    ${gnutar}/bin/tar -xzf ./tmp/cyusb_linux_1.0.5.tar.gz -C ./tmp
    ${gnutar}/bin/tar -xzf ./tmp/ARM_GCC.tar.gz -C ./tmp

    mv ./tmp/ $out
    # Links to a bunch of stuff that requires GUI, rm since we dont need this
    rm -rf $out/cyfx3sdk/JTAG
    rm -rf $out/cyusb_linux_1.0.5
    mkdir -p $out/bin

    echo $out
  '';

  postPatch = ''
    # addAutoPatchelfSearchPath $out/cyfx3sdk/JTAG/OpenOCD/Linux/x64/
    # patchelf --set-interpreter $(cat $NIX_CC/nix-support/dynamic-linker) $out/bin/arm-none-eabi-gcc
  '';

  installPhase = ''
    echo $unpackPhase
    runHook preInstall
    runHook postInstall
  '';

  meta = with lib; {
    platforms = platforms.linux;
  };
}
