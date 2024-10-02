# `bbx_audio`

[![Version: v0.1.0](https://img.shields.io/badge/Version-v0.1.0-blue.svg)](https://github.com/blackboxdsp/bbx_audio)
[![Test](https://github.com/blackboxdsp/bbx_audio/actions/workflows/ci-test.yml/badge.svg)](https://github.com/blackboxdsp/bbx_audio/actions/workflows/ci-test.yml)

> A collection of Rust crates for audio-related DSP operations 🧮

## Setup

If you would like to "play in the sandbox", you will need to install some additional dependencies.

### Linux

Install the following packages:
```bash
sudo apt install alsa libasound2-dev libssl-dev pkg-config
```

## Running

To run an example in the sandbox, use the following command:

```bash
# From the bbx_sandbox/ directory
cargo run --release --example <example_name>
```
