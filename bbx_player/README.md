# bbx_player

Audio playback abstractions for bbx_audio.

## Features

- **Simple API**: Create a player, call `play()`, get a handle to stop playback
- **Non-blocking**: Playback runs in a background thread
- **Multiple backends**: Choose between high-level (rodio) or low-level (cpal) backends
- **f32/f64 support**: Works with both sample types, converts to f32 at output

## Usage

### Default Backend (rodio)

```rust
use bbx_player::Player;
use std::time::Duration;

let player = Player::new(graph)?;
let handle = player.play()?;

std::thread::sleep(Duration::from_secs(5));
handle.stop();
```

### Custom Backend (cpal)

```rust
use bbx_player::{Player, backends::CpalBackend};

let backend = CpalBackend::try_default()?;
let player = Player::with_backend(graph, backend);
let handle = player.play()?;
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `rodio` | Yes | High-level backend using rodio |
| `cpal` | No | Low-level backend using cpal directly |

## Backend Comparison

| Backend | Pros | Cons |
|---------|------|------|
| `RodioBackend` | Simple, automatic device handling | Less control |
| `CpalBackend` | Direct device access, more control | More setup required |

## API Reference

### `Player<S>`

- `Player::new(graph)` - Create player with default rodio backend
- `Player::with_backend(graph, backend)` - Create player with custom backend
- `player.play()` - Start playback, returns `PlayHandle`

### `PlayHandle`

- `handle.stop()` - Stop playback
- `handle.is_stopped()` - Check if playback has stopped
