# LowPassFilterBlock

SVF-based low-pass filter with cutoff and resonance control.

## Overview

`LowPassFilterBlock` is a State Variable Filter (SVF) using the TPT (Topology Preserving Transform) algorithm. It provides:
- Stable filtering at all cutoff frequencies
- No delay-free loops
- Consistent behavior regardless of sample rate

## Creating a Low-Pass Filter

```rust
use bbx_dsp::blocks::LowPassFilterBlock;

// Create with cutoff at 1000 Hz, resonance at 0.707 (Butterworth)
let filter = LowPassFilterBlock::<f32>::new(1000.0, 0.707);
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Audio input |
| 0 | Output | Filtered audio |

## Parameters

| Parameter | Type | Range | Default |
|-----------|------|-------|---------|
| Cutoff | f64 | 20-20000 Hz | - |
| Resonance | f64 | 0.5-10.0 | 0.707 |

### Resonance Values

| Value | Character |
|-------|-----------|
| 0.5 | Heavily damped |
| 0.707 | Butterworth (flat) |
| 1.0 | Slight peak |
| 2.0+ | Pronounced peak |
| 10.0 | Near self-oscillation |

## Usage Examples

### Basic Filtering

```rust
let source = builder.add_oscillator(440.0, Waveform::Saw, None);
let filter = LowPassFilterBlock::new(2000.0, 0.707);

// Connect oscillator to filter
builder.connect(source, 0, filter_id, 0);
```

### Synthesizer Voice

```rust
// Typical synth voice with envelope-modulated filter
let osc = OscillatorBlock::new(440.0, Waveform::Saw);
let filter = LowPassFilterBlock::new(1000.0, 2.0);  // Resonant
let env = EnvelopeBlock::new(0.01, 0.2, 0.5, 0.3);
let amp = GainBlock::new(-6.0);
```

## Implementation Notes

- Uses TPT SVF algorithm for numerical stability
- Stereo processing (up to 2 channels)
- Independent state per channel
- Coefficients recalculated per buffer (supports modulation)
