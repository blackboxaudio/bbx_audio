// Build script for bbx_ffi
//
// Note: The C header (include/bbx_ffi.h) is maintained manually because
// cbindgen 0.27 doesn't fully support Rust 2024's #[unsafe(no_mangle)] attribute.
// When cbindgen adds support, this can be updated to auto-generate the header.

fn main() {
    // Trigger rebuild if source files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/handle.rs");
    println!("cargo:rerun-if-changed=src/audio.rs");
    println!("cargo:rerun-if-changed=src/params.rs");
    println!("cargo:rerun-if-changed=include/bbx_ffi.h");
}
