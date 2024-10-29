# `bbx_audio`

[![Test](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci-test.yml/badge.svg)](https://github.com/blackboxaudio/bbx_audio/actions/workflows/ci-test.yml)
[![Version: v0.1.0](https://img.shields.io/badge/Version-v0.1.0-blue.svg)](https://github.com/blackboxaudio/bbx_audio)
[![License](https://img.shields.io/badge/License-MIT-yellow)](https://github.com/blackboxaudio/bbx_audio/blob/develop/LICENSE)

> A collection of Rust crates for audio-related DSP operations 🧮

## Overview

`bbx_audio` is a collection of crates focused around audio-related DSP operations. The following crates are included within this repository:

- [`bbx_buffer`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_buffer) - Buffer trait, types, and operations
- [`bbx_draw`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_draw) - Visualization tooling
- [`bbx_dsp`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_dsp) - Graphs, nodes, and DSP logic
- [`bbx_file`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_file) - Reading / writing audio files
- [`bbx_midi`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_midi) - Streaming MIDI events
- [`bbx_sample`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_sample) - Numerical data traits, types, and operations
- [`bbx_sandbox`](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_sandbox) - Playground for all crates

## Getting Started

Setup is quite minimal except for a few required installations on Linux-based platforms.

:information_source: If you would like to use the `bbx_draw` crate for visualizations, follow the [instruction guide](https://guide.nannou.cc/getting_started/platform-specific_setup) to setup your environment for [Nannou](https://nannou.cc/).
Otherwise, continue on to the following steps.

### Linux

Install the following packages:
```bash
sudo apt install alsa libasound2-dev libssl-dev pkg-config
```

## Using the Sandbox

To run an example in the sandbox, use the following command:

```bash
# From the bbx_sandbox/ directory
cargo run --release --example <example_name>
```
