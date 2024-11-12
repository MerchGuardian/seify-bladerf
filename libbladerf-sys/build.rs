use std::env;
use std::path::PathBuf;

use anyhow::Context;
use bindgen::Builder;

fn main() -> anyhow::Result<()> {
    // Link shared library
    // NOTE: the lib in nix is called libbladeRF.so
    println!("cargo:rustc-link-lib=bladeRF");
    println!("cargo:rerun-if-env-changed=BLADERF_INCLUDE_PATH");

    let mut builder = Builder::default()
        .header("wrapper.h")
        .allowlist_item("(bladerf|BLADERF).*");

    let target = env::var("TARGET").expect("TARGET not set");
    // Handle android oddities
    if target.contains("android") {
        let ndk = env::var("ANDROID_NDK_HOME")
            .expect("ANDROID_NDK_HOME not set. This is required to build for android");

        let prebuilt = format!("{ndk}/toolchains/llvm/prebuilt/linux-x86_64");
        let sysroot = format!("{prebuilt}/sysroot");
        println!("Adding sysroot path: {sysroot}");
        let clang_base = format!("{prebuilt}/lib64/clang");

        let clang_version = std::fs::read_dir(&clang_base)
            .with_context(|| format!("Failed to open directory {clang_base}"))?
            .next()
            .with_context(|| format!("No enteries in {clang_base}"))?
            .with_context(|| format!("Failed to read dir: {clang_base}"))?
            .file_name();

        let clang_version = clang_version
            .to_str()
            .to_owned()
            .expect("Clang version name not utf8");
        println!("Found clang version {clang_version} in ndk: {clang_base}");

        builder = builder.clang_args(&[
            format!("--target={target}"),
            format!("--sysroot={sysroot}"),
            format!("-I{sysroot}/usr/include"),
            format!("-I{clang_base}/{clang_version}/include"),
            format!("-I{sysroot}/usr/include/{target}"),
            "-v".to_string(),
        ]);
    }

    if let Ok(path) = std::env::var("BLADERF_INCLUDE_PATH") {
        println!("Adding explicit blade rf include path: {path}");
        builder = builder.clang_arg(format!("-I{path}"));
    }

    println!("Using bindgen flags: {:?}", builder.command_line_flags());

    let bindings = builder
        .generate()
        .context("Failed to generate libbladerf bindings")?;

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .context("Failed to write bindings to OUT_DIR")?;

    Ok(())
}
