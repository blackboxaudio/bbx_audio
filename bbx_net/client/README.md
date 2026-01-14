# @bbx-audio/net

Lightweight WebSocket client library for communicating with `bbx_net` audio control servers.

## Installation

```bash
yarn add @bbx-audio/net
# or
npm install @bbx-audio/net
```

## Quick Start

```typescript
import { BbxClient } from '@bbx-audio/net'

const client = new BbxClient({
    url: 'ws://localhost:8080',
    roomCode: '123456',
    clientName: 'My Controller',
})

client.on('connected', (welcome) => {
    console.log(`Connected as ${welcome.node_id}`)
})

client.on('update', (update) => {
    console.log(`${update.param} = ${update.value}`)
})

await client.connect()

// Send parameter changes
client.setParam('gain', 0.75)
client.setParam('frequency', 0.5)

// Send trigger events
client.trigger('note_on')

// Request current state
client.requestSync()

// Disconnect when done
client.disconnect()
```

## API Reference

### BbxClient

The main client class for WebSocket communication.

#### Constructor

```typescript
new BbxClient(config: IBbxClientConfig)
```

#### Configuration

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `url` | `string` | required | WebSocket server URL |
| `roomCode` | `string` | required | Room code to join |
| `clientName` | `string` | `undefined` | Optional display name |
| `reconnect` | `boolean` | `true` | Auto-reconnect on disconnect |
| `reconnectDelay` | `number` | `1000` | Base delay between reconnects (ms) |
| `maxReconnectAttempts` | `number` | `5` | Max reconnection attempts |
| `pingInterval` | `number` | `5000` | Ping interval for latency measurement (ms) |

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `state` | `ConnectionState` | Current connection state |
| `nodeId` | `string \| null` | Assigned node ID (after connect) |
| `latency` | `number` | Last measured latency (ms) |
| `clockOffset` | `number` | Server clock offset (ms) |

#### Methods

| Method | Description |
|--------|-------------|
| `connect(): Promise<void>` | Connect to the server |
| `disconnect(): void` | Disconnect from the server |
| `setParam(param, value, at?): void` | Send parameter change |
| `trigger(name, at?): void` | Send trigger event |
| `requestSync(): void` | Request current parameter state |
| `on(event, handler): void` | Subscribe to events |
| `off(event, handler): void` | Unsubscribe from events |
| `toServerTime(localTimeMs): number` | Convert local time to server time (µs) |
| `toLocalTime(serverTimeUs): number` | Convert server time to local time (ms) |

#### Events

| Event | Handler Signature | Description |
|-------|-------------------|-------------|
| `connected` | `(welcome: IWelcomeMessage) => void` | Connection established |
| `disconnected` | `(reason?: string) => void` | Disconnected from server |
| `state` | `(state: IStateMessage) => void` | Received parameter state |
| `update` | `(update: IUpdateMessage) => void` | Parameter value updated |
| `error` | `(error: IErrorMessage) => void` | Error from server |
| `roomClosed` | `() => void` | Room was closed |
| `latency` | `(latencyMs: number) => void` | Latency measurement updated |

### Connection States

```typescript
type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting'
```

### Error Handling

```typescript
import { BbxClient, BbxError } from '@bbx-audio/net'

try {
    await client.connect()
} catch (error) {
    if (error instanceof BbxError) {
        switch (error.code) {
            case 'INVALID_ROOM':
                console.error('Room code not found')
                break
            case 'ROOM_FULL':
                console.error('Room is at capacity')
                break
            case 'CONNECTION_FAILED':
                console.error('Failed to connect')
                break
        }
    }
}
```

### Scheduled Parameter Changes

Parameters can be scheduled for future execution using server timestamps:

```typescript
// Schedule a parameter change 100ms in the future
const futureTime = Date.now() + 100
client.setParam('filter_cutoff', 0.5, client.toServerTime(futureTime))
```

### State Synchronization

```typescript
client.on('state', (state) => {
    for (const param of state.params) {
        console.log(`${param.name}: ${param.value} (range: ${param.min}-${param.max})`)
    }
})

client.requestSync()
```

## Message Types

All message types match the Rust protocol in `bbx_net`:

### Client → Server

- `IJoinMessage` - Join a room
- `IParamMessage` - Parameter value change
- `ITriggerMessage` - Trigger event
- `ISyncMessage` - Request state sync
- `IPingMessage` - Latency ping
- `ILeaveMessage` - Leave room

### Server → Client

- `IWelcomeMessage` - Join confirmation
- `IStateMessage` - Current parameter state
- `IUpdateMessage` - Parameter update
- `IPongMessage` - Ping response
- `IErrorMessage` - Error notification
- `IRoomClosedMessage` - Room closed

## License

MIT
