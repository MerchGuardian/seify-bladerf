{ stdenvNoCC, lib, fetchFromGitHub, autoreconfHook, libtool, ndk, cmake, pkg-config, git, bladerf-src }:
rec {
  libusb = stdenvNoCC.mkDerivation rec {
    pname = "libusb";
    version = "1.0.27";

    src = fetchFromGitHub {
      owner = "libusb";
      repo = "libusb";
      rev = "v${version}";
      hash = "sha256-OtzYxWwiba0jRK9X+4deWWDDTeZWlysEt0qMyGUarDo=";
    };

    # NDK contains all build inputs
    nativeBuildInputs = [];

    NDK = "${ndk}";

    buildPhase = ''
      cd android/jni
      export NDK="${NDK}"
      echo "NDK: $NDK"
      $NDK/ndk-build
    '';

    installPhase = ''
      mkdir -p $out/lib
      mkdir -p $out/include/libusb-1.0

      # Copy just arm v8 for now. Looks like cmake can only handle one so at a time anyway
      # NOTE: Downstream users will expect `libusb-1.0` (with dash)
      cp -v ../libs/arm64-v8a/libusb1.0.so $out/lib/libusb-1.0.so

      cp -v ../../libusb/libusb.h $out/include/libusb-1.0/
      cp -v ../../libusb/version.h $out/include/libusb-1.0/
    '';
  };
  libbladerf = stdenvNoCC.mkDerivation rec {
    pname = "libbladeRF";
    version = "master";

    src = bladerf-src;

    nativeBuildInputs = [ cmake pkg-config git ];
    buildInputs = [ libusb ];

    NDK_TOOLCHAIN = "${ndk}/toolchains/llvm/prebuilt/linux-x86_64/";

    cmakeFlags = [
      "-DCMAKE_TOOLCHAIN_FILE=${ndk}/build/cmake/android.toolchain.cmake"
      "-DANDROID_NDK=${ndk}"
      "-DANDROID_ABI=arm64-v8a"
      "-DANDROID_PLATFORM=android-21"
      "-DCMAKE_SYSTEM_NAME=Android"
      "-DCMAKE_VERBOSE_MAKEFILE=ON"
      "-DENABLE_BACKEND_LIBUSB=ON"
      "-DCMAKE_FIND_ROOT_PATH=${libusb};${ndk}/sysroot"
      "-DCMAKE_INSTALL_PREFIX=$out"
      "-DBUILD_DOCUMENTATION=OFF"
      # FIXME: HACK
      "-DVERSION_INFO_OVERRIDE=foxhunter-${builtins.substring 0 7 /*src.rev */ "DEADBEEF"}"
    ];

    preConfigure = ''
      echo "NDK_TOOLCHAIN: $NDK_TOOLCHAIN"
      echo "LIBUSB_PATH: ${libusb}"

      export CC="${NDK_TOOLCHAIN}/bin/aarch64-linux-android21-clang"
      export CXX="${NDK_TOOLCHAIN}/bin/aarch64-linux-android21-clang++"
      export AR="${NDK_TOOLCHAIN}/bin/aarch64-linux-android-ar"
      export AS="${NDK_TOOLCHAIN}/bin/aarch64-linux-android-as"
      export LD="${NDK_TOOLCHAIN}/bin/aarch64-linux-android-ld"
      export RANLIB="${NDK_TOOLCHAIN}/bin/aarch64-linux-android-ranlib"
      export STRIP="${NDK_TOOLCHAIN}/bin/aarch64-linux-android-strip"
    '';

    configurePhase = ''
      set -x
      cmake -B build -S . ${lib.escapeShellArgs cmakeFlags}
    '';

    buildPhase = ''
      set -x
      cmake --build build
    '';

    installPhase = ''
      set -x
      cmake --install build --prefix $out
    '';
  };
}
