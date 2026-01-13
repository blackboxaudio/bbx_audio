# JUCE Plugin Integration Overview

bbx_audio provides a complete solution for writing audio plugin DSP in Rust while using JUCE for the UI and plugin framework.

## Architecture

```
JUCE AudioProcessor (C++)
         |
         v
   +-----------+
   | bbx::Graph|  <-- RAII C++ wrapper
   +-----------+
         |
         v  (C FFI calls)
   +------------+
   | bbx_ffi.h  |  <-- Generated C header
   +------------+
         |
         v
   +-------------+
   | PluginGraph |  <-- Your Rust DSP (implements PluginDsp)
   +-------------+
         |
         v
   +------------+
   | DSP Blocks |  <-- Gain, Panner, Filters, etc.
   +------------+
```

## Key Components

### Rust Side

- **`PluginDsp` trait** - Defines the interface your DSP must implement
- **`bbx_plugin_ffi!` macro** - Generates all C FFI exports automatically
- **Parameter system** - Define parameters in JSON or Rust, generate indices for both languages

### C++ Side

- **`bbx_ffi.h`** - C header with FFI function declarations and error codes
- **`bbx_graph.h`** - Header-only C++ RAII wrapper class
- **Parameter indices** - Generated `#define` constants matching Rust indices

## Quick Start Checklist

For experienced developers, here's the minimal setup:

1. Add Corrosion submodule: `git submodule add https://github.com/corrosion-rs/corrosion.git vendor/corrosion`
2. Create `dsp/Cargo.toml` with `bbx_plugin` dependency and `crate-type = ["staticlib"]`
3. Copy `bbx_ffi.h` and `bbx_graph.h` to `dsp/include/`
4. Implement `PluginDsp` trait and call `bbx_plugin_ffi!(YourType)`
5. Add Corrosion to CMakeLists.txt and link `dsp` to your plugin target
6. Use `bbx::Graph` in your AudioProcessor

For detailed guidance, follow the integration steps below.

## Integration Steps

1. [Project Setup](project-setup.md) - Directory structure and prerequisites
2. [Rust Crate Configuration](rust-crate.md) - Set up `Cargo.toml` and FFI headers
3. [CMake with Corrosion](cmake-setup.md) - Configure the build system
4. [Implementing PluginDsp](plugin-dsp.md) - Write your DSP processing chain
5. [Parameter System](parameters.md) - Define and manage plugin parameters
6. [AudioProcessor Integration](audio-processor.md) - Connect to JUCE
7. [Complete Example](complete-example.md) - Full working reference

**Reference Documentation:**
- [FFI Integration](ffi-integration.md) - C FFI layer details
- [C FFI Header Reference](ffi-header.md) - Complete `bbx_ffi.h` documentation
- [C++ RAII Wrapper](cpp-wrapper.md) - Using `bbx::Graph` in JUCE

## Benefits

- **Type Safety**: Rust's type system prevents memory bugs in DSP code
- **Performance**: Zero-cost abstractions with no runtime overhead
- **Separation**: Clean boundary between DSP logic and UI/framework code
- **Testability**: DSP can be tested independently of the plugin framework
- **Portability**: Same DSP code works with any C++-compatible framework

## Limitations

- **Build Complexity**: Requires Rust toolchain in addition to C++ build
- **Debug Boundaries**: Debugging across FFI requires care
- **No Hot Reload**: DSP changes require full rebuild and plugin reload
