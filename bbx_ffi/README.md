# bbx_ffi

C FFI bindings for integrating bbx_audio DSP with JUCE and other C/C++ frameworks.

## Features

- **Macro-based code generation**: Single macro generates all FFI exports
- **RAII handle management**: Safe opaque pointer lifecycle
- **Buffer processing**: Zero-copy audio buffer interop
- **Plugin integration**: Works with JUCE AudioProcessor

## Usage

### Implementing PluginDsp

```rust
use bbx_dsp::{PluginDsp, context::DspContext};
use bbx_ffi::bbx_plugin_ffi;

pub struct MyPlugin {
    // Your DSP state
}

impl PluginDsp for MyPlugin {
    fn new() -> Self {
        MyPlugin { /* ... */ }
    }

    fn prepare(&mut self, context: &DspContext) {
        // Called before processing starts
    }

    fn reset(&mut self) {
        // Called when playback stops
    }

    fn apply_parameters(&mut self, params: &[f32]) {
        // Called when parameters change
    }

    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        context: &DspContext,
    ) {
        // Audio processing
    }
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// Generate FFI exports
bbx_plugin_ffi!(MyPlugin);
```

### Generated FFI Functions

The `bbx_plugin_ffi!` macro generates:

- `bbx_create() -> *mut BbxGraph` - Create plugin instance
- `bbx_destroy(handle)` - Destroy plugin instance
- `bbx_prepare(handle, sample_rate, buffer_size, channels)` - Prepare for playback
- `bbx_reset(handle)` - Reset state
- `bbx_apply_parameters(handle, params, count)` - Update parameters
- `bbx_process(handle, inputs, outputs, channels, samples)` - Process audio

### JUCE Integration

See the workspace README for complete JUCE integration examples with CMake build setup.

## License

MIT
