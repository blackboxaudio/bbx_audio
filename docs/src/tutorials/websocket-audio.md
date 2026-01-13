# WebSocket Audio Control

This tutorial covers using `bbx_net` to control audio parameters over WebSocket, enabling browser-based and phone PWA interfaces.

## Prerequisites

> **Prior knowledge**: Familiarity with:
> - [Your First DSP Graph](first-graph.md) - Graph basics
> - [Parameter Modulation](modulation.md) - Understanding parameters

## When to Use WebSocket

WebSocket is ideal for:

- **Browser interfaces** - Build custom web UIs with HTML/CSS/JavaScript
- **Phone PWAs** - Progressive web apps that work offline
- **Multi-user control** - Room-based architecture for multiple connected clients
- **Custom dashboards** - React, Vue, or vanilla JS control surfaces

If you need to connect existing apps like TouchOSC, Max/MSP, or hardware controllers, see [OSC Audio Control](osc-audio.md) instead.

## Project Configuration

Add WebSocket support to your `Cargo.toml`:

```toml
[dependencies]
bbx_net = { version = "0.4", features = ["websocket"] }
bbx_dsp = "0.4"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Core Concepts

### Producer/Consumer Pattern

Network messages flow through a lock-free ring buffer (same as OSC):

```
Network Thread  ─────►  NetBufferProducer  ─────►  NetBufferConsumer  ─────►  Audio Thread
   (WebSocket)           (sends messages)           (receives messages)        (applies changes)
```

The audio thread never allocates or blocks—it reads messages using `drain_into_stack()`.

### Parameter Hashing

Parameters are identified by 32-bit FNV-1a hashes:

```rust
use bbx_net::hash_param_name;

let freq_hash = hash_param_name("freq");
let gain_hash = hash_param_name("gain");
```

### Room-Based Architecture

WebSocket uses room codes to isolate clients:

- Each room has a 6-digit numeric code (e.g., `123456`)
- Clients must join a room before sending parameter changes
- Multiple rooms can run simultaneously with different connected clients
- Rooms expire after 24 hours of inactivity (configurable)

## Setting Up the WebSocket Server

WebSocket requires a tokio async runtime:

```rust
use bbx_net::{
    net_buffer,
    websocket::{WsServer, WsServerConfig, ServerCommand},
};
use tokio::sync::mpsc;
use std::thread;

fn main() {
    let (producer, consumer) = net_buffer(256);
    let (command_tx, command_rx) = mpsc::channel::<ServerCommand>(256);

    let config = WsServerConfig {
        bind_addr: "0.0.0.0:8080".parse().unwrap(),
        ..Default::default()
    };

    let ws_server = WsServer::new(config, producer, command_rx);

    // Spawn the async server in a separate thread
    let command_tx_clone = command_tx.clone();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create a room for clients to join
            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
            command_tx_clone
                .send(ServerCommand::CreateRoom { response: response_tx })
                .await
                .unwrap();

            let room_code = response_rx.await.unwrap();
            println!("Room code: {room_code}");

            // Run the server
            ws_server.run().await.unwrap();
        });
    });

    // Your main thread handles audio processing with consumer
}
```

## JSON Protocol Reference

### Client → Server Messages

| Type | Fields | Description |
|------|--------|-------------|
| `join` | `room_code`, `client_name`? | Join a room |
| `param` | `param`, `value`, `at`? | Change parameter value |
| `trigger` | `name`, `at`? | Fire momentary event |
| `sync` | — | Request current state |
| `ping` | `client_time` | Latency measurement |
| `leave` | — | Exit room |

### Server → Client Messages

| Type | Fields | Description |
|------|--------|-------------|
| `welcome` | `node_id`, `server_time` | Join confirmation |
| `state` | `params[]` | Current parameter state |
| `update` | `param`, `value` | Parameter changed |
| `pong` | `client_time`, `server_time` | Latency response |
| `error` | `code`, `message` | Error notification |
| `closed` | — | Room shutdown |

### JSON Examples

**Join a room:**
```json
{"type": "join", "room_code": "123456", "client_name": "Phone 1"}
```

**Change a parameter:**
```json
{"type": "param", "param": "freq", "value": 0.5}
```

**With scheduled time (microseconds):**
```json
{"type": "param", "param": "freq", "value": 0.5, "at": 1000000}
```

**Fire a trigger:**
```json
{"type": "trigger", "name": "note_on"}
```

**Request state sync:**
```json
{"type": "sync"}
```

**Measure latency:**
```json
{"type": "ping", "client_time": 1705123456789}
```

**Server welcome response:**
```json
{"type": "welcome", "node_id": "abc-123-def", "server_time": 1705123456790}
```

**State sync response:**
```json
{"type": "state", "params": [
  {"name": "freq", "value": 0.5, "min": 0.0, "max": 1.0},
  {"name": "gain", "value": 0.8, "min": 0.0, "max": 1.0}
]}
```

## Building a Web Client

### Minimal JavaScript Client

```javascript
const ws = new WebSocket('ws://192.168.1.100:8080');

