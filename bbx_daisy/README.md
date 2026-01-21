# bbx_daisy

Electrosmith Daisy hardware support for bbx_audio.

This crate provides stack-allocated buffer types and hardware abstractions for running bbx_audio DSP on Electrosmith Daisy platforms (Seed, Pod, Patch SM, etc.).

## Features

- **Stack-allocated buffers**: `StaticSampleBuffer` and `FrameBuffer` for embedded targets without heap allocation
- **Embedded DSP context**: Memory-optimized `EmbeddedDspContext` for realtime processing
- **Hardware abstractions**: GPIO, ADC, encoder, and audio codec support (Phase 4)
- **Pin mappings**: Pre-defined pin configurations for all Daisy variants

## Supported Boards

| Board | Feature Flag | Codec | Status |
|-------|--------------|-------|--------|
| Daisy Seed | `seed` (default) | AK4556 | Buffer types ready |
| Daisy Seed 1.1 | `seed_1_1` | WM8731 | Buffer types ready |
| Daisy Seed 1.2 | `seed_1_2` | PCM3060 | Buffer types ready |
| Daisy Pod | `pod` | WM8731 | Buffer types ready |
| Patch SM | `patch_sm` | PCM3060 | Buffer types ready |
| Patch.Init() | `patch_init` | PCM3060 | Buffer types ready |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bbx_daisy = { version = "0.4.3", default-features = false, features = ["seed"] }
```

### Basic Usage

```rust
#![no_std]
#![no_main]

use bbx_daisy::{StaticSampleBuffer, FrameBuffer, EmbeddedDspContext};

// Stack-allocated buffers (no heap!)
let mut left: StaticSampleBuffer<32, f32> = StaticSampleBuffer::new();
let mut right: StaticSampleBuffer<32, f32> = StaticSampleBuffer::new();

// Interleaved output for DMA
let mut output: FrameBuffer<32, 2, f32> = FrameBuffer::new();

// DSP context
let mut ctx: EmbeddedDspContext<32> = EmbeddedDspContext::new(48000.0);

// Process audio (example: simple passthrough)
fn process_audio(
    input: &FrameBuffer<32>,
    output: &mut FrameBuffer<32>,
    ctx: &mut EmbeddedDspContext<32>,
) {
    // Deinterleave input
    let mut left: StaticSampleBuffer<32, f32> = StaticSampleBuffer::new();
    let mut right: StaticSampleBuffer<32, f32> = StaticSampleBuffer::new();
    input.deinterleave_to(left.as_mut_slice(), right.as_mut_slice());

    // Process DSP here...

    // Interleave output
    output.interleave_from(left.as_slice(), right.as_slice());
    ctx.advance();
}
```

## Buffer Types

### StaticSampleBuffer

Stack-allocated sample buffer for per-channel DSP processing. Implements `bbx_core::Buffer` trait for compatibility with bbx_dsp blocks.

```rust
use bbx_daisy::StaticSampleBuffer;

let mut buffer: StaticSampleBuffer<64, f32> = StaticSampleBuffer::new();
buffer.fill(0.5);
buffer[0] = 1.0;
```

### FrameBuffer

Interleaved multi-channel buffer for SAI/DMA hardware transfers. Required format for Daisy codecs.

```rust
use bbx_daisy::FrameBuffer;

let mut buffer: FrameBuffer<64, 2, f32> = FrameBuffer::new();
buffer.set_frame(0, 0.5, -0.5);  // Set stereo frame

// Convert between interleaved and separate channels
let left = [0.1, 0.2, 0.3, 0.4];
let right = [0.5, 0.6, 0.7, 0.8];
buffer.interleave_from(&left, &right);
```

## EmbeddedDspContext

Memory-optimized DSP context for embedded targets:

- Uses `f32` for sample rate (saves 4 bytes vs `f64`)
- Uses `u32` for sample counter (sufficient for ~24 hours at 48kHz)
- Buffer size is a const generic (known at compile time)

```rust
use bbx_daisy::EmbeddedDspContext;

let mut ctx: EmbeddedDspContext<32> = EmbeddedDspContext::new(48000.0);
assert_eq!(ctx.buffer_size(), 32);

// After each audio callback
ctx.advance();
```

## Building

```bash
# Install ARM target
rustup target add thumbv7em-none-eabihf

# Build for Daisy Seed
cargo build -p bbx_daisy --target thumbv7em-none-eabihf --release

# Build for other variants
cargo build -p bbx_daisy --target thumbv7em-none-eabihf --features pod --release
cargo build -p bbx_daisy --target thumbv7em-none-eabihf --features patch_sm --release
```

## Roadmap

- [x] Phase 3: Buffer types and crate structure
- [ ] Phase 4: HAL integration (SAI, DMA, codecs)
- [ ] Phase 5: Full board support packages
- [ ] Phase 6: Examples and documentation
