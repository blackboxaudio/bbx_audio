# Modulation Value Collection

How modulation values are collected and distributed.

## Collection Process

After each modulator block processes, the first sample of **every** one of its
modulation outputs is copied into a flat table:

```rust
fn collect_modulation_values(&mut self, block_id: BlockId) {
    let output_count = self.blocks[block_id.0].modulation_outputs().len();
    let start = self.modulation_offsets[block_id.0];

    // Modulation output i is read from audio output i
    for output in 0..output_count {
        let buffer_index = self.get_buffer_index(block_id, output);
        if let Some(&first_sample) = self.audio_buffers[buffer_index].as_slice().first() {
            self.modulation_values[start + output] = first_sample;
        }
    }
}
```

## Why First Sample?

Modulation is control-rate (per-buffer), not audio-rate (per-sample):

- LFO at 5 Hz with 512-sample buffer at 44.1 kHz
- Buffer duration: 512 / 44100 ≈ 11.6 ms
- LFO phase change: 5 * 0.0116 ≈ 0.058 cycles
- Taking first sample is sufficient

When a control needs per-sample resolution, connect the modulator as an **audio input**
of the target instead (`VcaBlock` reads its envelope on input 1).

## Storage

A flat value array plus an offset table, both sized during `prepare()`. Block `b`'s
outputs occupy `values[offsets[b]..offsets[b + 1]]`, so a `ModulationSource { block, output }`
resolves to `offsets[block] + output` with no search:

```rust
// In Graph
modulation_values: Vec<S>,
modulation_offsets: Vec<usize>, // blocks.len() + 1 entries

// Sized during prepare()
self.modulation_offsets.push(0);
for block in &self.blocks {
    let end = self.modulation_offsets.last().unwrap() + block.modulation_outputs().len();
    self.modulation_offsets.push(end);
}
self.modulation_values.resize(total, S::ZERO);
```

Nothing is allocated after `prepare()`.

## Passing to Blocks

Blocks receive a borrowed `ModulationValues` view over both arrays:

```rust
block.process(
    input_slices.as_slice(),
    output_slices.as_mut_slice(),
    &ModulationValues::new(&self.modulation_values, &self.modulation_offsets),
    &self.context,
);
```

Blocks never index the table themselves. Each `Parameter` holds its routes and resolves
`base + Σ depth · source` through the view:

```rust
fn process(&mut self, ..., modulation_values: &ModulationValues<S>, ...) {
    let cutoff = self.cutoff.value(modulation_values);
    // Apply modulation
}
```

A source that does not exist, or an output index beyond what the block declares, reads
as zero. The audio thread never panics on a bad route; bad routes are rejected earlier by
`GraphBuilder::modulate`.

## Timing Considerations

Modulation is applied with 1-buffer latency:

1. Buffer N: LFO generates sample
2. Buffer N: Value collected
3. Buffer N: Target block uses value

This is acceptable for musical modulation (typically < 20 Hz).
