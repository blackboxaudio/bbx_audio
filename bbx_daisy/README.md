# bbx_daisy

Electrosmith Daisy hardware support for bbx_audio.

This crate provides stack-allocated buffer types and hardware abstractions for running bbx_audio DSP on Electrosmith Daisy platforms (Seed, Pod, Patch SM, etc.).

## Features

- **Zero boilerplate**: Entry point macros handle all hardware initialization
- **No unsafe in user code**: State management handled safely by the library
- **Stack-allocated buffers**: `StaticSampleBuffer` and `FrameBuffer` for embedded targets without heap allocation
- **Embedded DSP context**: Memory-optimized `EmbeddedDspContext` for realtime processing
- **Hardware abstractions**: GPIO, ADC, encoder, and audio codec support
- **Pin mappings**: Pre-defined pin configurations for all Daisy variants

## Supported Boards

**IMPORTANT**: Only one product feature should be enabled at a time. The build system enforces this and will fail if multiple features are detected.

| Board | Feature Flag | Codec | SAI Config | DMA Config | Status |
|-------|--------------|-------|------------|------------|--------|
| Daisy Seed | `seed` | AK4556 | CH_A TX (master) | Stream 0→A, Stream 1→B | ✓ Hardware-verified |
| Daisy Seed 1.1 | `seed_1_1` | WM8731 | CH_B TX (slave) | Stream 0→B, Stream 1→A | Builds; hardware-unverified |
| Daisy Seed 1.2 | `seed_1_2` | PCM3060 | CH_A TX (master) | Stream 0→A, Stream 1→B | Builds; hardware-unverified |
| Daisy Pod | `pod` | WM8731 | CH_B TX (slave) | Stream 0→B, Stream 1→A | Builds; hardware-unverified |
| Patch SM | `patch_sm` | PCM3060 | CH_B TX (slave) | Stream 0→B, Stream 1→A | Builds; hardware-unverified |
| Patch.Init() | `patch_init` | PCM3060 | CH_B TX (slave)* | Stream 0→B, Stream 1→A | Builds; hardware-unverified |

> **Pod owners:** the codec lives on the *Seed* seated in the Pod carrier, and the
> `pod` feature assumes a Seed 1.1 (WM8731). A Pod holding an original AK4556
> Seed must build with `--features seed` — the carrier's audio jacks route the
> Seed's codec either way (verified on hardware). Carrier controls (knobs,
> encoder) currently require the `pod` feature; decoupling carrier from Seed
> revision is planned.
| Patch (alias) | `patch` | PCM3060 | CH_B TX (slave)* | Stream 0→B, Stream 1→A | ✓ Verified |
| Daisy Field | `field` | - | - | - | ✗ Not Implemented |

\* `patch_init` and `patch` both use `patch_sm` hardware configuration

**Note**: The SAI (Serial Audio Interface) and DMA configurations vary by board due to hardware design differences. The crate automatically selects the correct master/slave channel configuration and DMA stream assignments based on the feature flag. All configurations match the reference libDaisy implementation.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bbx_daisy = { version = "0.5.1", default-features = false, features = ["seed"] }
```

### Audio Processing

Implement `AudioProcessor` and use `bbx_daisy_audio!`:

```rust
#![no_std]
#![no_main]

use bbx_daisy::prelude::*;

struct SineOscillator {
    phase: f32,
    phase_inc: f32,
}

impl SineOscillator {
    fn new(frequency: f32) -> Self {
        Self {
            phase: 0.0,
            phase_inc: frequency / DEFAULT_SAMPLE_RATE,
        }
    }
}

