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

| Board | Feature Flag | Codec | Status |
|-------|--------------|-------|--------|
| Daisy Seed | `seed` (default) | AK4556 | Ready |
| Daisy Seed 1.1 | `seed_1_1` | WM8731 | Ready |
| Daisy Seed 1.2 | `seed_1_2` | PCM3060 | Ready |
| Daisy Pod | `pod` | WM8731 | Ready |
| Patch SM | `patch_sm` | PCM3060 | Ready |
| Patch.Init() | `patch_init` | PCM3060 | Ready |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bbx_daisy = { version = "0.4.3", default-features = false, features = ["seed"] }
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
    fn process(&mut self, _input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
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

# Build for Daisy Seed
cargo build -p bbx_daisy --target thumbv7em-none-eabihf --release

# Build for other variants
cargo build -p bbx_daisy --target thumbv7em-none-eabihf --features pod --release
cargo build -p bbx_daisy --target thumbv7em-none-eabihf --features patch_sm --release
```

## Flashing to Hardware

### Prerequisites

Install probe-rs for flashing and debugging:

```bash
cargo install probe-rs-tools
```

You'll need one of the following debug probes:
- **ST-Link V2/V3** (included with STM32 Nucleo/Discovery boards)
- **J-Link** (Segger)
- **DAPLink** / **CMSIS-DAP** compatible probes

Alternatively, you can flash via USB bootloader (DFU) without a debug probe.

### Flashing with probe-rs (Recommended)

The crate is pre-configured with probe-rs as the cargo runner. Connect your debug probe to the Daisy's SWD pins and run:

```bash
# Flash and run an example (probe-rs is the configured runner)
cargo run --example 01_blink --release

# Or specify the target explicitly
cargo run --example 02_oscillator --target thumbv7em-none-eabihf --release
```

For pre-built binaries, use probe-rs directly:

```bash
probe-rs run --chip STM32H750VBTx target/thumbv7em-none-eabihf/release/examples/01_blink
```

**SWD Pin Connections:**
| Debug Probe | Daisy Seed Pin |
|-------------|----------------|
| SWDIO       | Pin 30 (PA13)  |
| SWCLK       | Pin 29 (PA14)  |
| GND         | GND            |
| 3.3V        | 3V3 (optional) |

### Flashing with DFU (No Debug Probe)

The STM32H750 has a built-in USB bootloader. To enter DFU mode:

1. Hold the **BOOT** button on the Daisy
2. Tap the **RESET** button (or power cycle)
3. Release **BOOT**

The Daisy will enumerate as a DFU device (VID: 0x0483, PID: 0xDF11).

Install dfu-util:

```bash
# macOS
brew install dfu-util

# Ubuntu/Debian
sudo apt install dfu-util

# Arch
sudo pacman -S dfu-util
```

Convert and flash:

```bash
# Convert ELF to BIN
llvm-objcopy -O binary \
    target/thumbv7em-none-eabihf/release/examples/01_blink \
    01_blink.bin

# Flash via DFU (address 0x08000000 is internal flash)
dfu-util -a 0 -s 0x08000000:leave -D 01_blink.bin
```

### Troubleshooting

**"No probe found"**
- Verify USB connection and that the probe is powered
- On Linux, you may need udev rules. Create `/etc/udev/rules.d/99-probe-rs.rules`:
  ```
  # ST-Link
  ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374*", MODE="0666"
  # J-Link
  ATTRS{idVendor}=="1366", ATTRS{idProduct}=="*", MODE="0666"
  ```
  Then run `sudo udevadm control --reload-rules && sudo udevadm trigger`

**"Target not found" or "Chip not detected"**
- Ensure the chip is `STM32H750VBTx` (verify in `.cargo/config.toml`)
- Check SWD connections and that the Daisy is powered

**DFU device not detected**
- Ensure you entered DFU mode correctly (LED should not blink)
- On Linux, add udev rule for STM32 DFU: `ATTRS{idVendor}=="0483", ATTRS{idProduct}=="df11", MODE="0666"`
