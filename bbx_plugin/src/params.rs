//! Parameter definition and code generation utilities.
//!
//! This module provides two ways to define plugin parameters:
//!
//! 1. **JSON-based**: Parse a `parameters.json` file using [`ParamsFile`]
//! 2. **Programmatic**: Define parameters as const arrays using [`ParamDef`]
//!
//! Both approaches can generate Rust and C++ code for parameter indices.

use serde::Deserialize;

// ============================================================================
// Programmatic Parameter Definition (const arrays)
// ============================================================================

/// Parameter type variants for programmatic declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
    /// Boolean parameter (on/off toggle).
    Bool { default: bool },

    /// Float parameter with range.
    Float { min: f64, max: f64, default: f64 },

    /// Choice parameter (dropdown/enum).
    Choice {
        choices: &'static [&'static str],
        default_index: usize,
    },
}

/// Parameter definition for programmatic declaration.
///
/// Use the const fn constructors to build parameter definitions:
///
/// ```ignore
/// const PARAMETERS: &[ParamDef] = &[
///     ParamDef::float("GAIN", "Gain", -60.0, 30.0, 0.0),
///     ParamDef::bool("MONO", "Mono", false),
///     ParamDef::choice("MODE", "Mode", &["A", "B", "C"], 0),
/// ];
/// ```
#[derive(Debug, Clone)]
pub struct ParamDef {
    /// Parameter ID (used for code generation, e.g., "GAIN" → PARAM_GAIN).
    pub id: &'static str,
    /// Display name shown in the UI.
    pub name: &'static str,
    /// Parameter type and configuration.
    pub param_type: ParamType,
}

impl ParamDef {
    /// Create a boolean parameter.
    pub const fn bool(id: &'static str, name: &'static str, default: bool) -> Self {
        Self {
            id,
            name,
            param_type: ParamType::Bool { default },
        }
    }

    /// Create a float parameter with range.
    pub const fn float(id: &'static str, name: &'static str, min: f64, max: f64, default: f64) -> Self {
        Self {
            id,
            name,
            param_type: ParamType::Float { min, max, default },
        }
    }

    /// Create a choice parameter with options.
    pub const fn choice(
        id: &'static str,
        name: &'static str,
        choices: &'static [&'static str],
        default_index: usize,
    ) -> Self {
        Self {
            id,
            name,
            param_type: ParamType::Choice { choices, default_index },
        }
    }
}

/// Generate Rust parameter index constants from a const array of ParamDefs.
///
/// Output example:
/// ```text
/// pub const PARAM_GAIN: usize = 0;
/// pub const PARAM_PAN: usize = 1;
/// pub const PARAM_COUNT: usize = 2;
/// ```
pub fn generate_rust_indices_from_defs(params: &[ParamDef]) -> String {
    let mut code = String::from("// Auto-generated parameter indices - DO NOT EDIT\n\n");

    for (index, param) in params.iter().enumerate() {
        code.push_str(&format!("pub const PARAM_{}: usize = {};\n", param.id, index));
    }

    code.push_str(&format!(
        "\n#[allow(dead_code)]\npub const PARAM_COUNT: usize = {};\n",
        params.len()
    ));

    code
}

/// Generate C header with parameter index constants from a const array of ParamDefs.
///
/// Output example:
/// ```text
/// #define PARAM_GAIN 0
/// #define PARAM_PAN 1
/// #define PARAM_COUNT 2
/// static const char* PARAM_IDS[PARAM_COUNT] = { "GAIN", "PAN" };
/// ```
pub fn generate_c_header_from_defs(params: &[ParamDef]) -> String {
    let mut content = String::new();
    content.push_str("/* Auto-generated parameter indices - DO NOT EDIT */\n\n");
    content.push_str("#ifndef BBX_PARAMS_H\n");
    content.push_str("#define BBX_PARAMS_H\n\n");

    for (index, param) in params.iter().enumerate() {
        content.push_str(&format!("#define PARAM_{} {}\n", param.id, index));
    }

    content.push_str(&format!("\n#define PARAM_COUNT {}\n\n", params.len()));

    // Generate PARAM_IDS array for dynamic iteration
    if !params.is_empty() {
        content.push_str("static const char* PARAM_IDS[PARAM_COUNT] = {\n");
        for (i, param) in params.iter().enumerate() {
            let comma = if i < params.len() - 1 { "," } else { "" };
            content.push_str(&format!("    \"{}\"{}\n", param.id, comma));
        }
        content.push_str("};\n\n");
    }

    content.push_str("#endif /* BBX_PARAMS_H */\n");

    content
}

// ============================================================================
// JSON-based Parameter Definition (parameters.json)
// ============================================================================

/// JSON parameter definition (for parsing parameters.json).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonParamDef {
    /// Parameter ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Parameter type: "boolean", "float", or "choice".
    #[serde(rename = "type")]
    pub param_type: String,
    /// Default value for boolean/float parameters.
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
    /// Default index for choice parameters.
    #[serde(default)]
    pub default_value_index: Option<usize>,
    /// Minimum value for float parameters.
    #[serde(default)]
    pub min: Option<f64>,
    /// Maximum value for float parameters.
    #[serde(default)]
    pub max: Option<f64>,
    /// Unit label for float parameters (e.g., "dB").
    #[serde(default)]
    pub unit: Option<String>,
    /// Midpoint for skewed float parameters.
    #[serde(default)]
    pub midpoint: Option<f64>,
    /// Step interval for float parameters.
    #[serde(default)]
    pub interval: Option<f64>,
    /// Number of decimal places to display.
    #[serde(default)]
    pub fraction_digits: Option<u32>,
    /// Available choices for choice parameters.
    #[serde(default)]
    pub choices: Option<Vec<String>>,
}

