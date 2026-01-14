/**
 * Internal WebSocket connection manager.
 * @internal
 */

import { BbxError } from './errors.ts'
import type { ClientMessage, ServerMessage, IWelcomeMessage } from './types.ts'

/* eslint-disable @typescript-eslint/no-unused-vars */
const connectionStates = ['disconnected', 'connecting', 'connected', 'reconnecting'] as const

/**
 * The current state of the WebSocket connection.
 *
 * - `disconnected` - No active connection
 * - `connecting` - Connection attempt in progress
 * - `connected` - Successfully connected and authenticated
 * - `reconnecting` - Attempting to reconnect after a disconnect
 */
export type ConnectionState = (typeof connectionStates)[number]

/**
 * Configuration options for the Connection manager.
 * @internal
 */
export interface IConnectionConfig {
    /** WebSocket server URL. */
    url: string
    /** Room code to join. */
    roomCode: string
    /** Optional display name for this client. */
    clientName?: string
    /** Whether to automatically reconnect on disconnect. */
    reconnect: boolean
    /** Base delay between reconnection attempts in milliseconds. */
    reconnectDelay: number
    /** Maximum number of reconnection attempts before giving up. */
    maxReconnectAttempts: number
    /** Interval between ping messages in milliseconds. */
    pingInterval: number
    /** Timeout for initial connection in milliseconds. */
    connectionTimeout: number
}

/**
 * Callbacks for connection events.
 * @internal
 */
export interface IConnectionCallbacks {
    /** Called when a message is received from the server. */
    onMessage: (message: ServerMessage) => void
    /** Called when the connection state changes. */
    onStateChange: (state: ConnectionState) => void
    /** Called when latency is measured. */
    onLatency: (latencyMs: number) => void
    /** Called before each reconnection attempt. */
    onReconnecting: (attempt: number, maxAttempts: number, delayMs: number) => void
}

/**
 * Low-level WebSocket connection manager.
 * Handles connection lifecycle, reconnection, and ping/pong for clock sync.
 *
 * Use {@link BbxClient} instead for most applications.
 * @internal
 */
export class Connection {
    private ws: WebSocket | null = null
    private reconnectAttempts = 0
    private reconnectTimeout: ReturnType<typeof setTimeout> | null = null
    private connectionTimeoutId: ReturnType<typeof setTimeout> | null = null
    private pingInterval: ReturnType<typeof setInterval> | null = null
    private lastPingTime = 0
    private _latency = 0
    private _clockOffset = 0
    private _state: ConnectionState = 'disconnected'
    private _nodeId: string | null = null
    private pendingConnect: {
        resolve: (welcome: IWelcomeMessage) => void
        reject: (error: BbxError) => void
    } | null = null

    constructor(
        private config: IConnectionConfig,
        private callbacks: IConnectionCallbacks
    ) {}

    /** Current connection state. */
    get state(): ConnectionState {
        return this._state
    }

    /** Node ID assigned by the server, or null if not connected. */
    get nodeId(): string | null {
        return this._nodeId
    }

    /** Last measured round-trip latency in milliseconds. */
    get latency(): number {
        return this._latency
    }

    /** Estimated offset between local and server clocks in milliseconds. */
    get clockOffset(): number {
        return this._clockOffset
    }

    /**
     * Establish a WebSocket connection and join the configured room.
     * @returns Promise resolving with the welcome message on success
     * @throws {BbxError} If already connected, connection fails, or times out
     */
    connect(): Promise<IWelcomeMessage> {
        return new Promise((resolve, reject) => {
            if (this._state === 'connected' || this._state === 'connecting') {
                reject(new BbxError('CONNECTION_FAILED', 'Already connected or connecting'))
                return
            }

            this.pendingConnect = { resolve, reject }
            this.setState('connecting')

            this.connectionTimeoutId = setTimeout(() => {
                if (this.pendingConnect) {
                    this.pendingConnect.reject(new BbxError('TIMEOUT', 'Connection timed out'))
                    this.pendingConnect = null
                    this.cleanup()
                    this.setState('disconnected')
                }
            }, this.config.connectionTimeout)

            this.createWebSocket()
        })
    }

    /** Close the connection and clean up resources. */
    disconnect(): void {
        this.cleanup()
        this.setState('disconnected')
    }

