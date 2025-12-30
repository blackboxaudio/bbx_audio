# Changelog

All notable changes to bbx_audio.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- mdBook documentation

### Changed
- (none)

### Fixed
- (none)

## [0.1.0] - Initial Release

### Added

- **bbx_core**: Foundational utilities
  - Denormal handling
  - SPSC ring buffer
  - Stack-allocated vector
  - XorShift RNG
  - Error types

- **bbx_dsp**: DSP graph system
  - Block trait and BlockType enum
  - Graph and GraphBuilder
  - Topological sorting
  - Parameter modulation
  - Blocks: Oscillator, Gain, Panner, Overdrive, DC Blocker, Channel Router, LFO, Envelope, File I/O, Output

- **bbx_file**: Audio file I/O
  - WAV reading via wavers
  - WAV writing via hound

- **bbx_midi**: MIDI utilities
  - Message parsing
  - Message buffer
  - Input streaming

- **bbx_plugin**: Plugin integration
  - PluginDsp trait
  - FFI macro
  - Parameter definitions
  - C/C++ headers

- **bbx_sandbox**: Examples
  - Sine wave generation
  - PWM synthesis
  - Overdrive effect
  - WAV file I/O
  - MIDI input

---

For the full changelog, see [CHANGELOG.md](https://github.com/blackboxaudio/bbx_audio/blob/develop/CHANGELOG.md).
