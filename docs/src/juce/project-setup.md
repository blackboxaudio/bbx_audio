# Project Setup

This guide walks through setting up a JUCE plugin project with Rust DSP.

## Directory Structure

A typical project structure:

```
my-plugin/
├── CMakeLists.txt
├── dsp/                      # Rust DSP crate
│   ├── Cargo.toml
│   ├── include/
│   │   ├── bbx_ffi.h        # C FFI header
│   │   └── bbx_graph.h      # C++ RAII wrapper
│   └── src/
│       └── lib.rs           # PluginDsp implementation
├── src/                      # JUCE plugin source
│   ├── PluginProcessor.cpp
│   ├── PluginProcessor.h
│   ├── PluginEditor.cpp
│   └── PluginEditor.h
└── vendor/
    └── corrosion/           # Git submodule
```

## Prerequisites

1. **Rust toolchain** - Install from [rustup.rs](https://rustup.rs)
2. **CMake 3.15+** - For building the plugin
3. **JUCE** - Framework for the plugin
4. **Corrosion** - CMake integration for Rust

### Adding Corrosion

Add Corrosion as a git submodule:

```bash
git submodule add https://github.com/corrosion-rs/corrosion.git vendor/corrosion
```

## Next Steps

- [Rust Crate Configuration](rust-crate.md) - Set up `Cargo.toml`
- [CMake with Corrosion](cmake-setup.md) - Configure the build system
