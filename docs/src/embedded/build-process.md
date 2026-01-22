# Build Process

This guide covers toolchain installation, cargo configuration, and flashing methods for Daisy embedded development.

## Toolchain Installation

### 1. Install ARM Target

```bash
rustup target add thumbv7em-none-eabihf
```

This target supports the ARM Cortex-M7 with hardware floating point (used by STM32H750).

### 2. Install probe-rs (Recommended)

probe-rs provides flashing and debugging via ST-Link, J-Link, or CMSIS-DAP probes:

```bash
# macOS
brew install probe-rs

# Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

# Windows
winget install probe-rs
```

### 3. Install dfu-util (Alternative)

For USB flashing without a debug probe:

```bash
# macOS
brew install dfu-util

# Linux (Debian/Ubuntu)
sudo apt install dfu-util

# Windows
# Download from http://dfu-util.sourceforge.net/
```

## Cargo Configuration

The repository includes `.cargo/config.toml` for bbx_daisy with the ARM target and probe-rs runner pre-configured. Key settings:

```toml
[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32H750VBTx"
rustflags = ["-C", "link-arg=-Tlink.x"]
```

## Feature Flag Selection

Select your board variant via feature flags:

| Board | Feature | Cargo Flag |
|-------|---------|------------|
| Daisy Seed | `seed` | `--features seed` (default) |
| Daisy Seed 1.1 | `seed_1_1` | `--features seed_1_1` |
| Daisy Seed 1.2 | `seed_1_2` | `--features seed_1_2` |
| Daisy Pod | `pod` | `--features pod` |
| Patch SM | `patch_sm` | `--features patch_sm` |
| Patch.Init() | `patch_init` | `--features patch_init` |

Only one board feature should be enabled at a time.

## Building

```bash
# Build the crate
cargo build -p bbx_daisy --target thumbv7em-none-eabihf --features seed --release

# Build a specific example
cargo build -p bbx_daisy --example 02_oscillator --target thumbv7em-none-eabihf --release
```

Always use `--release` for production builds to enable optimizations critical for realtime audio.

## Flashing Methods

### Method 1: probe-rs (Recommended)

Connect your debug probe (ST-Link, J-Link, or CMSIS-DAP) and run:

```bash
cargo run -p bbx_daisy --example 02_oscillator --release
```

This builds and flashes in one step. The probe-rs runner is configured in `.cargo/config.toml`.

### Method 2: DFU (No Debug Probe)

1. Enter DFU mode on the Daisy:
   - Hold the **BOOT** button
   - Tap the **RESET** button
   - Release **BOOT**

2. Flash via USB:
```bash
dfu-util -a 0 -s 0x08000000:leave -D target/thumbv7em-none-eabihf/release/examples/02_oscillator
```

The `:leave` suffix causes the device to exit DFU mode and run the firmware after flashing.

## Debugging with RTT

Real-Time Transfer (RTT) provides printf-style debugging over the debug probe without UART:

```rust
use defmt::info;

info!("Sample rate: {}", DEFAULT_SAMPLE_RATE);
```

View output with:

```bash
probe-rs run --chip STM32H750VBTx target/thumbv7em-none-eabihf/release/examples/02_oscillator
```

RTT output appears in the terminal alongside the flashing progress.

## Optimization Flags

For maximum performance, the release profile in `Cargo.toml` includes:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
debug = false
```

These settings produce the smallest, fastest binary but increase compile time.
