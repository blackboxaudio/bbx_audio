/**
 * BbxClient - WebSocket client for bbx_net audio control.
 */

import { Connection, type ConnectionState } from './connection.ts'
import { BbxError } from './errors.ts'
import type { ServerMessage, IWelcomeMessage, IStateMessage, IUpdateMessage, IErrorMessage } from './types.ts'

/* eslint-disable @typescript-eslint/no-unused-vars */
const clientEventNames = [
    'connected',
    'disconnected',
    'reconnecting',
    'state',
    'update',
    'error',
    'roomClosed',
    'latency',
] as const

/** Event names that can be subscribed to via {@link BbxClient.on}. */
export type ClientEventName = (typeof clientEventNames)[number]

/**
 * Event handler signatures for each client event.
 * Use with {@link BbxClient.on}, {@link BbxClient.off}, and {@link BbxClient.once}.
 */
export interface IBbxClientEvents {
    /** Fired after successfully connecting and joining a room. */
    connected: (welcome: IWelcomeMessage) => void
    /** Fired when disconnected from the server. */
    disconnected: (reason?: string) => void
    /** Fired before each reconnection attempt. */
    reconnecting: (attempt: number, maxAttempts: number, delayMs: number) => void
    /** Fired when full parameter state is received (after join or sync request). */
    state: (state: IStateMessage) => void
    /** Fired when a parameter is updated by another client or the server. */
    update: (update: IUpdateMessage) => void
    /** Fired when the server sends an error. */
    error: (error: IErrorMessage) => void
    /** Fired when the room is closed by the host. */
    roomClosed: () => void
    /** Fired when latency measurement is updated. */
    latency: (latencyMs: number) => void
}

/**
 * Configuration options for {@link BbxClient}.
 */
export interface IBbxClientConfig {
    /** WebSocket server URL (e.g., `ws://localhost:8080`). */
    url: string
    /** Room code to join. */
    roomCode: string
    /** Optional display name for this client. */
    clientName?: string
    /** Auto-reconnect on disconnect. Default: `true`. */
    reconnect?: boolean
    /** Base delay between reconnect attempts in ms. Default: `1000`. */
    reconnectDelay?: number
    /** Max reconnection attempts before giving up. Default: `5`. */
    maxReconnectAttempts?: number
    /** Interval between ping messages in ms. Default: `5000`. */
    pingInterval?: number
    /** Connection timeout in ms. Default: `10000`. */
    connectionTimeout?: number
}

type EventHandler<K extends ClientEventName> = IBbxClientEvents[K]

/**
 * WebSocket client for controlling bbx_net audio servers.
 *
 * Provides real-time parameter control, event triggers, and state synchronization
 * with automatic reconnection and clock synchronization for scheduled changes.
 *
 * @example
 * ```ts
 * const client = new BbxClient({
 *     url: 'ws://localhost:8080',
 *     roomCode: '123456',
 * })
 *
 * client.on('update', (msg) => console.log(msg.param, msg.value))
 * await client.connect()
 *
 * client.setParam('gain', 0.75)
 * ```
 */
export class BbxClient {
    private connection: Connection
    private handlers: Map<ClientEventName, Set<EventHandler<ClientEventName>>> = new Map()
    private _disconnectReason: string | undefined

    /**
     * Create a new client instance.
     * @param config - Client configuration
     * @throws {Error} If `url` or `roomCode` is missing
     */
    constructor(config: IBbxClientConfig) {
        if (!config.url) {
            throw new Error('url is required')
        }
        if (!config.roomCode) {
            throw new Error('roomCode is required')
        }

        const fullConfig = {
            url: config.url,
            roomCode: config.roomCode,
            clientName: config.clientName,
            reconnect: config.reconnect ?? true,
            reconnectDelay: config.reconnectDelay ?? 1000,
            maxReconnectAttempts: config.maxReconnectAttempts ?? 5,
            pingInterval: config.pingInterval ?? 5000,
            connectionTimeout: config.connectionTimeout ?? 10000,
        }

        this.connection = new Connection(fullConfig, {
            onMessage: (message) => this.handleMessage(message),
            onStateChange: (state) => this.handleStateChange(state),
            onLatency: (latencyMs) => this.emit('latency', latencyMs),
            onReconnecting: (attempt, max, delay) => this.emit('reconnecting', attempt, max, delay),
        })
    }

    /** Current connection state: `disconnected`, `connecting`, `connected`, or `reconnecting`. */
    get state(): ConnectionState {
        return this.connection.state
    }

    /** Unique node ID assigned by the server, or `null` if not connected. */
    get nodeId(): string | null {
        return this.connection.nodeId
    }

    /** Last measured round-trip latency in milliseconds. */
    get latency(): number {
        return this.connection.latency
    }

    /** Estimated offset between local and server clocks in milliseconds. */
    get clockOffset(): number {
        return this.connection.clockOffset
    }

