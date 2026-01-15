# Troubleshooting

Common issues and solutions.

## Build Issues

### "toolchain not found"

This workspace requires Rust nightly. The version is pinned in `rust-toolchain.toml` (`nightly-2025-06-08`), so `rustup` should automatically select it when you enter the project directory. If you don't have it installed:

```bash
rustup toolchain install nightly-2025-06-08
```

### Linux Audio Errors

Install ALSA development packages:

```bash
sudo apt install libasound2-dev
```

### Slow Compilation

Use release mode for faster runtime:

```bash
cargo build --release
```

## Runtime Issues

### No Audio Output

1. Check audio device is available
2. Verify sample rate matches system
3. Ensure output block is connected

### Crackling/Glitches

1. Increase buffer size
2. Check CPU usage
3. Avoid allocations in audio thread
4. Profile for bottlenecks

### Silence

1. Verify block connections
2. Check gain levels (not -inf dB)
3. Ensure `prepare()` was called

## FFI Issues

### "Cannot find -ldsp"

1. Ensure Rust crate builds successfully
2. Check Corrosion configuration
3. Verify `staticlib` crate type

### Header Not Found

Verify `target_include_directories` in CMake:

```cmake
target_include_directories(${TARGET} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/dsp/include)
```

### Linking Errors

Check crate type in `Cargo.toml`:

```toml
[lib]
crate-type = ["staticlib"]
```

## Parameter Issues

### Parameters Not Updating

1. Verify parameter indices match
2. Check `apply_parameters()` implementation
3. Ensure JUCE is calling `Process()` with params

### Wrong Parameter Values

1. Verify JSON/code generation sync
2. Check parameter count matches
3. Debug print received values

## Getting Help

1. Check [GitHub Issues](https://github.com/blackboxaudio/bbx_audio/issues)
2. Search existing discussions
3. Open a new issue with:
   - bbx_audio version
   - Platform and OS
   - Minimal reproduction
   - Error messages
