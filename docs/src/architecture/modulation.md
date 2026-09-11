# Modulation System

How parameters are modulated in bbx_audio.

## Overview

Modulation allows parameters to change over time:

- LFO modulating pitch → vibrato
- Envelope modulating amplitude → ADSR
- LFO modulating filter cutoff → wah effect

## Control Rate vs Audio Rate

bbx_audio uses **control rate** modulation through parameters:

| Aspect | Audio Rate | Control Rate |
|--------|------------|--------------|
| Updates | Per sample | Per buffer |
| CPU cost | High | Low |
| Latency | None | 1 buffer |
| Precision | Perfect | Good enough |

Most musical modulation is below 20 Hz, well within control rate capabilities. When a
control needs per-sample resolution, connect the modulator's output as an **audio input**
instead, the way `VcaBlock` takes an envelope on input 1.

## Modulation Flow

1. **Modulator block processes** (LFO, Envelope)
2. **First sample of every modulation output collected** into a flat table, block by block
3. **Target block receives** the table as a `ModulationValues` view
4. **Parameter resolves** `base + Σ depth · source`

```rust
fn collect_modulation_values(&mut self, block_id: BlockId) {
    let start = self.modulation_offsets[block_id.0];
    for output in 0..self.blocks[block_id.0].modulation_outputs().len() {
        let buffer_index = self.get_buffer_index(block_id, output);
        self.modulation_values[start + output] = self.audio_buffers[buffer_index][0];
    }
}
```

Modulation output `i` of a block is read from its audio output `i`, so a modulator with
several outputs simply declares several `ModulationOutput`s and writes several buffers.

## Parameter Type

```rust
pub struct Parameter<S: Sample> {
    base: S,
    routes: StackVec<ModulationRoute<S>, 4>, // { source: (BlockId, output), depth: S }
}
```

A parameter with no routes is a constant. Routes sum, and each carries its own depth, so
the modulator emits its natural range and the destination decides how much of it to use.

## Routing Modulation

Use `modulate()` for the common case (first output, unity depth) and `modulate_with()` to
choose an output and a depth. Both validate the wiring and return `Result`:

```rust
// Create LFO (frequency, depth, waveform, seed)
let lfo = builder.add(LfoBlock::new(5.0, 1.0, Waveform::Sine, None));

// Create oscillator
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));

// ±5 Hz vibrato
builder.modulate_with(ModulationRoute::new(ModulationSource::first_output(lfo), 5.0), osc, "frequency")?;
```

Errors are typed (`GraphError::UnknownParameter`, `BlockNotFound`,
`InvalidModulationOutput`, `Parameter`) and surface at build time, never on the audio thread.
