# bbx_dsp

A block-based audio DSP system for building signal processing graphs.

## Features

- **Graph-based architecture**: Connect blocks to form processing chains
- **Generic sample type**: Works with f32 or f64 precision
- **Realtime-safe processing**: Stack-allocated buffers, no allocations in audio thread
- **Parameter modulation**: LFOs and envelopes can modulate block parameters
- **Topological sorting**: Automatic execution order based on connections

## Block Types

### Generators
- `OscillatorBlock` - Waveform generator (sine, square, saw, triangle, pulse, noise)

### Effectors
- `GainBlock` - Level control in dB
- `OverdriveBlock` - Asymmetric soft-clipping distortion
- `PannerBlock` - Stereo panning with constant power law
- `DcBlockerBlock` - DC offset removal
- `ChannelRouterBlock` - Channel routing and manipulation
- `LowPassFilterBlock` - SVF-based TPT low-pass filter with cutoff/resonance

### Modulators
- `LfoBlock` - Low-frequency oscillator for parameter modulation
- `EnvelopeBlock` - ADSR envelope generator

### I/O
- `FileInputBlock` - Read audio from files
- `FileOutputBlock` - Write audio to files (non-blocking I/O)
- `OutputBlock` - Terminal graph output

## PluginDsp Trait

For plugin integration, implement `PluginDsp` with optional MIDI support:
- `process()` receives `midi_events: &[MidiEvent]` parameter
- Override `note_on()`, `note_off()`, `control_change()`, `pitch_bend()` for MIDI handling

## Usage

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

// Create a graph with 44.1kHz sample rate, 512 sample buffer, stereo output
let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Add an oscillator
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// Build the graph (automatically adds output block)
let mut graph = builder.build();

// Process audio
let mut left = vec![0.0f32; 512];
let mut right = vec![0.0f32; 512];
let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
graph.process_buffers(&mut outputs);
```

## License

MIT
