# bbx_net

Network audio control for art installations and live performance: OSC and WebSocket protocols.

## Overview

bbx_net provides:

- Lock-free, realtime-safe network message passing
- OSC (UDP) support for TouchOSC, Max/MSP, Pure Data
- WebSocket support for phone PWAs and browser interfaces
- FNV-1a parameter hashing for efficient lookup
- Clock synchronization for sample-accurate timing

## Installation

Enable protocols via feature flags:

```toml
[dependencies]
bbx_net = { version = "0.4", features = ["osc"] }        # OSC only
bbx_net = { version = "0.4", features = ["websocket"] }  # WebSocket only
bbx_net = { version = "0.4", features = ["full"] }       # Both protocols
```

## Features

| Feature | Description |
|---------|-------------|
| [OSC Server](net/osc.md) | UDP-based OSC receiver |
| [WebSocket Server](net/websocket.md) | Room-based WebSocket server |

## Quick Example

Create a buffer pair for communication between network and audio threads:

```rust
use bbx_net::{net_buffer, NetMessage, NetMessageType, hash_param_name};

// Create buffer (network thread → audio thread)
let (mut producer, mut consumer) = net_buffer(256);

// Network thread: send parameter changes
let msg = NetMessage {
    message_type: NetMessageType::ParameterChange,
    param_hash: hash_param_name("gain"),
    value: 0.5,
    node_id: Default::default(),
    timestamp: Default::default(),
};
producer.try_send(msg);

// Audio thread: receive messages (realtime-safe, no allocation)
let events = consumer.drain_into_stack();
for msg in events {
    if msg.param_hash == hash_param_name("gain") {
        // Apply gain change
    }
}
```

## Architecture

The crate uses a producer/consumer pattern with a lock-free ring buffer:

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│  TouchOSC   │────▶│   OscServer      │     │             │
│  (Phone)    │ UDP │   (UDP recv)     │     │             │
└─────────────┘     └────────┬─────────┘     │             │
                             │               │   Audio     │
                             ▼               │   Thread    │
                    ┌──────────────────┐     │             │
                    │ NetBufferProducer│────▶│ NetBuffer   │
                    └──────────────────┘     │ Consumer    │
                             ▲               │             │
┌─────────────┐     ┌────────┴─────────┐     │             │
│  Browser    │────▶│   WsServer       │     │             │
│  (Phone)    │ WS  │   (Tokio async)  │     │             │
└─────────────┘     └──────────────────┘     └─────────────┘
```

Both OSC and WebSocket servers populate the same `NetBufferProducer`, allowing mixed control sources in a single application.

## Protocol Reference

### NetMessage

The core message type for network communication:

```rust
pub struct NetMessage {
    pub message_type: NetMessageType,  // ParameterChange, Trigger, etc.
    pub param_hash: u32,                // FNV-1a hash of parameter name
    pub value: f32,                     // Parameter value
    pub node_id: NodeId,                // Source node identifier
    pub timestamp: SyncedTimestamp,     // Synchronized timestamp
}
```

### Message Types

| Type | Description |
|------|-------------|
| ParameterChange | Float value change |
| Trigger | Momentary event |
| StateRequest | Query current state |
| StateResponse | Server state response |
| Ping / Pong | Latency measurement |

### Parameter Hashing

Parameters are identified by FNV-1a hash for efficient matching:

```rust
use bbx_net::hash_param_name;

let freq_hash = hash_param_name("freq");
let gain_hash = hash_param_name("gain");

// In audio thread (realtime-safe):
if msg.param_hash == freq_hash {
    // Handle frequency change
}
```

## Realtime Safety

The `NetBufferConsumer` provides realtime-safe methods:

- `drain_into_stack()` — Returns up to 64 messages using stack allocation
- `try_pop()` — Pop a single message
- `is_empty()` / `len()` — Check buffer state

## Examples

See `bbx_sandbox` for complete examples:

- `14_osc_synth.rs` — TouchOSC-controlled synthesizer
- `15_ws_synth.rs` — WebSocket-controlled synthesizer

```bash
cargo run --example 14_osc_synth -p bbx_sandbox
cargo run --example 15_ws_synth -p bbx_sandbox
```
