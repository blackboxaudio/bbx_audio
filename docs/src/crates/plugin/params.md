# Parameter Definitions

Utilities for defining plugin parameters and generating code.

## Overview

bbx_plugin provides two approaches:

1. **JSON-based** - Parse `parameters.json`
2. **Programmatic** - Define as Rust const arrays

Both generate Rust constants and C headers.

## JSON-Based Definitions

### ParamsFile

Parse a JSON file:

```rust
use bbx_plugin::ParamsFile;

let json = r#"{
    "parameters": [
        {"id": "GAIN", "name": "Gain", "type": "float", "min": -60.0, "max": 30.0, "defaultValue": 0.0},
        {"id": "MONO", "name": "Mono", "type": "boolean", "defaultValue": false}
    ]
}"#;

let params = ParamsFile::from_json(json)?;
```

### JsonParamDef

Parameter definition from JSON:

```rust
pub struct JsonParamDef {
    pub id: String,
    pub name: String,
    pub param_type: String,  // "float", "boolean", "choice"
    pub default_value: Option<serde_json::Value>,
    pub default_value_index: Option<usize>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub unit: Option<String>,
    pub midpoint: Option<f64>,
    pub interval: Option<f64>,
    pub fraction_digits: Option<u32>,
    pub choices: Option<Vec<String>>,
}
```

### Generating Code

```rust
use bbx_plugin::ParamsFile;

let params = ParamsFile::from_json(json)?;

// Generate Rust constants
let rust_code = params.generate_rust_indices();
// pub const PARAM_GAIN: usize = 0;
// pub const PARAM_MONO: usize = 1;
// pub const PARAM_COUNT: usize = 2;

// Generate C header
let c_header = params.generate_c_header();
// #define PARAM_GAIN 0
// #define PARAM_MONO 1
// #define PARAM_COUNT 2
// static const char* PARAM_IDS[PARAM_COUNT] = { "GAIN", "MONO" };
```

The C header includes a `PARAM_IDS` array for dynamic iteration over parameters in C++.

## Programmatic Definitions

### ParamDef

Define parameters as const:

```rust
use bbx_plugin::{ParamDef, ParamType};

const PARAMETERS: &[ParamDef] = &[
    ParamDef::float("GAIN", "Gain", -60.0, 30.0, 0.0),
    ParamDef::bool("MONO", "Mono", false),
    ParamDef::choice("MODE", "Mode", &["A", "B", "C"], 0),
];
```

### Constructors

```rust
// Boolean
ParamDef::bool("ID", "Name", default)

// Float with range
ParamDef::float("ID", "Name", min, max, default)

// Choice (dropdown)
ParamDef::choice("ID", "Name", &["Option1", "Option2"], default_index)
```

### Generating Code

```rust
use bbx_plugin::{generate_rust_indices_from_defs, generate_c_header_from_defs};

let rust_code = generate_rust_indices_from_defs(PARAMETERS);
let c_header = generate_c_header_from_defs(PARAMETERS);
```

## Build Script Integration

```rust
// build.rs
use std::fs;

fn main() {
    // Read parameters.json
    let json = fs::read_to_string("parameters.json").unwrap();
    let params = bbx_plugin::ParamsFile::from_json(&json).unwrap();

    // Generate Rust code
    let rust_code = params.generate_rust_indices();
    fs::write(
        format!("{}/params.rs", std::env::var("OUT_DIR").unwrap()),
        rust_code,
    ).unwrap();

    // Generate C header
    let c_header = params.generate_c_header();
    fs::write("include/bbx_params.h", c_header).unwrap();

    println!("cargo:rerun-if-changed=parameters.json");
}
```

In lib.rs:

```rust
include!(concat!(env!("OUT_DIR"), "/params.rs"));
```

## See Also

- [parameters.json Format](../../juce/parameters-json.md) - Full JSON schema
- [Code Generation](../../juce/parameters-codegen.md) - Integration details
