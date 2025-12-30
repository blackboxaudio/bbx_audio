# bbx_plugin

Plugin integration crate for JUCE and other C/C++ frameworks.

## Overview

bbx_plugin provides:

- Re-exports of `bbx_dsp` for single-dependency usage
- `PluginDsp` trait for plugin DSP implementations
- `bbx_plugin_ffi!` macro for generating C FFI exports
- Parameter definition and code generation utilities

## Installation

```toml
[package]
name = "my_plugin_dsp"
edition = "2024"

[lib]
crate-type = ["staticlib"]

[dependencies]
bbx_plugin = "0.1"
```

## Features

| Feature | Description |
|---------|-------------|
| [PluginDsp Trait](plugin/plugin-dsp.md) | Interface for plugin DSP |
| [FFI Macro](plugin/ffi-macro.md) | Generate C exports |
| [Parameter Definitions](plugin/params.md) | JSON and programmatic params |

## Quick Example

```rust
use bbx_plugin::{PluginDsp, DspContext, bbx_plugin_ffi};

pub struct MyPlugin {
    gain: f32,
}

impl Default for MyPlugin {
    fn default() -> Self { Self::new() }
}

impl PluginDsp for MyPlugin {
    fn new() -> Self {
        Self { gain: 1.0 }
    }

    fn prepare(&mut self, _context: &DspContext) {}

    fn reset(&mut self) {}

    fn apply_parameters(&mut self, params: &[f32]) {
        self.gain = params.get(0).copied().unwrap_or(1.0);
    }

    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        context: &DspContext,
    ) {
        for ch in 0..inputs.len().min(outputs.len()) {
            for i in 0..context.buffer_size {
                outputs[ch][i] = inputs[ch][i] * self.gain;
            }
        }
    }
}

// Generate FFI exports
bbx_plugin_ffi!(MyPlugin);
```

## Re-exports

bbx_plugin re-exports key types from bbx_dsp:

```rust
// All available from bbx_plugin directly
use bbx_plugin::{
    PluginDsp,
    DspContext,
    blocks::{GainBlock, PannerBlock, OscillatorBlock},
    waveform::Waveform,
};
```

## JUCE Integration

For complete integration guide, see [JUCE Plugin Integration](../juce/overview.md).
