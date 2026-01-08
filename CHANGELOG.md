# Changelog

All notable changes to bbx_audio will be documented in this file.

## [0.2.1] - 2026-01-08

### Bug Fixes

- Create index.html file for docs

### Features

- Add support for embedding into JUCE projects (#37) ([#37](https://github.com/blackboxaudio/bbx_audio/pull/37))
- Add basic synthesizer support (#43) ([#43](https://github.com/blackboxaudio/bbx_audio/pull/43))

### Miscellaneous

- Use MD book in documentation
- Update README

### Refactor

- New lib design (#33) ([#33](https://github.com/blackboxaudio/bbx_audio/pull/33))
- Improve parameter initialization (#41) ([#41](https://github.com/blackboxaudio/bbx_audio/pull/41))

## [0.2.0] - 2025-12-30

### Features

- Add support for embedding into JUCE projects (#37) ([#37](https://github.com/blackboxaudio/bbx_audio/pull/37))

### Refactor

- New lib design (#33) ([#33](https://github.com/blackboxaudio/bbx_audio/pull/33))
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
