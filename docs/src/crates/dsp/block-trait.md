# Block Trait

The `Block` trait defines the interface for DSP processing blocks.

## Trait Definition

```rust
pub trait Block<S: Sample> {
    /// Prepare for playback with given context
    fn prepare(&mut self, _context: &DspContext) {}

    /// Process audio through the block
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    );

    /// Number of input ports
    fn input_count(&self) -> usize;

    /// Number of output ports
    fn output_count(&self) -> usize;

    /// Modulation outputs provided by this block
    fn modulation_outputs(&self) -> &[ModulationOutput];
}
```

## Implementing a Custom Block

```rust
use bbx_dsp::{
    block::Block,
    context::DspContext,
    parameter::ModulationOutput,
    sample::Sample,
};

struct MyGainBlock<S: Sample> {
    gain: S,
}

impl<S: Sample> MyGainBlock<S> {
    fn new(gain_db: f64) -> Self {
        let linear = (10.0_f64).powf(gain_db / 20.0);
        Self {
            gain: S::from_f64(linear),
        }
    }
}

impl<S: Sample> Block<S> for MyGainBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        _modulation_values: &[S],
        context: &DspContext,
    ) {
        for ch in 0..inputs.len().min(outputs.len()) {
            for i in 0..context.buffer_size {
                outputs[ch][i] = inputs[ch][i] * self.gain;
            }
        }
    }

    fn input_count(&self) -> usize { 1 }
    fn output_count(&self) -> usize { 1 }
    fn modulation_outputs(&self) -> &[ModulationOutput] { &[] }
}
```

## Process Method

The `process` method receives:

- `inputs` - Slice of input channel buffers
- `outputs` - Mutable slice of output channel buffers
- `modulation_values` - Values from connected modulator blocks
- `context` - Processing context (sample rate, buffer size)

### Input/Output Layout

```rust
fn process(
    &mut self,
    inputs: &[&[S]],           // inputs[channel][sample]
    outputs: &mut [&mut [S]],  // outputs[channel][sample]
    modulation_values: &[S],
    context: &DspContext,
) {
    // inputs.len() = number of input channels
    // inputs[0].len() = number of samples per channel
}
```

### Modulation Values

For blocks that receive modulation:

```rust
fn process(
    &mut self,
    inputs: &[&[S]],
    outputs: &mut [&mut [S]],
    modulation_values: &[S],
    context: &DspContext,
) {
    // modulation_values[0] = value from first connected modulator
    // Use for per-block (not per-sample) modulation
    let mod_depth = modulation_values.get(0).copied().unwrap_or(S::ZERO);
}
```

## Port Counts

### input_count / output_count

Return the number of audio ports:

```rust
// Mono effect
fn input_count(&self) -> usize { 1 }
fn output_count(&self) -> usize { 1 }

// Stereo panner
fn input_count(&self) -> usize { 1 }
fn output_count(&self) -> usize { 2 }

// Mixer (4 inputs, 1 output)
fn input_count(&self) -> usize { 4 }
fn output_count(&self) -> usize { 1 }
```

### modulation_outputs

For modulator blocks (LFO, envelope), return a slice describing each modulation output:

```rust
fn modulation_outputs(&self) -> &[ModulationOutput] {
    &[ModulationOutput {
        name: "LFO",
        min_value: -1.0,
        max_value: 1.0,
    }]
}
```

For non-modulator blocks, return an empty slice:

```rust
fn modulation_outputs(&self) -> &[ModulationOutput] { &[] }
```

## Lifecycle Methods

### prepare

Called when audio specs change:

```rust
fn prepare(&mut self, context: &DspContext) {
    // Recalculate filter coefficients for new sample rate
    self.coefficient = calculate_coefficient(context.sample_rate);
}
```

## Real-Time Safety

Blocks should follow real-time safety guidelines:

- Avoid allocating memory in `process()`
- Use atomic operations for cross-thread communication
- Never block or lock
