# bbx_plugin

Plugin integration crate for bbx_audio DSP with C FFI bindings for JUCE and other C/C++ frameworks.

This crate re-exports `bbx_dsp`, so plugin projects only need to add `bbx_plugin` as a dependency.

## Features

- **Single dependency**: Re-exports `bbx_dsp` for convenient access to all DSP types
- **Macro-based code generation**: Single macro generates all FFI exports
- **RAII handle management**: Safe opaque pointer lifecycle
- **Buffer processing**: Zero-copy audio buffer interop
- **Plugin integration**: Works with JUCE AudioProcessor

## Cargo Features

### `ftz-daz`

Enables hardware-level denormal prevention. When enabled, `enable_ftz_daz()` is called automatically during `prepare()`, setting CPU flags to flush denormal floating-point numbers to zero.

```toml
[dependencies]
bbx_plugin = { version = "...", features = ["ftz-daz"] }
```

| Platform | Behavior |
|----------|----------|
| x86/x86_64 | Full FTZ + DAZ (inputs and outputs) |
| AArch64 (Apple Silicon) | FTZ only (outputs) |
| Other | No-op |

This is recommended for production audio plugins to avoid the 10-100x CPU slowdowns that denormals can cause.

### `simd`

Enables SIMD optimizations for DSP processing. Requires nightly Rust.

```toml
[dependencies]
bbx_plugin = { version = "...", features = ["simd"] }
```

Propagates to both `bbx_core` and `bbx_dsp`, enabling vectorized operations in supported blocks (Oscillator, LFO, Gain).

## Usage

### Implementing PluginDsp

```rust
use bbx_plugin::{PluginDsp, DspContext, bbx_plugin_ffi};

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
        midi_events: &[MidiEvent],
        context: &DspContext,
    ) {
        // Audio processing (midi_events for synthesizers)
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
- `bbx_process(handle, inputs, outputs, midi_events, midi_count, channels, samples)` - Process audio with MIDI

### JUCE Integration

See the workspace README for complete JUCE integration examples with CMake build setup.

## License

MIT
