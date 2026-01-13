# Network Architecture

This document covers the architecture of `bbx_net` for network-based audio control.

## Overview

`bbx_net` provides lock-free, realtime-safe network message passing for distributed audio systems. The design separates protocol handling (which may block or allocate) from audio thread consumption (which must be allocation-free and lock-free).

**Design goals:**

- **Realtime safety** - No allocations, locks, or system calls in audio thread paths
- **Protocol agnostic** - Core message types work with any network protocol
- **Feature-gated** - Optional `osc` and `websocket` features minimize dependencies

## Lock-Free Message Passing

The core of `bbx_net` is the producer/consumer pattern using an SPSC (Single-Producer, Single-Consumer) ring buffer from `bbx_core`.

### Buffer Types

```rust
// Create a buffer pair
let (producer, consumer) = net_buffer(256);
```

- **`NetBufferProducer`** - Used by network threads to send messages
- **`NetBufferConsumer`** - Used by the audio thread to receive messages

### Realtime-Safe Consumption

The consumer provides methods that never allocate:

```rust
// Returns up to 64 messages in a stack-allocated array
let messages = consumer.drain_into_stack();

// Or pop one at a time
while let Some(msg) = consumer.try_pop() {
    // Handle message
}
```

`drain_into_stack()` uses `StackVec<NetMessage, 64>`, avoiding heap allocation entirely. The constant `MAX_NET_EVENTS_PER_BUFFER` (64) limits stack usage while providing enough capacity for typical use.

## Protocol-Agnostic Design

### NetMessage

All network protocols produce the same message type:

```rust
pub struct NetMessage {
    pub message_type: NetMessageType,  // ParameterChange, Trigger, Ping, etc.
    pub param_hash: u32,               // FNV-1a hash of parameter name
    pub value: f32,                    // Parameter value
    pub node_id: NodeId,               // Source node identifier
    pub timestamp: SyncedTimestamp,    // Synchronized timestamp
}
```

### FNV-1a Parameter Hashing

Parameters are identified by 32-bit hashes to avoid string comparisons in the audio thread:

```rust
pub fn hash_param_name(name: &str) -> u32 {
    const FNV_OFFSET: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    let mut hash = FNV_OFFSET;
    for byte in name.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
```

Pre-compute hashes at startup, then compare integers in the audio callback.

### NodeId

Each network node has a 128-bit UUID for identification:

```rust
pub struct NodeId {
    pub high: u64,
    pub low: u64,
}
```

Generated using `XorShiftRng` for deterministic distribution. Supports UUID string format (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).

## OSC Architecture

The OSC module provides UDP-based Open Sound Control support.

### Server Design

`OscServer` runs a blocking UDP receive loop on a dedicated thread:

```
┌─────────────┐      UDP      ┌──────────────┐
│  TouchOSC   │──────────────▶│  OscServer   │
│  (Phone)    │               │  (blocking)  │
└─────────────┘               └──────┬───────┘
                                     │
                                     ▼
                              ┌──────────────┐
                              │ NetBuffer    │
                              │ Producer     │
                              └──────────────┘
```

The blocking design:
- Keeps latency low (no async overhead)
- Runs on a single dedicated thread
- Uses `rosc` crate for packet parsing

### Address Parsing

OSC addresses are parsed into `AddressPath`:

| Pattern | Meaning |
|---------|---------|
| `/blocks/param/<name>` | Broadcast to all blocks |
| `/block/<uuid>/param/<name>` | Target specific block |

Invalid addresses are silently dropped.

## WebSocket Architecture

The WebSocket module provides room-based bidirectional communication.

### Async Runtime

`WsServer` uses tokio for async I/O:

```
┌─────────────┐   WebSocket   ┌──────────────┐
│  Browser    │──────────────▶│  WsServer    │
│  (Phone)    │               │  (Tokio)     │
└─────────────┘               └──────┬───────┘
                                     │
                                     ▼
                              ┌──────────────┐
                              │ NetBuffer    │
                              │ Producer     │
                              └──────────────┘
```

The async design:
- Handles many concurrent connections efficiently
- Uses tokio's multi-threaded runtime
- Per-connection tasks for message handling

