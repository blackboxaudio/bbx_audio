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

## Integration Steps

1. [Project Setup](project-setup.md) - Create the Rust crate and CMake configuration
2. [Implementing PluginDsp](plugin-dsp.md) - Write your DSP processing chain
3. [Parameter System](parameters.md) - Define and manage plugin parameters
4. [FFI Integration](ffi-integration.md) - Understand the C FFI layer
5. [AudioProcessor Integration](audio-processor.md) - Connect to JUCE

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
