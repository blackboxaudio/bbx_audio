//! Build script for bbx_daisy.
//!
//! Copies the memory.x linker script to the output directory for ARM targets.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();

    if target.starts_with("thumbv7em") {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let memory_x = include_bytes!("memory.x");

        fs::write(out_dir.join("memory.x"), memory_x).expect("Failed to write memory.x");

        println!("cargo:rustc-link-search={}", out_dir.display());
        println!("cargo:rerun-if-changed=memory.x");
        println!("cargo:rerun-if-changed=build.rs");
    }
}
