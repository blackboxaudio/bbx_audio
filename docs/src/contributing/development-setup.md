# Development Setup

Set up your environment for contributing to bbx_audio.

## Prerequisites

### Rust Toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add nightly toolchain (required for fmt and clippy)
rustup toolchain install nightly
```

### Platform Dependencies

**Linux:**
```bash
sudo apt install alsa libasound2-dev libssl-dev pkg-config
```

**macOS/Windows:** No additional dependencies.

## Clone and Build

```bash
# Clone repository
git clone https://github.com/blackboxaudio/bbx_audio.git
cd bbx_audio

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace --release
```

## IDE Setup

### VS Code

Recommended extensions:
- rust-analyzer
- Even Better TOML
- CodeLLDB (debugging)

Settings (`.vscode/settings.json`):
```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.check.command": "clippy"
}
```

### Other IDEs

- **IntelliJ IDEA** - Use Rust plugin
- **Vim/Neovim** - Use rust-analyzer LSP
- **Emacs** - Use rustic-mode

## Development Workflow

```bash
# Format code
cargo +nightly fmt

# Run lints
cargo +nightly clippy

# Run tests
cargo test --workspace --release

# Run an example
cargo run --release --example 01_sine_wave -p bbx_sandbox
```

## Common Tasks

### Adding a New Block

1. Create block in appropriate `blocks/` subdirectory
2. Add to `BlockType` enum
3. Add builder method to `GraphBuilder`
4. Write tests
5. Update documentation

### Modifying FFI

1. Update Rust code in `bbx_plugin`
2. Regenerate header with cbindgen
3. Update `bbx_graph.h` if needed
4. Test with JUCE project