    /**
     * Send a message to the server. Silently ignored if not connected.
     * @param message - The message to send
     */
    send(message: ClientMessage): void {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message))
        }
    }

    private createWebSocket(): void {
        this.ws = new WebSocket(this.config.url)

        this.ws.onopen = () => {
            this.send({
                type: 'join',
                room_code: this.config.roomCode,
                client_name: this.config.clientName,
            })
        }

        this.ws.onclose = () => {
            this.handleClose()
        }

        this.ws.onerror = () => {
            this.clearConnectionTimeout()
            if (this.pendingConnect) {
                this.pendingConnect.reject(new BbxError('CONNECTION_FAILED', 'WebSocket connection failed'))
                this.pendingConnect = null
            }
        }

        this.ws.onmessage = (event) => {
            this.handleMessage(event)
        }
    }

    private handleMessage(event: MessageEvent): void {
        try {
            const message = JSON.parse(event.data as string) as ServerMessage

            switch (message.type) {
                case 'welcome':
                    this.clearConnectionTimeout()
                    this._nodeId = message.node_id
                    this._clockOffset = this.calculateClockOffset(message.server_time)
                    this.reconnectAttempts = 0
                    this.setState('connected')
                    this.startPingInterval()
                    if (this.pendingConnect) {
                        this.pendingConnect.resolve(message)
                        this.pendingConnect = null
                    }
                    break

                case 'pong':
                    this.handlePong(message.client_time, message.server_time)
                    break

                case 'error':
                    if (this.pendingConnect) {
                        this.pendingConnect.reject(new BbxError(message.code as 'INVALID_ROOM', message.message))
                        this.pendingConnect = null
                    }
                    break
            }

            this.callbacks.onMessage(message)
        } catch {
            // Ignore malformed messages
        }
    }

    private handleClose(): void {
        this.clearConnectionTimeout()
        this.stopPingInterval()
        this.ws = null
        this._nodeId = null

        if (this._state === 'connected' && this.config.reconnect) {
            this.scheduleReconnect()
        } else if (this._state !== 'reconnecting') {
            this.setState('disconnected')
        }
    }

    private scheduleReconnect(): void {
        if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
            this.setState('disconnected')
            return
        }

        this.reconnectAttempts++
        const delay = this.config.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)

        this.setState('reconnecting')
        this.callbacks.onReconnecting(this.reconnectAttempts, this.config.maxReconnectAttempts, delay)

        this.reconnectTimeout = setTimeout(() => {
            this.createWebSocket()
        }, delay)
    }

    private startPingInterval(): void {
        this.stopPingInterval()
        this.pingInterval = setInterval(() => {
            this.sendPing()
        }, this.config.pingInterval)
    }

    private stopPingInterval(): void {
        if (this.pingInterval) {
            clearInterval(this.pingInterval)
            this.pingInterval = null
        }
    }

    private clearConnectionTimeout(): void {
        if (this.connectionTimeoutId) {
            clearTimeout(this.connectionTimeoutId)
            this.connectionTimeoutId = null
        }
    }

    private sendPing(): void {
        this.lastPingTime = Date.now() * 1000
        this.send({
            type: 'ping',
            client_time: this.lastPingTime,
        })
    }

    private handlePong(clientTime: number, serverTime: number): void {
        const now = Date.now() * 1000
        const rtt = now - clientTime

        this._latency = rtt / 2 / 1000

        const t1 = clientTime
        const t2 = serverTime
        const t3 = serverTime
        const t4 = now
        this._clockOffset = ((t2 - t1 + (t3 - t4)) / 2 / 1000) | 0

        this.callbacks.onLatency(this._latency)
    }

    private calculateClockOffset(serverTime: number): number {
        const localTime = Date.now() * 1000
        return ((serverTime - localTime) / 1000) | 0
    }

    private setState(state: ConnectionState): void {
        if (this._state !== state) {
            this._state = state
            this.callbacks.onStateChange(state)
        }
    }

    private cleanup(): void {
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout)
            this.reconnectTimeout = null
        }

        this.clearConnectionTimeout()
        this.stopPingInterval()

        if (this.ws) {
            this.ws.onclose = null
            this.ws.onerror = null
            this.ws.onmessage = null
            this.ws.onopen = null
            this.ws.close()
            this.ws = null
        }

        this._nodeId = null
        this.reconnectAttempts = 0

        if (this.pendingConnect) {
            this.pendingConnect.reject(new BbxError('CONNECTION_FAILED', 'Connection cancelled'))
            this.pendingConnect = null
        }
    }
}
