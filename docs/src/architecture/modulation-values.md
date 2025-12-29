# Modulation Value Collection

How modulation values are collected and distributed.

## Collection Process

After each modulator block processes:

```rust
fn collect_modulation_values(&mut self, block_id: BlockId) {
    // Skip if block has no modulation outputs
    if self.blocks[block_id.0].modulation_outputs().is_empty() {
        return;
    }

    // Get the first sample as the modulation value
    let buffer_index = self.get_buffer_index(block_id, 0);
    if let Some(&first_sample) = self.audio_buffers[buffer_index].get(0) {
        self.modulation_values[block_id.0] = first_sample;
    }
}
```

## Why First Sample?

Modulation is control-rate (per-buffer), not audio-rate (per-sample):

- LFO at 5 Hz with 512-sample buffer at 44.1 kHz
- Buffer duration: 512 / 44100 ≈ 11.6 ms
- LFO phase change: 5 * 0.0116 ≈ 0.058 cycles
- Taking first sample is sufficient

## Storage

Pre-allocated array indexed by block ID:

```rust
// In Graph
modulation_values: Vec<S>,

// Sized during prepare_for_playback
self.modulation_values.resize(self.blocks.len(), S::ZERO);
```

## Passing to Blocks

Blocks receive the full modulation array:

```rust
block.process(
    input_slices.as_slice(),
    output_slices.as_mut_slice(),
    &self.modulation_values,  // All values
    &self.context,
);
```

Blocks index by their modulator's BlockId:

```rust
fn process(&mut self, ..., modulation: &[S], ...) {
    let mod_value = modulation[self.modulator_id.0];
    // Apply modulation
}
```

## Timing Considerations

Modulation is applied with 1-buffer latency:

1. Buffer N: LFO generates sample
2. Buffer N: Value collected
3. Buffer N: Target block uses value

This is acceptable for musical modulation (typically < 20 Hz).
