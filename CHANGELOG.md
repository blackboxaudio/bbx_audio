# Changelog

All notable changes to bbx_audio will be documented in this file.

## [0.5.0] - 2026-08-06

### Features

- Add Electrosmith Daisy hardware support: new no_std `bbx_daisy` crate with SAI/DMA audio, board init macros, codec drivers, and on-device examples (#116) ([#116](https://github.com/blackboxaudio/bbx_audio/pull/116))
- Add `no_std` support to `bbx_core`, `bbx_dsp`, and `bbx_midi` via a `std`/`alloc` feature lattice (#116) ([#116](https://github.com/blackboxaudio/bbx_audio/pull/116))
- Route all DSP math through `libm` (`bbx_core::math::Real`) for bit-identical results across std and no_std builds (#116) ([#116](https://github.com/blackboxaudio/bbx_audio/pull/116))
- Add `Source` trait to `bbx_player` for playback abstractions
- Enable the STM32H750 double-precision FPU in embedded builds (`-C target-cpu=cortex-m7`)
- Add `08_cycle_check` example measuring on-device DSP cost with the DWT cycle counter

### Bug Fixes

- Add `prepare`/`reset` methods to handle DSP context changes (#112) ([#112](https://github.com/blackboxaudio/bbx_audio/pull/112))
- Add a `std` feature (default) to `bbx_plugin` so it builds without `simd`
- Harden the Daisy audio ISR: startup ordering vs NVIC unmask, explicit interrupt priority, atomic control sharing, cache-line-guaranteed DMA buffers, bounded SAI startup wait

### Breaking Changes

- Rename `AudioBuffer` to `SampleBuffer`; move the `Buffer` trait to `bbx_core` (#116) ([#116](https://github.com/blackboxaudio/bbx_audio/pull/116))
- Combine `Graph` prepare methods (#114) ([#114](https://github.com/blackboxaudio/bbx_audio/pull/114))
- `bbx_daisy`'s `set_callback` and `init_and_start` now return `Result<(), AudioError>`

## [0.4.3] - 2026-01-15

### Bug Fixes

- Improve file output (#82) ([#82](https://github.com/blackboxaudio/bbx_audio/pull/82))

### Features

- Add offline rendering (#83) ([#83](https://github.com/blackboxaudio/bbx_audio/pull/83))
- Add `bbx_player` crate for playback abstractions (#84) ([#84](https://github.com/blackboxaudio/bbx_audio/pull/84))

## [0.4.2] - 2026-01-14

### Features

- Include websocket client npm package (#77) ([#77](https://github.com/blackboxaudio/bbx_audio/pull/77))

## [0.4.1] - 2026-01-13

### Features

- Enable network-based audio control (#73) ([#73](https://github.com/blackboxaudio/bbx_audio/pull/73))

## [0.4.0] - 2026-01-13

### Bug Fixes

- Add filter resonance compensation curve (#67) ([#67](https://github.com/blackboxaudio/bbx_audio/pull/67))

### Features

- Add support for channel layouts (#69) ([#69](https://github.com/blackboxaudio/bbx_audio/pull/69))

### Refactor

- Use PolyBLEP algorithms (#65) ([#65](https://github.com/blackboxaudio/bbx_audio/pull/65))

## [0.3.1] - 2026-01-10

### Bug Fixes

- Synth voice not reset (#48) ([#48](https://github.com/blackboxaudio/bbx_audio/pull/48))

### Features

- Add MIDI message processing (#51) ([#51](https://github.com/blackboxaudio/bbx_audio/pull/51))

### Miscellaneous

- Add SIMD optimizations (#49) ([#49](https://github.com/blackboxaudio/bbx_audio/pull/49))

## [0.3.0] - 2026-01-08

### Added

- Add `bbx_draw` crate for audio visualizations (#45) ([#45](https://github.com/blackboxaudio/bbx_audio/pull/45))

## [0.2.1] - 2026-01-08

### Bug Fixes

- Create index.html file for docs

### Features

- Add basic synthesizer support (#43) ([#43](https://github.com/blackboxaudio/bbx_audio/pull/43))

### Miscellaneous

- Use MD book in documentation
- Update README

### Refactor

- Improve parameter initialization (#41) ([#41](https://github.com/blackboxaudio/bbx_audio/pull/41))

## [0.2.0] - 2025-12-30

### Features

- Add support for embedding into JUCE projects (#37) ([#37](https://github.com/blackboxaudio/bbx_audio/pull/37))

### Refactor

- New lib design (#33) ([#33](https://github.com/blackboxaudio/bbx_audio/pull/33))

## [0.1.0] - 2024-XX-XX

### Added

- Initial release of bbx_audio workspace
- `bbx_core`: Foundational utilities (SPSC ring buffer, denormal handling, stack vector, RNG)
- `bbx_dsp`: Block-based DSP graph system with oscillators, effects, and modulators
- `bbx_midi`: MIDI message parsing and streaming
- `bbx_file`: WAV file I/O integration
- `bbx_plugin`: FFI bindings for C/C++ plugin integration
