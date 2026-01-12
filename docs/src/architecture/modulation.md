# Modulation System

How parameters are modulated in bbx_audio.

## Overview

Modulation allows parameters to change over time:

- LFO modulating pitch -> vibrato
- Envelope modulating amplitude -> ADSR
- LFO modulating filter cutoff -> wah effect

## Control Rate vs Audio Rate

bbx_audio uses **control rate** modulation:

| Aspect | Audio Rate | Control Rate |
|--------|------------|--------------|
| Updates | Per sample | Per buffer |
| CPU cost | High | Low |
| Latency | None | 1 buffer |
| Precision | Perfect | Good enough |

Most musical modulation is below 20 Hz, well within control rate capabilities.

## Modulation Flow

1. **Modulator block processes** (LFO, Envelope)
2. **First sample collected** as modulation value
3. **Target block receives** value in modulation array
4. **Block uses Parameter** to apply smoothed modulation

```rust
fn collect_modulation_values(&mut self, block_id: BlockId) {
    if has_modulation_outputs(block_id) {
        let buffer_index = self.get_buffer_index(block_id, 0);
        let first_sample = self.audio_buffers[buffer_index][0];
        self.modulation_values[block_id.0] = first_sample;
    }
}
```

## Parameter Type

Parameters combine value sources with built-in smoothing:

```rust
pub struct Parameter<S: Sample, T: SmoothingStrategy = Linear> {
    source: ParameterSource<S>,
    smoother: SmoothedValue<S, T>,
    ramp_length_ms: f64,
}

pub enum ParameterSource<S: Sample> {
    Constant(S),
    Modulated(BlockId),
}
```

Blocks use `Parameter` for click-free value changes:

```rust
// Update target from modulation source
self.frequency.update_target(modulation_values);

// Get smoothed values for processing
if self.frequency.is_smoothing() {
    for i in 0..buffer_size {
        let freq = self.frequency.next_value();
        // Process with per-sample frequency...
    }
} else {
    let freq = self.frequency.current();
    // Process with constant frequency...
}
```

## Routing Modulation

Use the `modulate()` method to connect modulators to parameters:

```rust
// Create LFO
let lfo = builder.add_lfo(5.0, Waveform::Sine);

// Create oscillator
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Connect modulation
builder.modulate(lfo, osc, "frequency");
```
