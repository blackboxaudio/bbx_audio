# `bbx_audio`

[![Test](https://github.com/blackboxdsp/bbx_audio/actions/workflows/ci-test.yml/badge.svg)](https://github.com/blackboxdsp/bbx_audio/actions/workflows/ci-test.yml)
[![Version: v0.1.0](https://img.shields.io/badge/Version-v0.1.0-blue.svg)](https://github.com/blackboxdsp/bbx_audio)
[![License](https://img.shields.io/badge/License-MIT-yellow)](https://github.com/blackboxdsp/bbx_audio/blob/develop/LICENSE)

> A collection of Rust crates for audio-related DSP operations 🧮

## Setup

Setup is quite minimal except for a few required installations on Linux-based platforms.

### Linux

Install the following packages:
```bash
sudo apt install alsa libasound2-dev libssl-dev pkg-config
```

:info: If you would like to use the `bbx_draw` crate for visualizations, follow the [instruction guide](https://guide.nannou.cc/getting_started/platform-specific_setup) to setup
[Nannou](https://nannou.cc/).

## Running

To run an example in the sandbox, use the following command:

```bash
# From the bbx_sandbox/ directory
cargo run --release --example <example_name>
```
