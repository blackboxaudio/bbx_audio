# Player API

The `Player` struct coordinates DSP graph playback through a backend.

## Player\<S\>

```rust
pub struct Player<S: Sample> {
    graph: Graph<S>,
    backend: Box<dyn Backend<S>>,
}
```

### Constructors

#### `Player::new(graph)` (requires `rodio` feature)

Create a player with the default rodio backend:

```rust
use bbx_player::Player;

let player = Player::new(graph)?;
```

#### `Player::with_backend(graph, backend)`

Create a player with a custom backend:

```rust
use bbx_player::{Player, backends::CpalBackend};

let backend = CpalBackend::try_default()?;
let player = Player::with_backend(graph, backend);
```

### Methods

#### `play(self) -> Result<PlayHandle>`

Start non-blocking playback. The player is consumed and a `PlayHandle` is returned for controlling playback.

```rust
let handle = player.play()?;
```

## PlayHandle

Handle returned from `Player::play()` for controlling playback.

```rust
pub struct PlayHandle { /* ... */ }
```

### Methods

#### `stop(&self)`

Stop playback. This signals the background thread to terminate.

```rust
handle.stop();
```

#### `is_stopped(&self) -> bool`

Check if playback has been stopped.

```rust
if handle.is_stopped() {
    println!("Playback ended");
}
```

## Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  1. Create Player          2. Start Playback    3. Control      │
│  ┌──────────────────┐      ┌────────────────┐   ┌────────────┐  │
│  │ Player::new()    │─────▶│ player.play()  │──▶│ PlayHandle │  │
│  │ Player::with_*() │      │ (consumes)     │   │            │  │
│  └──────────────────┘      └────────────────┘   └─────┬──────┘  │
│                                                       │         │
│                                                       ▼         │
│                                              ┌────────────────┐ │
│                                              │ handle.stop()  │ │
│                                              │ (signals end)  │ │
│                                              └────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Error Handling

The `Result` type is `Result<T, PlayerError>`:

```rust
pub enum PlayerError {
    /// No audio output device is available on the system.
    NoOutputDevice,
    /// Failed to initialize the audio output device.
    DeviceInitFailed(String),
    /// An error occurred during audio playback.
    PlaybackFailed(String),
    /// An error from the underlying audio backend (rodio or cpal).
    BackendError(String),
}
```

## Thread Safety

- `Player` is `Send` but not `Sync`
- `PlayHandle` is both `Send` and `Sync`
- The stop flag uses `AtomicBool` with `SeqCst` ordering for reliable cross-thread communication
