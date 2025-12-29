//! Build script for bbx_ffi crate.
//!
//! Emits the include directory path so dependent crates can find the C/C++ headers.
//! This enables automatic header discovery when bbx_ffi is used as a path, git, or
//! crates.io dependency.

use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let include_dir = manifest_dir.join("include");

    // Emit include path - this becomes DEP_BBX_FFI_INCLUDE for dependents
    println!("cargo:include={}", include_dir.display());

    // Re-run if headers change
    println!("cargo:rerun-if-changed=include/bbx_ffi.h");
    println!("cargo:rerun-if-changed=include/bbx_wrapper.h");
}
