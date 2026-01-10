# DspContext

`DspContext` holds audio processing parameters passed to DSP blocks.

## Definition

```rust
pub struct DspContext {
    /// Sample rate in Hz
    pub sample_rate: f64,

    /// Number of samples per buffer
    pub buffer_size: usize,

    /// Number of audio channels
    pub num_channels: usize,
}
```

## Creating a Context

```rust
use bbx_dsp::context::DspContext;

let context = DspContext::new(44100.0, 512, 2);
```

### Default Constants

```rust
use bbx_dsp::context::{DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE};

// DEFAULT_SAMPLE_RATE = 44100.0
// DEFAULT_BUFFER_SIZE = 512
```

## Usage in Blocks

Context is passed to all block methods:

```rust
use bbx_dsp::{block::Block, context::DspContext, sample::Sample};

impl<S: Sample> Block<S> for MyBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        context: &DspContext,
        modulation: &[S],
    ) {
        // Use context values
        let sample_rate = context.sample_rate;
        let buffer_size = context.buffer_size;

        for i in 0..buffer_size {
            // Process samples
        }
    }

    fn prepare(&mut self, context: &DspContext) {
        // Recalculate coefficients based on sample rate
        self.coefficient = self.calculate_coefficient(context.sample_rate);
    }
}
```

## Common Patterns

### Sample Rate Dependent Calculations

```rust
use bbx_dsp::context::DspContext;

fn calculate_filter_coefficient(cutoff_hz: f64, context: &DspContext) -> f64 {
    let normalized_freq = cutoff_hz / context.sample_rate;
    // Calculate coefficient...
    1.0 - (-2.0 * f64::PI * normalized_freq).exp()
}
```

### Time-Based Parameters

```rust
use bbx_dsp::context::DspContext;

fn samples_for_milliseconds(ms: f64, context: &DspContext) -> usize {
    (ms * context.sample_rate / 1000.0) as usize
}

fn samples_for_seconds(seconds: f64, context: &DspContext) -> usize {
    (seconds * context.sample_rate) as usize
}
```

### Phase Increment

```rust
use bbx_dsp::context::DspContext;

fn phase_increment(frequency: f64, context: &DspContext) -> f64 {
    frequency / context.sample_rate
}
```

## Context Changes

When audio specs change (e.g., DAW sample rate changes):

1. `prepare()` is called on all blocks
2. Blocks should recalculate time-dependent values
3. `reset()` may also be called to clear state

```rust
fn prepare(&mut self, context: &DspContext) {
    // Sample rate changed - recalculate everything
    self.phase_increment = self.frequency / context.sample_rate;
    self.filter.recalculate_coefficients(context.sample_rate);

    // Resize buffers if needed
    if self.delay_buffer.len() != context.buffer_size {
        self.delay_buffer.resize(context.buffer_size, 0.0);
    }
}
```
