# Modulation System

How parameters are modulated in bbx_audio.

## Overview

Modulation allows parameters to change over time:

- LFO modulating pitch → vibrato
- Envelope modulating amplitude → ADSR
- LFO modulating filter cutoff → wah effect

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
4. **Block applies** value to parameters

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

```rust
pub enum Parameter<S: Sample> {
    Static(S),              // Fixed value
    Modulated(BlockId),     // Controlled by modulator
}
```

## Routing Modulation

Use the `modulate()` method to connect modulators to parameters:

```rust
// Create LFO (frequency, depth, waveform, seed)
let lfo = builder.add(LfoBlock::new(5.0, 0.5, Waveform::Sine, None));

// Create oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// Connect modulation
builder.modulate(lfo, osc, "frequency");
```