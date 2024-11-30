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
