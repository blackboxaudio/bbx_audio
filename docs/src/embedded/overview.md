# Embedded Development

bbx_audio supports embedded audio development on Electrosmith Daisy hardware through the `bbx_daisy` crate. This chapter covers setup, building, and deploying audio applications to ARM Cortex-M microcontrollers.

## Supported Hardware

| Board | Feature Flag | Codec | Notes |
|-------|-------------|-------|-------|
| Daisy Seed | `seed` (default) | AK4556 | Base development board |
| Daisy Seed 1.1 | `seed_1_1` | WM8731 | Updated codec |
| Daisy Seed 1.2 | `seed_1_2` | PCM3060 | Latest revision |
| Daisy Pod | `pod` | WM8731 | Seed + controls enclosure |
| Patch SM | `patch_sm` | PCM3060 | Surface-mount module |
| Patch.Init() | `patch_init` | PCM3060 | Eurorack format |

All boards use the STM32H750 microcontroller with:
- 480 MHz ARM Cortex-M7 core
- 128 KB internal flash + external QSPI
- 1 MB total RAM across multiple regions
- 48 kHz / 96 kHz audio sample rates

## Prerequisites

1. **Rust nightly toolchain** (already configured via `rust-toolchain.toml`)
2. **ARM target**: `rustup target add thumbv7em-none-eabihf`
3. **Debug probe** (recommended): ST-Link, J-Link, or CMSIS-DAP
4. **Or DFU utility**: `dfu-util` for USB flashing without a probe

## Quick Start

Add the dependency with your board's feature:

```toml
[dependencies]
bbx_daisy = { git = "https://github.com/blackboxaudio/bbx_audio", features = ["seed"] }
```

Create a minimal audio processor:

```rust
#![no_std]
#![no_main]

use bbx_daisy::prelude::*;

struct SineOsc {
    phase: f32,
}

impl AudioProcessor for SineOsc {
    fn process(&mut self, _input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        for i in 0..BLOCK_SIZE {
            let sample = libm::sinf(self.phase * core::f32::consts::TAU) * 0.5;
            output.set_frame(i, sample, sample);
            self.phase = (self.phase + 440.0 / DEFAULT_SAMPLE_RATE).fract();
        }
    }
}

bbx_daisy_audio!(SineOsc, SineOsc { phase: 0.0 });
```

Build and flash:

```bash
# Build for ARM
cargo build -p bbx_daisy --example 02_oscillator --target thumbv7em-none-eabihf --release

# Flash with probe-rs (recommended)
cargo run -p bbx_daisy --example 02_oscillator --release

# Or flash via DFU
dfu-util -a 0 -s 0x08000000:leave -D target/thumbv7em-none-eabihf/release/examples/02_oscillator
```

## Learning Path

New to embedded development? Follow this recommended reading order:

### 1. Fundamentals
Start here to understand how embedded Rust differs from desktop Rust:
- [Introduction to Embedded Rust](fundamentals/intro-embedded-rust.md) - `#![no_std]`, `#![no_main]`, and essential crates
- [Hardware Peripherals](fundamentals/hardware-peripherals.md) - GPIO, DMA, SAI, I2C explained
- [Memory Model](fundamentals/memory-model.md) - Stack, static allocation, linker sections

### 2. Compilation & Toolchain
Understand how your code becomes firmware:
- [Toolchain & LLVM](compilation/toolchain.md) - Target triples, FPU, cross-compilation
- [Linker Scripts](compilation/linker-scripts.md) - memory.x and section placement
- [Binary Formats & Flashing](compilation/binary-formats.md) - ELF, BIN, DFU

### 3. Daisy Hardware
Deep dive into the STM32H750 and audio subsystem:
- [STM32H750 MCU](daisy-hardware/stm32h750.md) - Core architecture, memory regions, power domains
- [Clock Tree](daisy-hardware/clock-tree.md) - PLL configuration for audio
- [Audio Interface](daisy-hardware/audio-interface.md) - SAI, DMA, codecs

### 4. Practical Guides
Apply your knowledge:
- [Build Process](build-process.md) - Detailed toolchain setup and flashing methods
- [Memory Constraints](memory.md) - STM32H750 memory layout and optimization
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