impl AudioProcessor for SineOscillator {
    fn process(
        &mut self,
        _input: &FrameBuffer<BLOCK_SIZE>,
        output: &mut FrameBuffer<BLOCK_SIZE>,
        _controls: &Controls,
    ) {
        for i in 0..BLOCK_SIZE {
            let sample = sinf(self.phase * 2.0 * PI) * 0.5;
            output.set_frame(i, sample, sample);

            self.phase += self.phase_inc;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}

bbx_daisy_audio!(SineOscillator, SineOscillator::new(440.0));
```

### GPIO Applications

Use `bbx_daisy_run!` for non-audio applications:

```rust
#![no_std]
#![no_main]

use bbx_daisy::prelude::*;

fn blink(mut board: Board) -> ! {
    let led_pin = board.gpioc.pc7.into_push_pull_output();
    let mut led = Led::new(led_pin);

    loop {
        led.toggle();
        board.delay.delay_ms(500u16);
    }
}

bbx_daisy_run!(blink);
```

## Entry Point Macros

### `bbx_daisy_audio!`

For audio processing applications. Handles:
- Entry point setup (`#[cortex_m_rt::entry]`)
- Panic handler (`panic_halt`)
- Safe static state management for your `AudioProcessor`
- Audio callback registration
- Main loop with `wfi()`

### `bbx_daisy_run!`

For GPIO/general applications. Handles:
- Entry point setup
- Panic handler
- Hardware initialization (power, clocks, GPIO ports)
- Provides initialized `Board` struct with all peripherals ready

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

# Run cargo from this directory so .cargo/config.toml applies (it selects the
# thumbv7em-none-eabihf target and the dfu-util flash runner).
cd bbx_daisy

# Build for Daisy Seed (there is no default board — always pick exactly one)
cargo build --features seed --release

# Build an example (the in-repo patches; seed examples require the `seed` feature)
cargo build --example 01_blink --features seed --release

# Build for other variants
cargo build --features pod --release
cargo build --features seed_1_1 --release
cargo build --features seed_1_2 --release
cargo build --features patch_sm --release

# The build system enforces mutual exclusivity - this will fail:
# cargo build --features "seed,pod"  # ERROR: Multiple features enabled
```

## CPU caches (`dcache` feature)

By default the Cortex-M7 instruction and data caches are **off**. The audio DMA buffers
live in D2 SRAM and are accessed directly by both the CPU and the DMA, so they stay
coherent with no maintenance — this is the hardware-verified default.

The optional `dcache` feature turns the caches **on** and adds the matching DMA-buffer
cache maintenance to the audio interrupt (invalidate the RX buffer before reading, clean
the TX buffer after writing). The two are flipped together by this single flag, and they
have to be: running the maintenance ops with the D-cache disabled raises a `BusFault`,
while enabling the D-cache without maintenance feeds the codec stale data.

**Enable it when your DSP is CPU-bound and you need more headroom.** The caches are a
large speedup on the STM32H750 — the instruction cache especially, since flash has several
wait states at 400+ MHz — and the data cache helps data-heavy work (big wavetables, long
delay lines). If the default already keeps up, leave it off: it's the simpler, verified path.

```toml
bbx_daisy = { version = "0.5.1", default-features = false, features = ["seed", "dcache"] }
```

**Before shipping with it on:**

- Listen to the audio first — only the cache-off default is hardware-verified here. The
  cached path follows the reference daisy/libDaisy design, but confirm there are no glitches.
- Any DMA _you_ add while the D-cache is on is yours to keep coherent: invalidate before the
  CPU reads DMA-written data, clean after the CPU writes data the DMA reads, and keep those
  buffers 32-byte (cache-line) aligned. Only the built-in audio buffers are handled for you.

## Flashing to Hardware

Patches flash over USB DFU using `dfu-util` — no debug probe required. The build
targets internal flash (`0x08000000`), so programs up to 128 KB run directly
without the Daisy bootloader. (The examples are ~10–20 KB.)

### Prerequisites

Install `dfu-util` and the ARM GNU toolchain (for `arm-none-eabi-objcopy`):

```bash
# macOS
brew install dfu-util arm-none-eabi-binutils

# Ubuntu/Debian
sudo apt install dfu-util binutils-arm-none-eabi

# Arch
sudo pacman -S dfu-util arm-none-eabi-binutils
```

### Enter DFU mode

The STM32H750 has a built-in USB bootloader. To enter DFU mode:

1. Hold the **BOOT** button on the Daisy
2. Tap the **RESET** button (or power cycle)
3. Release **BOOT**

The Daisy enumerates as a DFU device (VID: `0x0483`, PID: `0xDF11`).

### Flash with `cargo run`

`dfu-util` is wired up as the cargo runner via `scripts/flash-dfu.sh`. Run cargo
from this directory; `cargo run` builds the example, converts the ELF to a raw
`.bin`, and flashes it:

```bash
cd bbx_daisy
cargo run --example 01_blink --features seed --release
cargo run --example 02_oscillator --features seed --release
```

### Flash a prebuilt binary

Run the script directly with any built ELF:

```bash
./scripts/flash-dfu.sh ../target/thumbv7em-none-eabihf/release/examples/01_blink
```

Or do it by hand:

```bash
arm-none-eabi-objcopy -O binary \
    ../target/thumbv7em-none-eabihf/release/examples/01_blink \
    01_blink.bin
dfu-util -a 0 -s 0x08000000:leave -D 01_blink.bin -d ,0483:df11
```

### Using a debug probe instead (optional)

If you have an ST-Link/J-Link wired to the Daisy's SWD pads, you can use
[probe-rs](https://probe.rs) instead of DFU:

```bash
cargo install probe-rs-tools
probe-rs run --chip STM32H750VBTx ../target/thumbv7em-none-eabihf/release/examples/01_blink
```

To make `cargo run` use it, set the runner in `.cargo/config.toml` to
`runner = "probe-rs run --chip STM32H750VBTx"`.

| SWD signal | Daisy Seed pin |
|------------|----------------|
| SWDIO      | Pin 30 (PA13)  |
| SWCLK      | Pin 29 (PA14)  |
| GND        | GND            |

### Troubleshooting

**`dfu-util: No DFU capable USB device available`**
- The Daisy isn't in DFU mode — hold BOOT, tap RESET, release BOOT, then retry.
- Make sure the USB cable carries data (not charge-only).

**Build "succeeds" but the device does nothing, or `cargo run` can't find the runner**
- Run cargo from the `bbx_daisy/` directory. Cargo reads `.cargo/config.toml`
  from the current directory upward, so building from the workspace root skips the
  ARM target and the flash runner.

**Program too large to flash**
- Internal flash is 128 KB. Trim the patch, or move to a QSPI/bootloader memory
  layout (not currently configured).

**DFU device not detected**
- Ensure you entered DFU mode correctly (LED should not blink)
- On Linux, add udev rule for STM32 DFU: `ATTRS{idVendor}=="0483", ATTRS{idProduct}=="df11", MODE="0666"`
