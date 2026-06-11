# Build Process

This guide covers toolchain installation, cargo configuration, and flashing methods for Daisy embedded development.

## Toolchain Installation

### 1. Install ARM Target

```bash
rustup target add thumbv7em-none-eabihf
```

This target supports the ARM Cortex-M7 with hardware floating point (used by STM32H750).

### 2. Install dfu-util and the ARM toolchain

Patches flash over USB DFU — no debug probe required. You need `dfu-util` and
`arm-none-eabi-objcopy` (from the ARM GNU binutils):

```bash
# macOS
brew install dfu-util arm-none-eabi-binutils

# Linux (Debian/Ubuntu)
sudo apt install dfu-util binutils-arm-none-eabi

# Windows
# Download dfu-util from http://dfu-util.sourceforge.net/ and install the
# Arm GNU Toolchain from https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads
```

### 3. Install probe-rs (optional)

Only needed if you have a debug probe (ST-Link, J-Link, or CMSIS-DAP) and prefer
SWD flashing with RTT logging. See [Using a debug probe](#using-a-debug-probe-optional).

## Cargo Configuration

`bbx_daisy/.cargo/config.toml` selects the ARM target and wires `dfu-util` as the
cargo runner:

```toml
[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
runner = "./scripts/flash-dfu.sh"
```

> **Run cargo from the `bbx_daisy/` directory.** Cargo discovers `.cargo/config.toml`
> from the current directory upward — not from the package being built — so building
> from the workspace root skips both the target and the runner. The cortex-m-rt
> linker script (`-Tlink.x`) is applied from `build.rs` (`cargo:rustc-link-arg-examples`)
> so linking still succeeds if you do build from the root with an explicit
> `--target thumbv7em-none-eabihf`.

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

Only one board feature should be enabled at a time. Use `--no-default-features`
when selecting a non-`seed` variant to avoid enabling `seed` as well.

## Building

```bash
cd bbx_daisy

# Build the crate (default: seed)
cargo build --release

# Build a specific example
cargo build --example 02_oscillator --release

# Build for another variant
cargo build --no-default-features --features patch_sm --release
```

Always use `--release` for production builds to enable optimizations critical for realtime audio.

## Flashing Methods

### Method 1: DFU with `cargo run` (recommended)

1. Enter DFU mode on the Daisy:
   - Hold the **BOOT** button
   - Tap the **RESET** button
   - Release **BOOT**

   The Daisy enumerates as a DFU device (VID `0x0483`, PID `0xDF11`).

2. Build and flash in one step:

   ```bash
   cd bbx_daisy
   cargo run --example 02_oscillator --release
   ```

   The runner (`scripts/flash-dfu.sh`) converts the ELF to a raw `.bin` with
   `arm-none-eabi-objcopy`, then flashes it with `dfu-util`. The `:leave` suffix
   causes the device to exit DFU mode and run the firmware after flashing.

### Method 2: Flash a prebuilt binary

```bash
cd bbx_daisy
./scripts/flash-dfu.sh ../target/thumbv7em-none-eabihf/release/examples/02_oscillator
```

Or by hand:

```bash
arm-none-eabi-objcopy -O binary \
    ../target/thumbv7em-none-eabihf/release/examples/02_oscillator \
    02_oscillator.bin
dfu-util -a 0 -s 0x08000000:leave -D 02_oscillator.bin -d ,0483:df11
```

> `dfu-util` needs a **raw binary** (`.bin`), not the ELF. Passing the ELF directly
> will fail or flash garbage.

### Using a debug probe (optional)

With an ST-Link/J-Link on the SWD pads, flash with probe-rs:

```bash
cargo install probe-rs-tools
probe-rs run --chip STM32H750VBTx ../target/thumbv7em-none-eabihf/release/examples/02_oscillator
```

To make `cargo run` use probe-rs, set the runner in `.cargo/config.toml` to
`runner = "probe-rs run --chip STM32H750VBTx"`.

## Logging / Debugging

By default there is **no logging configured** — the panic handler is `panic-halt`,
which simply halts the core on panic. There is no UART or RTT logger wired up.

To add `defmt` + RTT logging (requires a debug probe), you would:

1. Add `defmt`, `defmt-rtt`, and a `defmt`-based panic handler to `Cargo.toml`.
2. Add `-C link-arg=-Tdefmt.x` to the linker args (the `defmt` crate supplies that script).
3. Switch the runner to `probe-rs run --chip STM32H750VBTx` so RTT output is captured.

This is intentionally left out of the default configuration to keep the DFU
(no-probe) workflow self-contained.

## Optimization Flags

For maximum performance on the audio thread, add a release profile to the
workspace `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
debug = false
```

These settings produce the smallest, fastest binary but increase compile time.
