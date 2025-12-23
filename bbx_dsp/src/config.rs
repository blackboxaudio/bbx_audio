//! Graph configuration parsing and loading from JSON.
//!
//! This module provides types for deserializing graph configurations from JSON
//! and building `Graph` instances from them.

use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    block::{BlockId, BlockType},
    blocks::{
        effectors::{
            dc_blocker::DcBlockerBlock,
            filter::{FilterBlock, FilterMode},
            gain::GainBlock,
            panner::PannerBlock,
        },
        modulators::lfo::LfoBlock,
    },
    graph::Graph,
    sample::Sample,
    waveform::Waveform,
};

/// Error type for configuration parsing.
#[derive(Debug)]
pub enum ConfigError {
    /// JSON parsing error.
    ParseError(String),
    /// Unknown block type.
    UnknownBlockType(String),
    /// Invalid parameter value.
    InvalidParameter(String),
    /// Missing required field.
    MissingField(String),
    /// Invalid connection.
    InvalidConnection(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::UnknownBlockType(t) => write!(f, "Unknown block type: {}", t),
            ConfigError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            ConfigError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ConfigError::InvalidConnection(msg) => write!(f, "Invalid connection: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Root configuration structure.
#[derive(Debug, Deserialize)]
pub struct GraphConfig {
    /// Block definitions.
    pub blocks: Vec<BlockConfig>,
    /// Audio connections between blocks.
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
    /// Modulation connections.
    #[serde(default)]
    pub modulations: Vec<ModulationConfig>,
    /// Parameter bindings for external control (JUCE).
    #[serde(default)]
    pub parameter_bindings: HashMap<String, ParameterBindingConfig>,
}

/// Configuration for a single block.
#[derive(Debug, Deserialize)]
pub struct BlockConfig {
    /// Unique block ID.
    pub id: usize,
    /// Block type (e.g., "gain", "filter", "lfo").
    #[serde(rename = "type")]
    pub block_type: String,
    /// Optional human-readable name.
    #[serde(default)]
    pub name: String,
    /// Block-specific parameters.
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

/// Configuration for an audio connection.
#[derive(Debug, Deserialize)]
pub struct ConnectionConfig {
    /// Source block ID and output index.
    pub from: (usize, usize),
    /// Destination block ID and input index.
    pub to: (usize, usize),
}

/// Configuration for a modulation connection.
#[derive(Debug, Deserialize)]
pub struct ModulationConfig {
    /// Source modulator block ID.
    pub source: usize,
    /// Target block ID.
    pub target: usize,
    /// Target parameter name.
    pub param: String,
    /// Modulation depth (-1.0 to 1.0).
    #[serde(default)]
    pub depth: f64,
}

/// Configuration for binding a parameter to external control.
#[derive(Debug, Deserialize)]
pub struct ParameterBindingConfig {
    /// Target block ID.
    pub block: usize,
    /// Target parameter name.
    pub param: String,
}

impl GraphConfig {
    /// Parse a graph configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Build a Graph from this configuration.
    pub fn build_graph<S: Sample>(
        &self,
        sample_rate: f64,
        buffer_size: usize,
        num_channels: usize,
    ) -> Result<Graph<S>, ConfigError> {
        let mut graph = Graph::new(sample_rate, buffer_size, num_channels);

        // Maps config IDs to actual BlockIds
        let mut id_map: HashMap<usize, BlockId> = HashMap::new();

        // Create all blocks
        for block_config in &self.blocks {
            let block =
                create_block::<S>(block_config, sample_rate, buffer_size, num_channels)?;
            let block_id = graph.add_block(block);
            id_map.insert(block_config.id, block_id);
        }

        // Create connections
        for conn in &self.connections {
            let from_id = id_map
                .get(&conn.from.0)
                .ok_or_else(|| ConfigError::InvalidConnection(format!("Unknown source block: {}", conn.from.0)))?;
            let to_id = id_map
                .get(&conn.to.0)
                .ok_or_else(|| ConfigError::InvalidConnection(format!("Unknown destination block: {}", conn.to.0)))?;

            graph.connect(*from_id, conn.from.1, *to_id, conn.to.1);
        }

        // Apply modulations
        for mod_config in &self.modulations {
            let _source_id = id_map
                .get(&mod_config.source)
                .ok_or_else(|| ConfigError::InvalidConnection(format!("Unknown modulator: {}", mod_config.source)))?;
            let target_id = id_map
                .get(&mod_config.target)
                .ok_or_else(|| ConfigError::InvalidConnection(format!("Unknown target: {}", mod_config.target)))?;

            // Set the modulation on the target parameter
            // This uses the existing set_parameter infrastructure
            graph.set_modulation(
                BlockId(mod_config.source),
                *target_id,
                &mod_config.param,
                S::from_f64(mod_config.depth),
            )?;
        }

        // Add output block if not present
        let has_output = self.blocks.iter().any(|b| b.block_type == "output");
        if !has_output {
            graph.add_output_block();
        }

        graph.prepare_for_playback();
        Ok(graph)
    }
}

/// Create a block from its configuration (for FFI use).
pub fn create_block_ffi(
    config: &BlockConfig,
    sample_rate: f64,
    num_channels: usize,
) -> Result<BlockType<f32>, ConfigError> {
    create_block::<f32>(config, sample_rate, 0, num_channels)
}

/// Create a block from its configuration.
fn create_block<S: Sample>(
    config: &BlockConfig,
    _sample_rate: f64,
    _buffer_size: usize,
    num_channels: usize,
) -> Result<BlockType<S>, ConfigError> {
    match config.block_type.to_lowercase().as_str() {
        // I/O blocks
        "input" => {
            // Input blocks are passthrough - for effects, audio comes from external source
            // We don't create a real input block; the graph handles external input
            Ok(BlockType::Gain(GainBlock::unity(num_channels)))
        }

        "output" => {
            use crate::blocks::io::output::OutputBlock;
            Ok(BlockType::Output(OutputBlock::new(num_channels)))
        }

        // Effect blocks
        "dc_blocker" | "dcblocker" => {
            let coefficient = get_param_f64(&config.params, "coefficient", 0.995);
            Ok(BlockType::DcBlocker(DcBlockerBlock::new(
                S::from_f64(coefficient),
                num_channels,
            )))
        }

        "filter" => {
            let cutoff = get_param_f64(&config.params, "cutoff", 1000.0);
            let resonance = get_param_f64(&config.params, "resonance", 0.707);
            let mode_str = get_param_str(&config.params, "mode", "lowpass");
            let mode = FilterMode::from_str(&mode_str).unwrap_or(FilterMode::LowPass);

            Ok(BlockType::Filter(FilterBlock::new(
                S::from_f64(cutoff),
                S::from_f64(resonance),
                mode,
                num_channels,
            )))
        }

        "gain" => {
            let level = get_param_f64(&config.params, "level", 0.0);
            let smoothing_ms = get_param_f64(&config.params, "smoothing_ms", 20.0);

            Ok(BlockType::Gain(GainBlock::new(
                S::from_f64(level),
                smoothing_ms,
                num_channels,
            )))
        }

        "panner" | "pan" => {
            let position = get_param_f64(&config.params, "position", 0.0);
            Ok(BlockType::Panner(PannerBlock::new(S::from_f64(position))))
        }

        // Modulator blocks
        "lfo" => {
            let frequency = get_param_f64(&config.params, "frequency", 1.0);
            let depth = get_param_f64(&config.params, "depth", 1.0);
            let waveform_str = get_param_str(&config.params, "waveform", "sine");
            let waveform = match waveform_str.to_lowercase().as_str() {
                "sine" => Waveform::Sine,
                "triangle" | "tri" => Waveform::Triangle,
                "saw" | "sawtooth" => Waveform::Sawtooth,
                "square" => Waveform::Square,
                _ => Waveform::Sine,
            };

            Ok(BlockType::Lfo(LfoBlock::new(
                S::from_f64(frequency),
                S::from_f64(depth),
                waveform,
                None,
            )))
        }

        _ => Err(ConfigError::UnknownBlockType(config.block_type.clone())),
    }
}

/// Get a parameter as f64, with a default value.
fn get_param_f64(params: &HashMap<String, serde_json::Value>, key: &str, default: f64) -> f64 {
    params
        .get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(default)
}

/// Get a parameter as string, with a default value.
fn get_param_str(params: &HashMap<String, serde_json::Value>, key: &str, default: &str) -> String {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let json = r#"{
            "blocks": [
                { "id": 0, "type": "gain", "name": "input_gain", "params": { "level": -6.0 } },
                { "id": 1, "type": "output", "name": "out" }
            ],
            "connections": [
                { "from": [0, 0], "to": [1, 0] }
            ]
        }"#;

        let config = GraphConfig::from_json(json).unwrap();
        assert_eq!(config.blocks.len(), 2);
        assert_eq!(config.connections.len(), 1);
    }

    #[test]
    fn test_parse_full_config() {
        let json = r#"{
            "blocks": [
                { "id": 0, "type": "input", "name": "audio_in" },
                { "id": 1, "type": "dc_blocker", "params": { "coefficient": 0.995 } },
                { "id": 2, "type": "gain", "params": { "level": 0.0, "smoothing_ms": 20.0 } },
                { "id": 3, "type": "panner", "params": { "position": 0.0 } },
                { "id": 4, "type": "lfo", "params": { "frequency": 1.0, "waveform": "sine" } },
                { "id": 5, "type": "output", "name": "audio_out" }
            ],
            "connections": [
                { "from": [0, 0], "to": [1, 0] },
                { "from": [1, 0], "to": [2, 0] },
                { "from": [2, 0], "to": [3, 0] },
                { "from": [3, 0], "to": [5, 0] }
            ],
            "modulations": [
                { "source": 4, "target": 2, "param": "level", "depth": 0.5 }
            ],
            "parameter_bindings": {
                "GAIN": { "block": 2, "param": "level" },
                "PAN": { "block": 3, "param": "position" }
            }
        }"#;

        let config = GraphConfig::from_json(json).unwrap();
        assert_eq!(config.blocks.len(), 6);
        assert_eq!(config.connections.len(), 4);
        assert_eq!(config.modulations.len(), 1);
        assert_eq!(config.parameter_bindings.len(), 2);
    }

    #[test]
    fn test_build_graph() {
        let json = r#"{
            "blocks": [
                { "id": 0, "type": "gain", "params": { "level": 0.0 } },
                { "id": 1, "type": "output" }
            ],
            "connections": [
                { "from": [0, 0], "to": [1, 0] }
            ]
        }"#;

        let config = GraphConfig::from_json(json).unwrap();
        let graph: Graph<f32> = config.build_graph(44100.0, 512, 2).unwrap();

        // Graph should have the blocks
        assert!(graph.context().sample_rate == 44100.0);
    }

    #[test]
    fn test_unknown_block_type() {
        let json = r#"{
            "blocks": [
                { "id": 0, "type": "unknown_type" }
            ]
        }"#;

        let config = GraphConfig::from_json(json).unwrap();
        let result: Result<Graph<f32>, _> = config.build_graph(44100.0, 512, 2);
        assert!(result.is_err());
    }
}
