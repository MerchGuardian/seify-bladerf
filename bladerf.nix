{ lib, stdenv, pkg-config, fetchurl, fetchpatch, bladerf-src, symlinkJoin, ncurses, cmake, git, doxygen, help2man, tecla, libusb1, udev }:
rec {
  xa4-bitstream = fetchurl {
    # See: https://www.nuand.com/fpga_images/
    url = "https://www.nuand.com/fpga/v0.15.3/hostedxA4.rbf";
    sha256 = "sha256-6qQVZQtrAOdfHijCqGDY+QV3sfRkiy97iKZXRfRkpts=";
  };
  fx3-firmware = fetchurl {
    # See: https://www.nuand.com/fx3_images/
    url = "https://www.nuand.com/fx3/bladeRF_fw_v2.4.0.img";
    sha256 = "sha256-Zw0cp6ocYAfrCZADUcOqmX5L4xbbwKL8FTKpCNAswKk=";
  };
  libbladerf = stdenv.mkDerivation rec {
    pname = "libbladeRF";
    version = "master";

    src = bladerf-src;

    nativeBuildInputs = [ cmake pkg-config git doxygen help2man ];
    # ncurses used due to https://github.com/Nuand/bladeRF/blob/ab4fc672c8bab4f8be34e8917d3f241b1d52d0b8/host/utilities/bladeRF-cli/CMakeLists.txt#L208
    buildInputs = [ tecla libusb1 ncurses ]
      ++ lib.optionals stdenv.hostPlatform.isLinux [ udev ];

    # Fixup shebang
    prePatch = "patchShebangs host/utilities/bladeRF-cli/src/cmd/doc/generate.bash";

    # Let us avoid nettools as a dependency.
    postPatch = ''
      sed -i 's/$(hostname)/hostname/' host/utilities/bladeRF-cli/src/cmd/doc/generate.bash
    '';

    # bladeRF-fsk (cli) support for BladeRf 2.0 micro
    patches = [
      (fetchpatch {
        url = "https://github.com/Nuand/bladeRF/commit/2db141bf6225abd3cc51e64da14461739bab35dc.patch";
        sha256 = "sha256-UHRw7HkjYFwRbGQki5l5vSaxLbhYFrWsN6ZEYSjYB2s=";
      })
    ];

    cmakeFlags = [
      "-DBUILD_DOCUMENTATION=ON"
      # FIXME: HACK
      "-DVERSION_INFO_OVERRIDE=foxhunter-${builtins.substring 0 7 /*src.rev */ "DEADBEEF"}"
    ] ++ lib.optionals stdenv.hostPlatform.isLinux [
      "-DUDEV_RULES_PATH=etc/udev/rules.d"
      "-DINSTALL_UDEV_RULES=ON"
      "-DBLADERF_GROUP=bladerf"
    ] ++ lib.optionals stdenv.hostPlatform.isDarwin [
      "-DCMAKE_C_FLAGS=-Wno-error=format"
    ];

    env = lib.optionalAttrs stdenv.cc.isClang {
      NIX_CFLAGS_COMPILE = "-Wno-error=unused-but-set-variable";
    };

    hardeningDisable = [ "fortify" ];

    meta = with lib; {
      homepage = "https://nuand.com/libbladeRF-doc";
      description = "Supporting library of the BladeRF SDR opensource hardware";
      license = licenses.lgpl21;
      platforms = platforms.unix;
    };
  };
}