### Room Management

`RoomManager` handles room lifecycle:

```rust
pub struct RoomManager {
    rooms: HashMap<String, Room>,
    config: RoomConfig,
    rng: XorShiftRng,
}
```

Rooms provide simple authentication via numeric codes (default 4 digits). Features include:
- Automatic expiration (default 24 hours)
- Per-room client limits
- Client tracking with `NodeId`

### Server Commands

The main thread controls the server via `ServerCommand`:

```rust
pub enum ServerCommand {
    BroadcastParam { name: String, value: f32 },
    CreateRoom { response: oneshot::Sender<String> },
    CloseRoom { code: String },
    Shutdown,
}
```

### Connection State

Each client connection tracks:
- `node_id` - Unique identifier
- `room_code` - Which room they joined
- `clock_offset` - NTP-style offset for synchronization
- `last_ping` - For connection health checks

## Clock Synchronization

For sample-accurate timing across the network, `bbx_net` provides clock synchronization.

### ClockSync

`ClockSync` manages timestamps with a realtime-safe cached value:

```rust
// Main thread: update clock periodically
clock_sync.tick();

// Audio thread: read cached value (no system call)
let now = clock_sync.cached_now();
```

The `tick()` method updates an atomic; `cached_now()` reads it without calling `Instant::elapsed()`.

### NTP-Style Offset Calculation

Clock offset between client and server is calculated using:

```rust
pub fn calculate_offset(t1: u64, t2: u64, t3: u64, t4: u64) -> i64 {
    // t1: client send time
    // t2: server receive time
    // t3: server send time
    // t4: client receive time
    ((t2 as i64 - t1 as i64) + (t3 as i64 - t4 as i64)) / 2
}
```

### SyncedTimestamp

Timestamps map to sample offsets within audio buffers:

```rust
impl SyncedTimestamp {
    pub fn to_sample_offset(&self, sample_rate: f64, buffer_size: usize) -> usize {
        // Convert microseconds to sample position
    }
}
```

This enables scheduling parameter changes at specific samples rather than just buffer boundaries.

## Thread Model

```
┌─────────────────────────────────────────────────────────────────┐
│                        Network Sources                          │
│  ┌──────────────────┐              ┌──────────────────────┐    │
│  │  TouchOSC        │              │   Web Browser        │    │
│  │  (UDP client)    │              │   (WebSocket client) │    │
│  └────────┬─────────┘              └──────────┬───────────┘    │
│           │                                   │                 │
└───────────┼───────────────────────────────────┼─────────────────┘
            │                                   │
      ┌─────▼──────────┐              ┌─────────▼─────────┐
      │  OscServer     │              │   WsServer        │
      │  Thread        │              │   (Tokio tasks)   │
      │  (blocking)    │              │   (async)         │
      └─────┬──────────┘              └─────────┬─────────┘
            │                                   │
            └─────────────┬─────────────────────┘
                          │
           ┌──────────────▼──────────────┐
           │  NetBufferProducer          │
           │  (lock-free SPSC)           │
           └──────────────┬──────────────┘
                          │
           ┌──────────────▼──────────────┐
           │  NetBufferConsumer          │
           │  (realtime-safe)            │
           │  - drain_into_stack()       │
           │  - try_pop()                │
           └──────────────┬──────────────┘
                          │
                ┌─────────▼──────────┐
                │   Audio Thread     │
                │                    │
                │ - Hash lookup      │
                │ - Block mutation   │
                │ - DSP processing   │
                └────────────────────┘
```

## Integration Points

`bbx_net` is protocol-agnostic and doesn't depend on `bbx_dsp`. Applications connect messages to DSP blocks:

```rust
// Pre-compute hashes
let freq_hash = hash_param_name("freq");

// In audio callback
let messages = consumer.drain_into_stack();
for msg in messages {
    if msg.param_hash == freq_hash {
        // Map to DSP block
        oscillator.set_frequency(msg.value * 1000.0);
    }
}
```

This separation allows flexible mapping strategies without coupling the network layer to DSP internals.
