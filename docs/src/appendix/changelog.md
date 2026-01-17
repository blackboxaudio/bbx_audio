# Changelog

All notable changes to bbx_audio.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- mdBook documentation with comprehensive guides
- Denormal handling support for Apple Silicon (AArch64)
- `ftz-daz` feature flag for flush-to-zero / denormals-are-zero mode
- `Block::prepare()` and `Graph::prepare()` methods for handling sample rate and context changes
- `Block::reset()` and `Graph::reset()` methods for clearing internal state without reconfiguration

### Changed
- Improved parameter initialization with dynamic approach
- GraphBuilder now connects all terminal blocks to output (fixes multi-oscillator graphs)

### Fixed
- Removed duplicate parameter index definitions
- Removed unnecessary PhantomData from GainBlock, OverdriveBlock, PannerBlock

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