ws.onopen = () => {
    // Join the room (get code from server console output)
    ws.send(JSON.stringify({ type: 'join', room_code: '123456' }));
};

ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);

    switch (msg.type) {
        case 'welcome':
            console.log('Connected as', msg.node_id);
            break;
        case 'state':
            console.log('Current state:', msg.params);
            break;
        case 'update':
            console.log('Parameter changed:', msg.param, '=', msg.value);
            break;
        case 'error':
            console.error('Error:', msg.code, msg.message);
            break;
    }
};

// Send parameter change
function setParameter(name, value) {
    ws.send(JSON.stringify({ type: 'param', param: name, value }));
}

// Example: control frequency with a slider
document.getElementById('freq-slider').addEventListener('input', (e) => {
    setParameter('freq', parseFloat(e.target.value));
});
```

### TypeScript Interfaces

```typescript
interface ClientMessage {
    type: 'join' | 'param' | 'trigger' | 'sync' | 'ping' | 'leave';
    room_code?: string;
    client_name?: string;
    param?: string;
    name?: string;
    value?: number;
    at?: number;
    client_time?: number;
}

interface ServerMessage {
    type: 'welcome' | 'state' | 'update' | 'pong' | 'error' | 'closed';
    node_id?: string;
    server_time?: number;
    params?: ParamState[];
    param?: string;
    value?: number;
    client_time?: number;
    code?: string;
    message?: string;
}

interface ParamState {
    name: string;
    value: number;
    min: number;
    max: number;
}
```

## Clock Synchronization

For time-sensitive applications (e.g., synchronized triggers), use the ping/pong mechanism:

```javascript
let clockOffset = 0;

function measureLatency() {
    const clientTime = Date.now() * 1000; // microseconds
    ws.send(JSON.stringify({ type: 'ping', client_time: clientTime }));
}

ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.type === 'pong') {
        const now = Date.now() * 1000;
        const roundTrip = now - msg.client_time;
        const latency = roundTrip / 2;
        clockOffset = msg.server_time - (msg.client_time + latency);
        console.log('Latency:', latency / 1000, 'ms');
    }
};

// Schedule a parameter change 100ms in the future
function scheduleParam(name, value, delayMs) {
    const serverTime = Date.now() * 1000 + clockOffset + (delayMs * 1000);
    ws.send(JSON.stringify({
        type: 'param',
        param: name,
        value,
        at: serverTime
    }));
}
```

## Room Management

Control rooms via `ServerCommand`:

```rust
use bbx_net::websocket::ServerCommand;

// Create a new room
let (tx, rx) = tokio::sync::oneshot::channel();
command_tx.send(ServerCommand::CreateRoom { response: tx }).await?;
let room_code = rx.await?;