/// Container for a parameters.json file.
#[derive(Debug, Clone, Deserialize)]
pub struct ParamsFile {
    /// List of parameter definitions.
    pub parameters: Vec<JsonParamDef>,
}

impl ParamsFile {
    /// Parse a parameters.json file from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Generate Rust parameter index constants.
    ///
    /// Output example:
    /// ```text
    /// pub const PARAM_GAIN: usize = 0;
    /// pub const PARAM_PAN: usize = 1;
    /// pub const PARAM_COUNT: usize = 2;
    /// ```
    pub fn generate_rust_indices(&self) -> String {
        let mut code = String::from("// Auto-generated from parameters.json - DO NOT EDIT\n\n");

        for (index, param) in self.parameters.iter().enumerate() {
            code.push_str(&format!("pub const PARAM_{}: usize = {};\n", param.id, index));
        }

        code.push_str(&format!(
            "\n#[allow(dead_code)]\npub const PARAM_COUNT: usize = {};\n",
            self.parameters.len()
        ));

        code
    }

    /// Generate C header with parameter index constants.
    ///
    /// Output example:
    /// ```text
    /// #define PARAM_GAIN 0
    /// #define PARAM_PAN 1
    /// #define PARAM_COUNT 2
    /// static const char* PARAM_IDS[PARAM_COUNT] = { "GAIN", "PAN" };
    /// ```
    pub fn generate_c_header(&self) -> String {
        let mut content = String::new();
        content.push_str("/* Auto-generated from parameters.json - DO NOT EDIT */\n\n");
        content.push_str("#ifndef BBX_PARAMS_H\n");
        content.push_str("#define BBX_PARAMS_H\n\n");

        for (index, param) in self.parameters.iter().enumerate() {
            content.push_str(&format!("#define PARAM_{} {}\n", param.id, index));
        }

        content.push_str(&format!("\n#define PARAM_COUNT {}\n\n", self.parameters.len()));

        // Generate PARAM_IDS array for dynamic iteration
        if !self.parameters.is_empty() {
            content.push_str("static const char* PARAM_IDS[PARAM_COUNT] = {\n");
            for (i, param) in self.parameters.iter().enumerate() {
                let comma = if i < self.parameters.len() - 1 { "," } else { "" };
                content.push_str(&format!("    \"{}\"{}\n", param.id, comma));
            }
            content.push_str("};\n\n");
        }

        content.push_str("#endif /* BBX_PARAMS_H */\n");

        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_def_constructors() {
        let bool_param = ParamDef::bool("MONO", "Mono", false);
        assert_eq!(bool_param.id, "MONO");
        assert_eq!(bool_param.param_type, ParamType::Bool { default: false });

        let float_param = ParamDef::float("GAIN", "Gain", -60.0, 30.0, 0.0);
        assert_eq!(float_param.id, "GAIN");

        let choice_param = ParamDef::choice("MODE", "Mode", &["A", "B"], 0);
        assert_eq!(choice_param.id, "MODE");
    }

    #[test]
    fn test_generate_rust_indices_from_defs() {
        const PARAMS: &[ParamDef] = &[
            ParamDef::float("GAIN", "Gain", -60.0, 30.0, 0.0),
            ParamDef::bool("MONO", "Mono", false),
        ];

        let code = generate_rust_indices_from_defs(PARAMS);
        assert!(code.contains("pub const PARAM_GAIN: usize = 0;"));
        assert!(code.contains("pub const PARAM_MONO: usize = 1;"));
        assert!(code.contains("#[allow(dead_code)]"));
        assert!(code.contains("pub const PARAM_COUNT: usize = 2;"));
    }

    #[test]
    fn test_generate_c_header_from_defs() {
        const PARAMS: &[ParamDef] = &[
            ParamDef::float("GAIN", "Gain", -60.0, 30.0, 0.0),
            ParamDef::bool("MONO", "Mono", false),
        ];

        let header = generate_c_header_from_defs(PARAMS);
        assert!(header.contains("#define PARAM_GAIN 0"));
        assert!(header.contains("#define PARAM_MONO 1"));
        assert!(header.contains("#define PARAM_COUNT 2"));
        assert!(header.contains("static const char* PARAM_IDS[PARAM_COUNT]"));
        assert!(header.contains("\"GAIN\""));
        assert!(header.contains("\"MONO\""));
    }

    #[test]
    fn test_params_file_from_json() {
        let json = r#"{
            "parameters": [
                {"id": "GAIN", "name": "Gain", "type": "float", "min": -60.0, "max": 30.0, "defaultValue": 0.0},
                {"id": "MONO", "name": "Mono", "type": "boolean", "defaultValue": false}
            ]
        }"#;

        let params = ParamsFile::from_json(json).unwrap();
        assert_eq!(params.parameters.len(), 2);
        assert_eq!(params.parameters[0].id, "GAIN");
        assert_eq!(params.parameters[1].id, "MONO");
    }

    #[test]
    fn test_params_file_generate_indices() {
        let json = r#"{"parameters": [{"id": "GAIN", "name": "Gain", "type": "float"}]}"#;
        let params = ParamsFile::from_json(json).unwrap();

        let rust_code = params.generate_rust_indices();
        assert!(rust_code.contains("pub const PARAM_GAIN: usize = 0;"));

        let c_header = params.generate_c_header();
        assert!(c_header.contains("#define PARAM_GAIN 0"));
        assert!(c_header.contains("static const char* PARAM_IDS[PARAM_COUNT]"));
        assert!(c_header.contains("\"GAIN\""));
    }
}
