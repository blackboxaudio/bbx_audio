# Rust Crate Configuration

Configure your Rust crate for FFI integration with JUCE.

## Cargo.toml

Create a `dsp/Cargo.toml`:

```toml
[package]
name = "dsp"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["staticlib", "cdylib"]

[dependencies]
bbx_plugin = "0.1"
```

### Crate Types

- **`staticlib`** - Static library for linking into C++ (recommended)
- **`cdylib`** - Dynamic library (alternative approach)

Using `staticlib` is recommended as it bundles all Rust code into a single library that links cleanly with the plugin.

## Using Git Dependencies

For the latest development version:

```toml
[dependencies]
bbx_plugin = { git = "https://github.com/blackboxaudio/bbx_audio" }
```

## lib.rs Structure

Your `dsp/src/lib.rs` should:

1. Import the necessary types
2. Define your DSP struct
3. Implement `PluginDsp`
4. Call the FFI macro

```rust
use bbx_plugin::{PluginDsp, DspContext, bbx_plugin_ffi};

pub struct PluginGraph {
    // Your DSP blocks
}

impl Default for PluginGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginDsp for PluginGraph {
    fn new() -> Self {
        PluginGraph {
            // Initialize blocks
        }
    }

    fn prepare(&mut self, context: &DspContext) {
        // Called when audio specs change
    }

    fn reset(&mut self) {
        // Clear DSP state
    }

    fn apply_parameters(&mut self, params: &[f32]) {
        // Map parameter values to blocks
    }

    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        context: &DspContext,
    ) {
        // Process audio
    }
}

// Generate FFI exports
bbx_plugin_ffi!(PluginGraph);
```

## FFI Headers

Copy the following headers to `dsp/include/`:

- `bbx_ffi.h` - From `bbx_plugin/include/bbx_ffi.h`
- `bbx_graph.h` - From `bbx_plugin/include/bbx_graph.h`

These headers provide the C and C++ interfaces for your JUCE code.
