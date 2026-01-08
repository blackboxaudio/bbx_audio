# Summary

[Introduction](README.md)

# Getting Started

- [Installation](getting-started/installation.md)
- [Quick Start](getting-started/quick-start.md)
- [Building from Source](getting-started/building.md)

# Tutorials

- [Your First DSP Graph](tutorials/first-graph.md)
- [Creating a Simple Oscillator](tutorials/oscillator.md)
- [Adding Effects](tutorials/effects.md)
- [Parameter Modulation with LFOs](tutorials/modulation.md)
- [Working with Audio Files](tutorials/audio-files.md)
- [MIDI Integration](tutorials/midi.md)

# JUCE Plugin Integration

- [Overview](juce/overview.md)
- [Project Setup](juce/project-setup.md)
    - [Rust Crate Configuration](juce/rust-crate.md)
    - [CMake with Corrosion](juce/cmake-setup.md)
- [Implementing PluginDsp](juce/plugin-dsp.md)
- [Parameter System](juce/parameters.md)
    - [parameters.json Format](juce/parameters-json.md)
    - [Programmatic Definition](juce/parameters-programmatic.md)
    - [Code Generation](juce/parameters-codegen.md)
- [FFI Integration](juce/ffi-integration.md)
    - [C FFI Header Reference](juce/ffi-header.md)
    - [C++ RAII Wrapper](juce/cpp-wrapper.md)
- [AudioProcessor Integration](juce/audio-processor.md)
- [Complete Example Walkthrough](juce/complete-example.md)

# Crate Reference

- [bbx_core](crates/bbx-core.md)
    - [Denormal Handling](crates/core/denormal.md)
    - [SPSC Ring Buffer](crates/core/spsc.md)
    - [Stack Vector](crates/core/stack-vec.md)
    - [Random Number Generation](crates/core/random.md)
    - [Error Types](crates/core/error.md)
- [bbx_dsp](crates/bbx-dsp.md)
    - [Graph and GraphBuilder](crates/dsp/graph.md)
    - [Block Trait](crates/dsp/block-trait.md)
    - [BlockType Enum](crates/dsp/block-type.md)
    - [Sample Trait](crates/dsp/sample.md)
    - [DspContext](crates/dsp/context.md)
    - [Parameter System](crates/dsp/parameters.md)
- [bbx_plugin](crates/bbx-plugin.md)
    - [PluginDsp Trait](crates/plugin/plugin-dsp.md)
    - [FFI Macro](crates/plugin/ffi-macro.md)
    - [Parameter Definitions](crates/plugin/params.md)
- [bbx_file](crates/bbx-file.md)
    - [WAV Reader](crates/file/wav-reader.md)
    - [WAV Writer](crates/file/wav-writer.md)
- [bbx_midi](crates/bbx-midi.md)
    - [MIDI Messages](crates/midi/messages.md)
    - [Message Buffer](crates/midi/buffer.md)

# Blocks Reference

- [Generators](blocks/generators.md)
    - [OscillatorBlock](blocks/generators/oscillator.md)
- [Effectors](blocks/effectors.md)
    - [GainBlock](blocks/effectors/gain.md)
    - [PannerBlock](blocks/effectors/panner.md)
    - [OverdriveBlock](blocks/effectors/overdrive.md)
    - [DcBlockerBlock](blocks/effectors/dc-blocker.md)
    - [ChannelRouterBlock](blocks/effectors/channel-router.md)
    - [LowPassFilterBlock](blocks/effectors/low-pass-filter.md)
- [Modulators](blocks/modulators.md)
    - [LfoBlock](blocks/modulators/lfo.md)
    - [EnvelopeBlock](blocks/modulators/envelope.md)
- [I/O Blocks](blocks/io.md)
    - [FileInputBlock](blocks/io/file-input.md)
    - [FileOutputBlock](blocks/io/file-output.md)
    - [OutputBlock](blocks/io/output.md)

# Architecture Deep-Dives

- [DSP Graph Architecture](architecture/graph-architecture.md)
    - [Topological Sorting](architecture/topological-sort.md)
    - [Buffer Management](architecture/buffer-management.md)
    - [Connection System](architecture/connections.md)
- [Real-Time Safety](architecture/realtime-safety.md)
    - [Stack Allocation Strategy](architecture/stack-allocation.md)
    - [Denormal Prevention](architecture/denormals.md)
    - [Lock-Free Patterns](architecture/lock-free.md)
- [Modulation System](architecture/modulation.md)
    - [Parameter<S> Type](architecture/parameter-type.md)
    - [Modulation Value Collection](architecture/modulation-values.md)
- [FFI Design](architecture/ffi-design.md)
    - [Handle Management](architecture/handle-management.md)
    - [Memory Safety Across Boundaries](architecture/memory-safety.md)
- [Performance Considerations](architecture/performance.md)
    - [Zero-Allocation Processing](architecture/zero-allocation.md)
    - [Cache Efficiency](architecture/cache-efficiency.md)
    - [SIMD Opportunities](architecture/simd.md)

# Contributing

- [Development Setup](contributing/development-setup.md)
- [Code Style](contributing/code-style.md)
- [Adding New Blocks](contributing/new-blocks.md)
- [Testing](contributing/testing.md)
- [Release Process](contributing/releasing.md)

# Appendix

- [Changelog](appendix/changelog.md)
- [Migration Guide](appendix/migration.md)
- [Troubleshooting](appendix/troubleshooting.md)
- [Glossary](appendix/glossary.md)
