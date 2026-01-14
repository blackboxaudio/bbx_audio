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

### Using the TypeScript Client

The [`@bbx-audio/net`](https://www.npmjs.com/package/@bbx-audio/net) package provides a high-level client with automatic reconnection, latency measurement, and type safety.

**Installation:**

```bash
npm install @bbx-audio/net
```

**Basic usage:**

```typescript
import { BbxClient } from '@bbx-audio/net'

const client = new BbxClient({
    url: 'ws://192.168.1.100:8080',
    roomCode: '123456',
    clientName: 'My Controller',
})

// Handle events
client.on('connected', (welcome) => {
    console.log('Connected as', welcome.node_id)
})

client.on('state', (state) => {
    console.log('Current state:', state.params)
})

client.on('update', (update) => {
    console.log('Parameter changed:', update.param, '=', update.value)
})

client.on('error', (error) => {
    console.error('Error:', error.code, error.message)
})

// Connect and control
await client.connect()

// Send parameter changes
client.setParam('freq', 0.5)
client.setParam('cutoff', 0.7)

// Send trigger events
client.trigger('note_on')

// Request current state
client.requestSync()
```

**HTML slider integration:**

```typescript
document.getElementById('freq-slider').addEventListener('input', (e) => {
    client.setParam('freq', parseFloat((e.target as HTMLInputElement).value))
})
```

## Clock Synchronization

The client automatically measures latency and calculates clock offset. Use the built-in conversion methods for scheduled parameter changes:

```typescript
// Schedule a parameter change 100ms in the future
const futureTime = Date.now() + 100
client.setParam('freq', 0.8, client.toServerTime(futureTime))

// Monitor latency
client.on('latency', (latencyMs) => {
    console.log('Latency:', latencyMs, 'ms')
})

// Access clock offset directly
console.log('Clock offset:', client.clockOffset, 'ms')
```

## Framework Examples

### React

```tsx
import { useEffect, useState, useCallback } from 'react'
import { BbxClient, type IParamState } from '@bbx-audio/net'

function AudioController({ url, roomCode }: { url: string; roomCode: string }) {
    const [client] = useState(() => new BbxClient({ url, roomCode, clientName: 'React Controller' }))
    const [connected, setConnected] = useState(false)
    const [params, setParams] = useState<IParamState[]>([])
    const [latency, setLatency] = useState<number | null>(null)
    const [error, setError] = useState<string | null>(null)

    useEffect(() => {
        client.on('connected', () => {
            setConnected(true)
            setError(null)
        })
        client.on('disconnected', () => setConnected(false))
        client.on('state', (state) => setParams(state.params))
        client.on('update', (update) => {
            setParams((prev) =>
                prev.map((p) => (p.name === update.param ? { ...p, value: update.value } : p))
            )
        })
        client.on('latency', setLatency)
        client.on('error', (err) => setError(`${err.code}: ${err.message}`))

        client.connect()
        return () => client.disconnect()
    }, [client])

    const handleChange = useCallback(
        (param: string, value: number) => client.setParam(param, value),
        [client]
    )

    if (!connected) return <div>Connecting...</div>

    return (
        <div>
            {error && <div className="error">{error}</div>}
            {latency !== null && <div className="latency">Latency: {latency.toFixed(0)}ms</div>}
            {params.map((param) => (
                <label key={param.name}>
                    {param.name}
                    <input
                        type="range"
                        min={param.min}
                        max={param.max}
                        step={0.01}
                        value={param.value}
                        onChange={(e) => handleChange(param.name, parseFloat(e.target.value))}
                    />
                </label>
            ))}
            <button onClick={() => client.trigger('note_on')}>Trigger Note</button>
        </div>
    )
}
```

### Svelte

```svelte
<script lang="ts">
    import { BbxClient, type IParamState } from '@bbx-audio/net'

    let { url, roomCode }: { url: string; roomCode: string } = $props()

    let client: BbxClient | null = $state(null)
    let connected = $state(false)
    let params = $state<IParamState[]>([])
    let latency = $state<number | null>(null)
    let error = $state<string | null>(null)

    $effect(() => {
        const bbxClient = new BbxClient({ url, roomCode, clientName: 'Svelte Controller' })
        client = bbxClient

        bbxClient.on('connected', () => {
            connected = true
            error = null
        })
        bbxClient.on('disconnected', () => (connected = false))
        bbxClient.on('state', (state) => (params = state.params))
        bbxClient.on('update', (update) => {
            params = params.map((p) =>
                p.name === update.param ? { ...p, value: update.value } : p
            )
        })
        bbxClient.on('latency', (ms) => (latency = ms))
        bbxClient.on('error', (err) => (error = `${err.code}: ${err.message}`))

        bbxClient.connect()

        return () => bbxClient.disconnect()
    })

    function handleChange(param: string, value: number) {
        client?.setParam(param, value)
    }

    function handleTrigger() {
        client?.trigger('note_on')
    }
</script>

{#if !connected}
    <div>Connecting...</div>
{:else}
    <div>
        {#if error}
            <div class="error">{error}</div>
        {/if}
        {#if latency !== null}
            <div class="latency">Latency: {latency.toFixed(0)}ms</div>
        {/if}
        {#each params as param}
            <label>
                {param.name}
                <input
                    type="range"
                    min={param.min}
                    max={param.max}
                    step={0.01}
                    value={param.value}
                    oninput={(e) => handleChange(param.name, parseFloat(e.currentTarget.value))}
                />
            </label>
        {/each}
        <button onclick={handleTrigger}>Trigger Note</button>
    </div>
{/if}
```

### Vue 3

```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { BbxClient, type IParamState } from '@bbx-audio/net'

const props = defineProps<{ url: string; roomCode: string }>()

const client = new BbxClient({
    url: props.url,
    roomCode: props.roomCode,
    clientName: 'Vue Controller',
})
const connected = ref(false)
const params = ref<IParamState[]>([])
const latency = ref<number | null>(null)
const error = ref<string | null>(null)

onMounted(() => {
    client.on('connected', () => {
        connected.value = true
        error.value = null
    })
    client.on('disconnected', () => (connected.value = false))
    client.on('state', (state) => (params.value = state.params))
    client.on('update', (update) => {
        const param = params.value.find((p) => p.name === update.param)
        if (param) param.value = update.value
    })
    client.on('latency', (ms) => (latency.value = ms))
    client.on('error', (err) => (error.value = `${err.code}: ${err.message}`))

    client.connect()
})

onUnmounted(() => client.disconnect())

function handleChange(param: string, value: number) {
    client.setParam(param, value)
}

function handleTrigger() {
    client.trigger('note_on')
}
</script>

<template>
    <div v-if="!connected">Connecting...</div>
    <div v-else>
        <div v-if="error" class="error">{{ error }}</div>
        <div v-if="latency !== null" class="latency">Latency: {{ latency.toFixed(0) }}ms</div>
        <label v-for="param in params" :key="param.name">
            {{ param.name }}
            <input
                type="range"
                :min="param.min"
                :max="param.max"
                step="0.01"
                :value="param.value"
                @input="handleChange(param.name, parseFloat(($event.target as HTMLInputElement).value))"
            />
        </label>
        <button @click="handleTrigger">Trigger Note</button>
    </div>
</template>
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