    /** `true` if currently connected to the server. */
    get isConnected(): boolean {
        return this.connection.state === 'connected'
    }

    /** `true` if currently attempting to reconnect. */
    get isReconnecting(): boolean {
        return this.connection.state === 'reconnecting'
    }

    /**
     * Connect to the server and join the configured room.
     * Emits `connected` event on success.
     * @throws {BbxError} If connection fails or times out
     */
    async connect(): Promise<void> {
        const welcome = await this.connection.connect()
        this.emit('connected', welcome)
    }

    /** Disconnect from the server. Emits `disconnected` event. */
    disconnect(): void {
        this._disconnectReason = 'client requested disconnect'
        this.connection.disconnect()
    }

    /**
     * Send a parameter change to the server.
     * @param param - Parameter name
     * @param value - New value
     * @param at - Optional server timestamp (µs) for scheduled execution. Use {@link toServerTime} to convert.
     */
    setParam(param: string, value: number, at?: number): void {
        this.connection.send({
            type: 'param',
            param,
            value,
            at,
        })
    }

    /**
     * Send a trigger event to the server.
     * @param name - Trigger name
     * @param at - Optional server timestamp (µs) for scheduled execution
     */
    trigger(name: string, at?: number): void {
        this.connection.send({
            type: 'trigger',
            name,
            at,
        })
    }

    /** Request the current parameter state. Server responds with `state` event. */
    requestSync(): void {
        this.connection.send({
            type: 'sync',
        })
    }

    /**
     * Subscribe to an event.
     * @param event - Event name
     * @param handler - Callback function
     */
    on<K extends ClientEventName>(event: K, handler: IBbxClientEvents[K]): void {
        if (!this.handlers.has(event)) {
            this.handlers.set(event, new Set())
        }
        this.handlers.get(event)!.add(handler as EventHandler<ClientEventName>)
    }

    /**
     * Unsubscribe from an event.
     * @param event - Event name
     * @param handler - The same callback passed to {@link on}
     */
    off<K extends ClientEventName>(event: K, handler: IBbxClientEvents[K]): void {
        const handlers = this.handlers.get(event)
        if (handlers) {
            handlers.delete(handler as EventHandler<ClientEventName>)
        }
    }

    /**
     * Subscribe to an event for a single occurrence. Automatically unsubscribes after firing.
     * @param event - Event name
     * @param handler - Callback function
     */
    once<K extends ClientEventName>(event: K, handler: IBbxClientEvents[K]): void {
        const wrapper = ((...args: unknown[]) => {
            this.off(event, wrapper as IBbxClientEvents[K])
            ;(handler as (...a: unknown[]) => void)(...args)
        }) as IBbxClientEvents[K]
        this.on(event, wrapper)
    }

    /**
     * Remove all listeners for a specific event, or all events if none specified.
     * @param event - Optional event name. If omitted, removes all listeners.
     */
    removeAllListeners(event?: ClientEventName): void {
        if (event) {
            this.handlers.delete(event)
        } else {
            this.handlers.clear()
        }
    }

    /**
     * Manually reconnect after a disconnect. Only valid when in `disconnected` state.
     * @throws {BbxError} If not currently disconnected
     */
    reconnect(): Promise<void> {
        if (this.state !== 'disconnected') {
            throw new BbxError('CONNECTION_FAILED', 'Cannot reconnect: not disconnected')
        }
        return this.connect()
    }

    /**
     * Convert a local timestamp to server time for scheduling.
     * @param localTimeMs - Local time in milliseconds (e.g., `Date.now()`)
     * @returns Server timestamp in microseconds, suitable for the `at` parameter
     */
    toServerTime(localTimeMs: number): number {
        return localTimeMs * 1000 + this.clockOffset * 1000
    }

    /**
     * Convert a server timestamp to local time.
     * @param serverTimeUs - Server time in microseconds
     * @returns Local time in milliseconds
     */
    toLocalTime(serverTimeUs: number): number {
        return (serverTimeUs - this.clockOffset * 1000) / 1000
    }

    private handleMessage(message: ServerMessage): void {
        switch (message.type) {
            case 'state':
                this.emit('state', message)
                break
            case 'update':
                this.emit('update', message)
                break
            case 'error':
                this.emit('error', message)
                break
            case 'closed':
                this._disconnectReason = 'room closed'
                this.emit('roomClosed')
                break
        }
    }

    private handleStateChange(state: ConnectionState): void {
        if (state === 'disconnected') {
            this.emit('disconnected', this._disconnectReason)
            this._disconnectReason = undefined
        }
    }

    private emit<K extends ClientEventName>(event: K, ...args: Parameters<IBbxClientEvents[K]>): void {
        const handlers = this.handlers.get(event)
        if (handlers) {
            for (const handler of handlers) {
                ;(handler as (...args: Parameters<IBbxClientEvents[K]>) => void)(...args)
            }
        }
    }
}
