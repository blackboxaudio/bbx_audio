# Installation

## Adding bbx_audio to Your Project

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
bbx_dsp = "0.4.2"
bbx_core = "0.4.2"
```

For JUCE plugin integration, you'll only need:

```toml
[dependencies]
bbx_plugin = "0.4.2"
```

For audio file I/O:

```toml
[dependencies]
bbx_file = "0.4.2"
```

For MIDI support:

```toml
[dependencies]
bbx_midi = "0.4.2"
```

## Using Git Dependencies

To use the latest development version:

```toml
[dependencies]
bbx_dsp = { git = "https://github.com/blackboxaudio/bbx_audio" }
bbx_plugin = { git = "https://github.com/blackboxaudio/bbx_audio" }
```

## Platform-Specific Setup

### Linux

Install required packages for audio I/O:

```bash
sudo apt install alsa libasound2-dev libssl-dev pkg-config
```

### macOS

No additional dependencies required. CoreAudio is used automatically.

### Windows

No additional dependencies required. WASAPI is used automatically.

## Local Development

To develop against a local copy of bbx_audio, create `.cargo/config.toml` in your project:

```toml
[patch."https://github.com/blackboxaudio/bbx_audio"]
bbx_core = { path = "/path/to/bbx_audio/bbx_core" }
bbx_dsp = { path = "/path/to/bbx_audio/bbx_dsp" }
bbx_plugin = { path = "/path/to/bbx_audio/bbx_plugin" }
bbx_midi = { path = "/path/to/bbx_audio/bbx_midi" }
```

This file should be added to your `.gitignore`.

## Verifying Installation

Create a simple test to verify everything is working:

```rust
use bbx_dsp::GraphBuilder;

fn main() {
    let _graph = GraphBuilder::new();
    println!("bbx_audio installed successfully!");
}
```

Run with:

```bash
cargo run
```