// Close a room (disconnects all clients)
command_tx.send(ServerCommand::CloseRoom { room_code: room_code.clone() }).await?;

// Shutdown the server
command_tx.send(ServerCommand::Shutdown).await?;
```

## Complete Example

Here's the WebSocket synthesizer from `bbx_sandbox/examples/15_ws_synth.rs`:

```rust
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
};

use bbx_dsp::{
    blocks::{GainBlock, LowPassFilterBlock, OscillatorBlock, PannerBlock},
    buffer::{AudioBuffer, Buffer},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::GraphBuilder,
    waveform::Waveform,
};
use bbx_net::{
    NetBufferConsumer, NetMessageType, hash_param_name, net_buffer,
    websocket::{ServerCommand, WsServer, WsServerConfig},
};
use rodio::{OutputStream, Source};
use tokio::sync::mpsc;

fn main() {
    println!("BBX WebSocket Synthesizer");
    println!("=========================\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Create network buffer
    let (producer, consumer) = net_buffer(256);
    let (command_tx, command_rx) = mpsc::channel::<ServerCommand>(256);

    // Configure WebSocket server
    let config = WsServerConfig {
        bind_addr: "0.0.0.0:8080".parse().unwrap(),
        ..Default::default()
    };

    let ws_server = WsServer::new(config, producer, command_rx);
    let command_tx_clone = command_tx.clone();
    let server_running = running.clone();

    // Spawn WebSocket server in background thread
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create a room
            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
            let _ = command_tx_clone
                .send(ServerCommand::CreateRoom { response: response_tx })
                .await;

            if let Ok(room_code) = response_rx.await {
                println!("Room code: {room_code}");
                println!("\nTo connect, use:");
                println!("  websocat ws://localhost:8080");
                println!("\nThen send:");
                println!("  {{\"type\": \"join\", \"room_code\": \"{room_code}\"}}");
            }

            let _ = ws_server.run().await;
        });
        server_running.store(false, Ordering::SeqCst);
    });

    // Wait for server to start
    thread::sleep(Duration::from_millis(100));

    println!("\nWebSocket server listening on port 8080");
    println!("\nParameter Mappings:");
    println!("  freq   - Oscillator frequency (0.0-1.0)");
    println!("  cutoff - Filter cutoff (0.0-1.0)");
    println!("  gain   - Output gain (0.0-1.0)");
    println!("  pan    - Stereo pan position (0.0-1.0)");
    println!("\nPress Ctrl+C to exit.\n");

    // Build DSP graph and start audio...
    // (Same pattern as OSC example)

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    let _ = command_tx.blocking_send(ServerCommand::Shutdown);
    println!("\nShutting down...");
}
```

## Running the Example

```bash
cargo run --release --example 15_ws_synth -p bbx_sandbox
```

## Testing with websocat

Install websocat for command-line WebSocket testing:

```bash
# macOS
brew install websocat

# Linux
cargo install websocat
```

Connect and test:

```bash
websocat ws://localhost:8080
```

Then send JSON messages:

```json
{"type": "join", "room_code": "123456"}
{"type": "param", "param": "freq", "value": 0.5}
{"type": "param", "param": "cutoff", "value": 0.7}
{"type": "sync"}
```

## Error Codes

| Code | Description |
|------|-------------|
| `INVALID_ROOM` | Room code doesn't exist |
| `ROOM_FULL` | Room is at capacity (default: 100 clients) |
| `NOT_IN_ROOM` | Tried to send parameter without joining |

## Next Steps

- [OSC Audio Control](osc-audio.md) — TouchOSC and external apps
- [WebSocket Server Reference](../crates/net/websocket.md) — Full API documentation
- [Network Architecture](../architecture/networking.md) — Lock-free design details
- [Real-Time Safety](../architecture/realtime-safety.md) — Understanding realtime constraints
