# Parameter Code Generation

Generate consistent parameter indices for Rust and C++ from a single source.

## Build Script Integration

The recommended approach is to generate code at build time using a `build.rs` script.

### Using parameters.json

```rust
// build.rs
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Read parameters.json
    let json = fs::read_to_string("parameters.json")
        .expect("Failed to read parameters.json");

    let params: serde_json::Value = serde_json::from_str(&json)
        .expect("Failed to parse parameters.json");

    // Generate Rust constants
    let mut rust_code = String::from("// Auto-generated - DO NOT EDIT\n\n");
    if let Some(parameters) = params["parameters"].as_array() {
        for (index, param) in parameters.iter().enumerate() {
            let id = param["id"].as_str().unwrap();
            rust_code.push_str(&format!("pub const PARAM_{}: usize = {};\n", id, index));
        }
        rust_code.push_str(&format!("\npub const PARAM_COUNT: usize = {};\n", parameters.len()));
    }

    // Write to OUT_DIR
    let dest_path = Path::new(&out_dir).join("params.rs");
    fs::write(&dest_path, rust_code).unwrap();

    // Generate C header
    let mut c_header = String::from("/* Auto-generated - DO NOT EDIT */\n\n");
    c_header.push_str("#ifndef BBX_PARAMS_H\n#define BBX_PARAMS_H\n\n");
    if let Some(parameters) = params["parameters"].as_array() {
        for (index, param) in parameters.iter().enumerate() {
            let id = param["id"].as_str().unwrap();
            c_header.push_str(&format!("#define PARAM_{} {}\n", id, index));
        }
        c_header.push_str(&format!("\n#define PARAM_COUNT {}\n", parameters.len()));
    }
    c_header.push_str("\n#endif /* BBX_PARAMS_H */\n");

    // Write to include directory
    fs::write("include/bbx_params.h", c_header).unwrap();

    println!("cargo:rerun-if-changed=parameters.json");
}
```

### Including Generated Code

In your `lib.rs`:

```rust
// Include the generated parameter constants
include!(concat!(env!("OUT_DIR"), "/params.rs"));
```

### Using ParamsFile API

Alternatively, use the built-in API:

```rust
// build.rs
use bbx_plugin::ParamsFile;
use std::fs;

fn main() {
    let json = fs::read_to_string("parameters.json").unwrap();
    let params = ParamsFile::from_json(&json).unwrap();

    // Generate Rust code
    let rust_code = params.generate_rust_indices();
    fs::write(format!("{}/params.rs", std::env::var("OUT_DIR").unwrap()), rust_code).unwrap();

    // Generate C header
    let c_header = params.generate_c_header();
    fs::write("include/bbx_params.h", c_header).unwrap();

    println!("cargo:rerun-if-changed=parameters.json");
}
```

## Using Programmatic Definitions

For compile-time definitions:

```rust
// build.rs
use bbx_plugin::{ParamDef, generate_rust_indices_from_defs, generate_c_header_from_defs};
use std::fs;

const PARAMETERS: &[ParamDef] = &[
    ParamDef::float("GAIN", "Gain", -60.0, 30.0, 0.0),
    ParamDef::bool("MONO", "Mono", false),
];

fn main() {
    let rust_code = generate_rust_indices_from_defs(PARAMETERS);
    fs::write(format!("{}/params.rs", std::env::var("OUT_DIR").unwrap()), rust_code).unwrap();

    let c_header = generate_c_header_from_defs(PARAMETERS);
    fs::write("include/bbx_params.h", c_header).unwrap();
}
```

## CMake Integration

Include the generated header in CMake:

```cmake
# Ensure Rust build runs first (Corrosion handles this)
corrosion_import_crate(MANIFEST_PATH dsp/Cargo.toml)

# Include the generated header directory
target_include_directories(${PLUGIN_TARGET} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/dsp/include)
```

## Verification

Add a test to verify Rust and C++ constants match:

```rust
#[test]
fn test_param_indices_match() {
    // If this compiles, indices are in sync
    assert_eq!(PARAM_COUNT, 7);
    assert!(PARAM_GAIN < PARAM_COUNT);
    assert!(PARAM_PAN < PARAM_COUNT);
}
```
