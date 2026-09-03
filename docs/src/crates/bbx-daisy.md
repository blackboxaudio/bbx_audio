# bbx_daisy

Electrosmith Daisy embedded audio support for no_std ARM Cortex-M targets.

## Overview

bbx_daisy provides:

- Hardware abstractions for Electrosmith Daisy platforms (Seed, Pod, Patch SM)
- Stack-allocated buffer types for realtime audio processing
- Integration with bbx_dsp blocks in embedded contexts
- Macros for minimal boilerplate audio applications

## Installation

```toml
[dependencies]
bbx_daisy = { version = "0.5", features = ["seed"] }
```

## Features

Exactly one board feature must be selected (there is no default):

| Feature | Description |
|---------|-------------|
| `seed` | Daisy Seed with AK4556 codec |
| `seed_1_1` | Daisy Seed 1.1 with WM8731 codec |
| `seed_1_2` | Daisy Seed 1.2 with PCM3060 codec |
| `pod` | Daisy Pod with WM8731 codec |
| `patch_sm` | Patch SM with PCM3060 codec |
| `patch_init` | Patch.Init() (uses Patch SM) |

## Quick Example

```rust
#![no_std]
#![no_main]

use bbx_daisy::prelude::*;

struct SineOsc {
    phase: f32,
    phase_inc: f32,
}

impl AudioProcessor for SineOsc {
    fn process(
        &mut self,
        _input: &FrameBuffer<BLOCK_SIZE>,
        output: &mut FrameBuffer<BLOCK_SIZE>,
        _controls: &Controls,
    ) {
        for i in 0..BLOCK_SIZE {
            let sample = libm::sinf(self.phase * core::f32::consts::TAU) * 0.5;
            output.set_frame(i, sample, sample);
            self.phase += self.phase_inc;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}

bbx_daisy_audio!(SineOsc, SineOsc {
    phase: 0.0,
    phase_inc: 440.0 / DEFAULT_SAMPLE_RATE
});
```

## API Reference

| Component | Description |
|-----------|-------------|
| `AudioProcessor` | Trait for implementing audio processing callbacks |
| `Controls` | Per-block snapshot of the control surface (knobs, CV jacks, gates, buttons) |
| `Outputs` / `outputs()` | Processor-driven outputs (LED, CV out, gate outs), applied by the main loop |
| `StaticSampleBuffer` | Stack-allocated sample buffer for DSP processing |
| `FrameBuffer` | Interleaved stereo buffer for SAI/DMA hardware output |
| `EmbeddedDspContext` | Audio context with sample rate and buffer size |
| `Board` | Hardware abstraction for GPIO, peripherals, and codec |

## Hardware Controls

`bbx_daisy_audio_with_controls!` reads the board's control surface in the main
loop (~1 kHz) and hands `process()` a lock-free `Controls` snapshot. On the Pod
that is `knobs[0..2]`; on the Patch.Init() it is the full panel — `knobs[0..4]`
(smoothed 0-1), `cv[0..4]` (CV jacks, bipolar -1..+1 ≈ ±5 V,
inversion-corrected), `gate1`/`gate2` (raw levels), `button` (B7), and `switch`
(B8). In the other direction, `bbx_daisy::outputs()` drives the Patch.Init()
front-panel LED (analog brightness via DAC), the CV OUT jack, and both gate
outputs from anywhere — typically inside `process()`.

## Buffer Types

### StaticSampleBuffer

A stack-allocated buffer for single-channel sample data:

```rust
let mut buffer = StaticSampleBuffer::<512>::new();
buffer.fill(0.0);
```

### FrameBuffer

An interleaved stereo buffer matching the SAI/DMA format:

```rust
let mut output = FrameBuffer::<BLOCK_SIZE>::new();
output.set_frame(0, left_sample, right_sample);
```

## Build and Flash

```bash
# Install ARM target
rustup target add thumbv7em-none-eabihf

# Build
cargo build -p bbx_daisy --example 02_oscillator --features seed --target thumbv7em-none-eabihf --release

# Flash with probe-rs (recommended)
cargo run -p bbx_daisy --example 02_oscillator --features seed --release

# Flash via DFU (hold BOOT, tap RESET, release BOOT first)
dfu-util -a 0 -s 0x08000000:leave -D target/thumbv7em-none-eabihf/release/examples/02_oscillator
```

## Examples

See `bbx_daisy/examples/` for working examples:

- `01_blink` - GPIO LED blink without audio
- `02_oscillator` - Basic sine wave output
- `05_pod_synth` - Pod knobs controlling pitch and filter cutoff
- `10_patch_init_io` - Full Patch.Init() control-surface bring-up check
