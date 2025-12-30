# Implementing PluginDsp

The `PluginDsp` trait defines the interface between your Rust DSP code and the FFI layer.

## The PluginDsp Trait

```rust
pub trait PluginDsp: Default + Send + 'static {
    fn new() -> Self;
    fn prepare(&mut self, context: &DspContext);
    fn reset(&mut self);
    fn apply_parameters(&mut self, params: &[f32]);
    fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], context: &DspContext);
}
```

## Trait Bounds

- **`Default`** - Required for the FFI layer to create instances
- **`Send`** - Allows transfer between threads (audio thread)
- **`'static`** - No borrowed references (owned data only)

## Method Reference

### new()

Create a new instance with default configuration.

```rust
fn new() -> Self {
    Self {
        gain: GainBlock::new(0.0),
        panner: PannerBlock::new(0.0),
    }
}
```

### prepare()

Called when audio specifications change. Initialize blocks with the new context.

```rust
fn prepare(&mut self, context: &DspContext) {
    // context.sample_rate - Sample rate in Hz
    // context.buffer_size - Samples per buffer
    // context.num_channels - Number of channels

    self.gain.prepare(context);
    self.panner.prepare(context);
}
```

### reset()

Clear all DSP state (filter histories, delay lines, oscillator phases).

```rust
fn reset(&mut self) {
    self.gain.reset();
    self.panner.reset();
}
```

### apply_parameters()

Map the flat parameter array to your DSP blocks.

```rust
fn apply_parameters(&mut self, params: &[f32]) {
    // Use generated constants for indices
    self.gain.level_db = params[PARAM_GAIN];
    self.panner.position = params[PARAM_PAN];
}
```

### process()

Process audio through your DSP chain.

```rust
fn process(
    &mut self,
    inputs: &[&[f32]],
    outputs: &mut [&mut [f32]],
    context: &DspContext,
) {
    // inputs[channel][sample] - Input audio
    // outputs[channel][sample] - Output audio (write here)

    // Example: copy input to output with gain
    for ch in 0..context.num_channels {
        for i in 0..context.buffer_size {
            outputs[ch][i] = inputs[ch][i] * self.gain.multiplier();
        }
    }
}
```

## Complete Example

```rust
use bbx_plugin::{PluginDsp, DspContext, bbx_plugin_ffi};
use bbx_plugin::blocks::{GainBlock, PannerBlock, DcBlockerBlock};

// Parameter indices (generated or manual)
const PARAM_GAIN: usize = 0;
const PARAM_PAN: usize = 1;
const PARAM_DC_BLOCK: usize = 2;

pub struct PluginGraph {
    gain: GainBlock<f32>,
    panner: PannerBlock<f32>,
    dc_blocker: DcBlockerBlock<f32>,
    dc_block_enabled: bool,
}

impl Default for PluginGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginDsp for PluginGraph {
    fn new() -> Self {
        Self {
            gain: GainBlock::new(0.0),
            panner: PannerBlock::new(0.0),
            dc_blocker: DcBlockerBlock::new(),
            dc_block_enabled: false,
        }
    }

    fn prepare(&mut self, context: &DspContext) {
        self.dc_blocker.prepare(context);
    }

    fn reset(&mut self) {
        self.dc_blocker.reset();
    }

    fn apply_parameters(&mut self, params: &[f32]) {
        self.gain.level_db = params[PARAM_GAIN];
        self.panner.position = params[PARAM_PAN];
        self.dc_block_enabled = params[PARAM_DC_BLOCK] > 0.5;
    }

    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        context: &DspContext,
    ) {
        let num_channels = context.num_channels.min(inputs.len()).min(outputs.len());
        let num_samples = context.buffer_size;

        for ch in 0..num_channels {
            for i in 0..num_samples {
                let mut sample = inputs[ch][i];

                // Apply gain
                sample *= self.gain.multiplier();

                // Apply DC blocker if enabled
                if self.dc_block_enabled {
                    sample = self.dc_blocker.process_sample(sample, ch);
                }

                outputs[ch][i] = sample;
            }
        }

        // Apply panning (modifies stereo field)
        self.panner.process_stereo(outputs, num_samples);
    }
}

// Generate FFI exports
bbx_plugin_ffi!(PluginGraph);
```
