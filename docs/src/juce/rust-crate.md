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

## FFI Headers

Copy these headers to `dsp/include/`:

- `bbx_ffi.h` - C FFI function declarations
- `bbx_graph.h` - C++ RAII wrapper class

Get them from the [bbx_plugin include directory](https://github.com/blackboxaudio/bbx_audio/tree/main/bbx_plugin/include).

For detailed header documentation, see [FFI Integration](ffi-integration.md).

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
