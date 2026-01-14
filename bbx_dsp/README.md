# bbx_dsp

A block-based audio DSP system for building signal processing graphs.

## Features

- **Graph-based architecture**: Connect blocks to form processing chains
- **Generic sample type**: Works with f32 or f64 precision
- **Realtime-safe processing**: Stack-allocated buffers, no allocations in audio thread
- **Parameter modulation**: LFOs and envelopes can modulate block parameters
- **Parameter smoothing**: Click-free parameter changes with configurable ramp times via `set_smoothing()`
- **Topological sorting**: Automatic execution order based on connections

## Block Types

### Generators
- `OscillatorBlock` - Waveform generator (sine, square, saw, triangle, pulse, noise)

### Effectors
- `GainBlock` - Level control in dB
- `OverdriveBlock` - Asymmetric soft-clipping distortion
- `PannerBlock` - Stereo, surround (VBAP), and ambisonic panning
- `DcBlockerBlock` - DC offset removal
- `ChannelRouterBlock` - Simple stereo channel routing
- `ChannelSplitterBlock` - Split multi-channel to mono outputs
- `ChannelMergerBlock` - Merge mono inputs to multi-channel
- `MatrixMixerBlock` - NxM mixing matrix
- `AmbisonicDecoderBlock` - Ambisonics B-format decoder
- `BinauralDecoderBlock` - Ambisonics B-format to stereo binaural for headphones
- `LowPassFilterBlock` - SVF-based TPT low-pass filter with cutoff/resonance

### Modulators
- `LfoBlock` - Low-frequency oscillator for parameter modulation
- `EnvelopeBlock` - ADSR envelope generator

### I/O
- `FileInputBlock` - Read audio from files
- `FileOutputBlock` - Write audio to files (non-blocking I/O)
- `OutputBlock` - Terminal graph output

## Multi-Channel System

Beyond basic stereo, bbx_dsp supports surround and ambisonic formats:

### Channel Layouts
- `Mono` - 1 channel
- `Stereo` - 2 channels (default)
- `Surround51` - 6 channels (5.1)
- `Surround71` - 8 channels (7.1)
- `AmbisonicFoa` - 4 channels (1st order)
- `AmbisonicSoa` - 9 channels (2nd order)
- `AmbisonicToa` - 16 channels (3rd order)
- `Custom(n)` - Arbitrary channel count

### Channel Config

Blocks declare how they handle multi-channel audio:
- `Parallel` (default) - Process each channel independently
- `Explicit` - Block handles routing internally (panners, mixers, decoders)

Use `GraphBuilder::with_layout()` to create graphs with specific channel configurations.

## Cargo Features

### `simd`

Enables SIMD optimizations for supported blocks. Requires nightly Rust.

```toml
[dependencies]
bbx_dsp = { version = "...", features = ["simd"] }
```

Optimized blocks:
- `OscillatorBlock` - Vectorized waveform generation
- `LfoBlock` - Vectorized modulation signal generation
- `GainBlock` - Vectorized gain application

## PluginDsp Trait

For plugin integration, implement `PluginDsp` with optional MIDI support:
- `process()` receives `midi_events: &[MidiEvent]` parameter
- Override `note_on()`, `note_off()`, `control_change()`, `pitch_bend()` for MIDI handling

## Benchmarking

Run performance benchmarks to measure SIMD optimization effectiveness:

```bash
# Scalar baseline
cargo bench --benches -p bbx_dsp -- --save-baseline scalar

# SIMD comparison (requires nightly)
cargo +nightly bench --benches -p bbx_dsp --features simd -- --save-baseline scalar
```

HTML reports are generated in `target/criterion/`. See the [full documentation](https://docs.rs/bbx_dsp) for details.

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
