// Build script for bbx_ffi
//
// Parses template-plugin/parameters.json to generate parameter defaults
// at compile time. The C header (include/bbx_ffi.h) is maintained manually.

use serde::Deserialize;
use std::{env, fs, path::Path};

#[derive(Deserialize)]
struct ParamDef {
    id: String,
    #[serde(rename = "defaultValue")]
    default_value: Option<serde_json::Value>,
    #[serde(rename = "defaultValueIndex")]
    default_value_index: Option<i32>,
}

#[derive(Deserialize)]
struct ParamsFile {
    parameters: Vec<ParamDef>,
}

fn main() {
    // Trigger rebuild if source files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/handle.rs");
    println!("cargo:rerun-if-changed=src/audio.rs");
    println!("cargo:rerun-if-changed=src/params.rs");
    println!("cargo:rerun-if-changed=include/bbx_ffi.h");

    // Parse parameters.json from template-plugin
    let params_path = Path::new("../template-plugin/parameters.json");

    if params_path.exists() {
        println!("cargo:rerun-if-changed={}", params_path.display());

        let json = fs::read_to_string(params_path).expect("Failed to read parameters.json");
        let params: ParamsFile = serde_json::from_str(&json).expect("Invalid parameters.json");

        generate_param_defaults(&params);
    } else {
        // Generate fallback defaults if parameters.json doesn't exist
        generate_fallback_defaults();
    }
}

fn generate_param_defaults(params: &ParamsFile) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("param_defaults.rs");

    let mut invert_left = false;
    let mut invert_right = false;
    let mut channel_mode = 0i32;
    let mut mono = false;
    let mut gain_db = 0.0f32;
    let mut pan = 0.0f32;
    let mut dc_offset = false;

    for param in &params.parameters {
        match param.id.as_str() {
            "INVERT_LEFT_CHANNEL" => {
                invert_left = param
                    .default_value
                    .as_ref()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
            "INVERT_RIGHT_CHANNEL" => {
                invert_right = param
                    .default_value
                    .as_ref()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
            "CHANNEL_CONFIGURATION" => {
                channel_mode = param.default_value_index.unwrap_or(0);
            }
            "MONO" => {
                mono = param
                    .default_value
                    .as_ref()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
            "GAIN" => {
                gain_db = param
                    .default_value
                    .as_ref()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
            }
            "PAN" => {
                pan = param
                    .default_value
                    .as_ref()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
            }
            "DC_OFFSET" => {
                dc_offset = param
                    .default_value
                    .as_ref()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
            _ => {}
        }
    }

    let code = format!(
        r#"ParamDefaults {{
    invert_left: {invert_left},
    invert_right: {invert_right},
    channel_mode: {channel_mode},
    mono: {mono},
    gain_db: {gain_db}f32,
    pan: {pan}f32,
    dc_offset: {dc_offset},
}}"#
    );

    fs::write(&dest_path, code).expect("Failed to write param_defaults.rs");
}

fn generate_fallback_defaults() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("param_defaults.rs");

    let code = r#"ParamDefaults {
    invert_left: false,
    invert_right: false,
    channel_mode: 0,
    mono: false,
    gain_db: 0.0f32,
    pan: 0.0f32,
    dc_offset: false,
}"#;

    fs::write(&dest_path, code).expect("Failed to write param_defaults.rs");
}
