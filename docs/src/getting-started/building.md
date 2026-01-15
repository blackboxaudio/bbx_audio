# Building from Source

This guide covers building bbx_audio from source for development or contribution.

## Prerequisites

### Rust Toolchain

bbx_audio requires Rust with the nightly toolchain for some development features:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add nightly toolchain
rustup toolchain install nightly

# Verify installation
rustc --version
cargo --version
```

### Platform Dependencies

#### Linux

```bash
sudo apt install libasound2-dev libssl-dev pkg-config
```

#### macOS / Windows

No additional dependencies required.

## Clone and Build

```bash
# Clone the repository
git clone https://github.com/blackboxaudio/bbx_audio.git
cd bbx_audio

# Build all crates
cargo build --workspace

# Build in release mode
cargo build --workspace --release
```

## Running Tests

```bash
# Run all tests
cargo test --workspace --release
```

## Code Quality

### Formatting

bbx_audio uses the nightly formatter:

```bash
cargo +nightly fmt
```

### Linting

Clippy is used for linting:

```bash
cargo +nightly clippy
```

## Running Examples

The `bbx_sandbox` crate contains example programs:

```bash
# List available examples
ls bbx_sandbox/examples/

# Run an example
cargo run --release --example <example_name> -p bbx_sandbox
```

## Generating Documentation

Generate rustdoc documentation:

```bash
cargo doc --workspace --no-deps --open
```

## Project Structure

```
bbx_audio/
├── bbx_core/       # Foundational utilities
├── bbx_dsp/        # DSP graph system
├── bbx_file/       # Audio file I/O
├── bbx_midi/       # MIDI handling
├── bbx_plugin/     # FFI bindings
├── bbx_sandbox/    # Examples
├── docs/           # This documentation
└── Cargo.toml      # Workspace manifest
```

## Build Configuration

The workspace uses Rust 2024 edition. Key settings in the root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "bbx_core",
    "bbx_dsp",
    "bbx_file",
    "bbx_midi",
    "bbx_plugin",
    "bbx_sandbox",
]
```

## Troubleshooting

### "toolchain not found" Error

Install the nightly toolchain:

```bash
rustup toolchain install nightly
```

### Audio Device Errors on Linux

Ensure ALSA development packages are installed:

```bash
sudo apt install libasound2-dev
```

### Slow Builds

Use release mode for faster runtime performance:

```bash
cargo build --release
```

For faster compile times during development, use debug mode:

```bash
cargo build
```
