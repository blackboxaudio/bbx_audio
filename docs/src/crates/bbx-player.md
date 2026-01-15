# bbx_player

Audio playback for DSP graphs with pluggable backends.

## Overview

bbx_player provides:

- Simple API: Create a player, call `play()`, get a handle to stop playback
- Non-blocking playback in a background thread
- Multiple backends via feature flags (rodio or cpal)
- Generic `f32`/`f64` sample support, converts to `f32` at output

## Installation

```toml
[dependencies]
bbx_player = "0.4"              # Default rodio backend
bbx_player = { version = "0.4", features = ["cpal"], default-features = false }  # cpal only
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `rodio` | Yes | High-level backend using rodio |
| `cpal` | No | Low-level backend using cpal directly |

## Quick Example

```rust
use bbx_player::Player;
use std::time::Duration;

// Build your DSP graph...
let graph = GraphBuilder::<f32>::new(44100.0, 512, 2)
    .add(OscillatorBlock::new(440.0, Waveform::Sine, None))
    .build();

// Create player with default backend
let player = Player::new(graph)?;

// Start non-blocking playback
let handle = player.play()?;

// Do other work while audio plays...
std::thread::sleep(Duration::from_secs(5));

// Stop playback
handle.stop();
```

## Architecture

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│                  │     │                  │     │                  │
│   Graph<S>       │────▶│   Player<S>      │────▶│   Backend        │
│   (DSP)          │     │   (Coordinator)  │     │   (Output)       │
│                  │     │                  │     │                  │
└──────────────────┘     └────────┬─────────┘     └────────┬─────────┘
                                  │                        │
                                  ▼                        ▼
                         ┌──────────────────┐     ┌──────────────────┐
                         │                  │     │                  │
                         │   Signal         │────▶│   Audio Device   │
                         │   (Iterator)     │     │   (System)       │
                         │                  │     │                  │
                         └──────────────────┘     └──────────────────┘
```

1. **Graph**: Your DSP processing graph (oscillators, filters, effects)
2. **Player**: Wraps the graph and manages the backend
3. **Signal**: Converts graph output to an interleaved sample iterator
4. **Backend**: Sends samples to the system audio device

## API Reference

| Component | Description |
|-----------|-------------|
| [Player API](player/player.md) | `Player` struct and `PlayHandle` |
| [Backend Trait](player/backend.md) | `Backend` trait and implementations |

## Backend Comparison

| Backend | Pros | Cons |
|---------|------|------|
| `RodioBackend` | Simple, automatic device handling | Less control over device selection |
| `CpalBackend` | Direct device access, more control | More setup required |

## Examples

See `bbx_sandbox` for complete examples that use `bbx_player` for audio output.
