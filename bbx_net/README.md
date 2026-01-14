# bbx_net

Network audio control for art installations and live performance: OSC and WebSocket protocols.

This crate provides lock-free, realtime-safe network message passing for distributed audio systems. It supports:

- **OSC (UDP)** — For TouchOSC, Max/MSP, Pure Data, and other creative tools
- **WebSocket** — For phone PWA interfaces and web-based control

## Features

Enable protocols via feature flags:

```toml
[dependencies]
bbx_net = { version = "0.4", features = ["osc"] }        # OSC only
bbx_net = { version = "0.4", features = ["websocket"] }  # WebSocket only
bbx_net = { version = "0.4", features = ["full"] }       # Both protocols
```

## Quick Start

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
    match msg.param_hash {
        h if h == hash_param_name("gain") => {
            // Apply gain change
        }
        _ => {}
    }
}
```

## OSC Server

The OSC server listens for UDP packets and forwards them to the audio thread.

```rust
use bbx_net::{net_buffer, osc::{OscServer, OscServerConfig}};

let (producer, consumer) = net_buffer(256);

let config = OscServerConfig {
    bind_addr: "0.0.0.0:9000".parse().unwrap(),
    ..Default::default()
};

let server = OscServer::new(config, producer);
let handle = server.spawn(); // Runs in background thread
```

### OSC Address Format

Messages must follow one of these patterns:

| Address | Description |
|---------|-------------|
| `/blocks/param/gain` | Broadcast gain parameter to all blocks |
| `/blocks/param/freq` | Broadcast frequency parameter to all blocks |
| `/block/<uuid>/param/gain` | Target specific block by UUID |

### TouchOSC Setup

1. Set **Host** to your computer's IP address
2. Set **Port** to 9000
3. Create faders with addresses like `/blocks/param/freq`, `/blocks/param/cutoff`, `/blocks/param/gain`
4. Set fader range to 0.0–1.0

## WebSocket Server

The WebSocket server supports room-based authentication and bidirectional communication.

```rust
use bbx_net::{
    net_buffer,
    websocket::{WsServer, WsServerConfig, ServerCommand},
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (producer, consumer) = net_buffer(256);
    let (command_tx, command_rx) = mpsc::channel(256);

    let config = WsServerConfig {
        bind_addr: "0.0.0.0:8080".parse().unwrap(),
        ..Default::default()
    };

    let server = WsServer::new(config, producer, command_rx);

    // Create a room
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    command_tx.send(ServerCommand::CreateRoom { response: response_tx }).await.unwrap();
    let room_code = response_rx.await.unwrap();
    println!("Room code: {room_code}");

    // Run server
    server.run().await.unwrap();
}
```

## TypeScript Client

The [`@bbx-audio/net`](https://www.npmjs.com/package/@bbx-audio/net) package provides a TypeScript/JavaScript client for web and Node.js applications.

### Installation

```bash
npm install @bbx-audio/net
# or
yarn add @bbx-audio/net
```

### Usage

```typescript
import { BbxClient } from '@bbx-audio/net'

const client = new BbxClient({
    url: 'ws://localhost:8080',
    roomCode: '123456',
})

client.on('connected', () => console.log('Connected!'))
client.on('update', (msg) => console.log(`${msg.param} = ${msg.value}`))

await client.connect()
client.setParam('freq', 0.5)
```

See the [full documentation](./client/README.md) for the complete API reference.

### WebSocket JSON Protocol

**Client → Server:**

```json
// Join a room
{"type": "join", "room_code": "ABC123"}

// Send parameter change
{"type": "param", "param": "freq", "value": 0.5}

// Send trigger event
{"type": "trigger", "name": "note_on"}

// Request current state
{"type": "sync"}

// Ping for latency measurement
{"type": "ping", "client_time": 1234567890}

// Leave room
{"type": "leave"}
```

**Server → Client:**

```json
// Welcome after joining
{"type": "welcome", "node_id": "uuid-string", "server_time": 1234567890}

// Current parameter state
{"type": "state", "params": [{"name": "freq", "value": 0.5, "min": 0.0, "max": 1.0}]}

// Parameter update (broadcast)
{"type": "update", "param": "freq", "value": 0.7}

// Pong response
{"type": "pong", "client_time": 1234567890, "server_time": 1234567891}

// Error
{"type": "error", "code": "INVALID_ROOM", "message": "Invalid room code"}

// Room closed
{"type": "closed"}
```

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

### Parameter Hashing

Parameters are identified by FNV-1a hash for efficient matching:

```rust
use bbx_net::hash_param_name;

let freq_hash = hash_param_name("freq");
let gain_hash = hash_param_name("gain");

// In audio thread:
if msg.param_hash == freq_hash {
    // Handle frequency change
}
```

### Realtime Safety

The `NetBufferConsumer` provides realtime-safe methods:

- `drain_into_stack()` — Returns up to 64 messages using stack allocation (no heap)
- `try_pop()` — Pop a single message
- `is_empty()` / `len()` — Check buffer state

## Examples

See `bbx_sandbox` for complete examples:

- `14_osc_synth.rs` — TouchOSC-controlled synthesizer
- `15_ws_synth.rs` — WebSocket-controlled synthesizer

```bash
# Run OSC example
cargo run --example 14_osc_synth -p bbx_sandbox

# Run WebSocket example
cargo run --example 15_ws_synth -p bbx_sandbox
```

## Architecture

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
