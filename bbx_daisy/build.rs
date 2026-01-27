//! Build script for bbx_daisy.
//!
//! - Copies the memory.x linker script to the output directory for ARM targets
//! - Validates that only one product feature is enabled at build time

use std::{env, fs, path::PathBuf};

fn main() {
    validate_product_features();

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

fn validate_product_features() {
    // Validate only one product feature is enabled
    let product_features = [
        ("seed", cfg!(feature = "seed")),
        ("seed_1_1", cfg!(feature = "seed_1_1")),
        ("seed_1_2", cfg!(feature = "seed_1_2")),
        ("pod", cfg!(feature = "pod")),
        ("patch_sm", cfg!(feature = "patch_sm")),
        ("patch_init", cfg!(feature = "patch_init")),
        ("patch", cfg!(feature = "patch")),
        ("field", cfg!(feature = "field")),
    ];

    let enabled: Vec<&str> = product_features
        .iter()
        .filter_map(|(name, enabled)| if *enabled { Some(*name) } else { None })
        .collect();

    // Check for multiple features (excluding aliases)
    let mut unique_products = Vec::new();
    for &feature in &enabled {
        match feature {
            "patch" => {
                if !unique_products.contains(&"patch_sm") {
                    unique_products.push("patch_sm");
                }
            }
            "patch_init" => {
                if !unique_products.contains(&"patch_sm") {
                    unique_products.push("patch_sm");
                }
            }
            _ => unique_products.push(feature),
        }
    }

    if unique_products.len() > 1 {
        panic!(
            "ERROR: Multiple bbx_daisy product features enabled: {:?}\n\
             Only one product feature can be enabled at a time.\n\
             Use --no-default-features and specify exactly one feature.\n\
             Valid features: seed, seed_1_1, seed_1_2, pod, patch_sm, patch_init, patch",
            enabled
        );
    }

    if enabled.is_empty() {
        panic!(
            "ERROR: No bbx_daisy product feature enabled.\n\
             Specify one of: seed, seed_1_1, seed_1_2, pod, patch_sm, patch_init, patch\n\
             Example: cargo build --features pod"
        );
    }

    // Block unimplemented field feature
    if cfg!(feature = "field") {
        panic!(
            "ERROR: The 'field' feature is not yet implemented.\n\
             The Daisy Field board is not currently supported by bbx_daisy.\n\
             Please open an issue at https://github.com/bbx-audio/bbx_audio/issues \
             if you need Daisy Field support."
        );
    }
}
