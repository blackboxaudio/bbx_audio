# Block Trait

The `Block` trait defines the interface for DSP processing blocks.

## Trait Definition

```rust
pub trait Block<S: Sample>: Send {
    /// Process audio through the block
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        context: &DspContext,
        modulation: &[S],
    );

    /// Number of input ports
    fn num_inputs(&self) -> usize;

    /// Number of output ports
    fn num_outputs(&self) -> usize;

    /// Number of modulation outputs (for LFOs, envelopes)
    fn num_modulation_outputs(&self) -> usize {
        0  // Default: no modulation outputs
    }

    /// Prepare for playback with given context
    fn prepare(&mut self, _context: &DspContext) {}

    /// Reset DSP state
    fn reset(&mut self) {}

    /// Finalize (for file output blocks)
    fn finalize(&mut self) {}
}
```

## Implementing a Custom Block

```rust
use bbx_dsp::{block::Block, context::DspContext, sample::Sample};

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
        context: &DspContext,
        _modulation: &[S],
    ) {
        for ch in 0..inputs.len().min(outputs.len()) {
            for i in 0..context.buffer_size {
                outputs[ch][i] = inputs[ch][i] * self.gain;
            }
        }
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}
```

## Process Method

The `process` method receives:

- `inputs` - Slice of input channel buffers
- `outputs` - Mutable slice of output channel buffers
- `context` - Processing context (sample rate, buffer size)
- `modulation` - Values from connected modulator blocks

### Input/Output Layout

```rust
fn process(
    &mut self,
    inputs: &[&[S]],      // inputs[channel][sample]
    outputs: &mut [&mut [S]],  // outputs[channel][sample]
    context: &DspContext,
    modulation: &[S],
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
    context: &DspContext,
    modulation: &[S],
) {
    // modulation[0] = value from first connected modulator
    // Use for per-block (not per-sample) modulation
    let mod_depth = modulation.get(0).copied().unwrap_or(S::ZERO);
}
```

## Port Counts

### num_inputs / num_outputs

Return the number of audio ports:

```rust
// Mono effect
fn num_inputs(&self) -> usize { 1 }
fn num_outputs(&self) -> usize { 1 }

// Stereo panner
fn num_inputs(&self) -> usize { 1 }
fn num_outputs(&self) -> usize { 2 }

// Mixer (4 inputs, 1 output)
fn num_inputs(&self) -> usize { 4 }
fn num_outputs(&self) -> usize { 1 }
```

### num_modulation_outputs

For modulator blocks (LFO, envelope):

```rust
fn num_modulation_outputs(&self) -> usize { 1 }
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

### reset

Clear all state:

```rust
fn reset(&mut self) {
    self.filter_state = 0.0;
    self.delay_buffer.fill(0.0);
}
```

### finalize

For file output blocks:

```rust
fn finalize(&mut self) {
    self.writer.flush().ok();
}
```

## Thread Safety

The `Send` bound ensures blocks can be sent to the audio thread. Blocks should:

- Avoid allocating memory in `process()`
- Use atomic operations for cross-thread communication
- Never block or lock
