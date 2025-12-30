# Changelog

All notable changes to bbx_audio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-XX-XX

### Added

- Initial release of bbx_audio workspace
- `bbx_core`: Foundational utilities (SPSC ring buffer, denormal handling, stack vector, RNG)
- `bbx_dsp`: Block-based DSP graph system with oscillators, effects, and modulators
- `bbx_midi`: MIDI message parsing and streaming
- `bbx_file`: WAV file I/O integration
- `bbx_plugin`: FFI bindings for C/C++ plugin integration
