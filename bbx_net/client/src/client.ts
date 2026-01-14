/**
 * BbxClient - WebSocket client for bbx_net audio control.
 */

import { Connection, type ConnectionState } from './connection.ts'
import type { ServerMessage, IWelcomeMessage, IStateMessage, IUpdateMessage, IErrorMessage } from './types.ts'

export const clientEventNames = [
    'connected',
    'disconnected',
    'state',
    'update',
    'error',
    'roomClosed',
    'latency',
] as const
export type ClientEventName = (typeof clientEventNames)[number]

export interface IBbxClientEvents {
    connected: (welcome: IWelcomeMessage) => void
    disconnected: (reason?: string) => void
    state: (state: IStateMessage) => void
    update: (update: IUpdateMessage) => void
    error: (error: IErrorMessage) => void
    roomClosed: () => void
    latency: (latencyMs: number) => void
}

export interface IBbxClientConfig {
    url: string
    roomCode: string
    clientName?: string
    reconnect?: boolean
    reconnectDelay?: number
    maxReconnectAttempts?: number
    pingInterval?: number
}

type EventHandler<K extends ClientEventName> = IBbxClientEvents[K]

export class BbxClient {
    private connection: Connection
    private handlers: Map<ClientEventName, Set<EventHandler<ClientEventName>>> = new Map()
    private _disconnectReason: string | undefined

    constructor(config: IBbxClientConfig) {
        const fullConfig = {
            url: config.url,
            roomCode: config.roomCode,
            clientName: config.clientName,
            reconnect: config.reconnect ?? true,
            reconnectDelay: config.reconnectDelay ?? 1000,
            maxReconnectAttempts: config.maxReconnectAttempts ?? 5,
            pingInterval: config.pingInterval ?? 5000,
        }

        this.connection = new Connection(fullConfig, {
            onMessage: (message) => this.handleMessage(message),
            onStateChange: (state) => this.handleStateChange(state),
            onLatency: (latencyMs) => this.emit('latency', latencyMs),
        })
    }

    get state(): ConnectionState {
        return this.connection.state
    }

    get nodeId(): string | null {
        return this.connection.nodeId
    }

    get latency(): number {
        return this.connection.latency
    }

    get clockOffset(): number {
        return this.connection.clockOffset
    }

    async connect(): Promise<void> {
        const welcome = await this.connection.connect()
        this.emit('connected', welcome)
    }

    disconnect(): void {
        this._disconnectReason = 'client requested disconnect'
        this.connection.disconnect()
    }

    setParam(param: string, value: number, at?: number): void {
        this.connection.send({
            type: 'param',
            param,
            value,
            at,
        })
    }

    trigger(name: string, at?: number): void {
        this.connection.send({
            type: 'trigger',
            name,
            at,
        })
    }

    requestSync(): void {
        this.connection.send({
            type: 'sync',
        })
    }

    on<K extends ClientEventName>(event: K, handler: IBbxClientEvents[K]): void {
        if (!this.handlers.has(event)) {
            this.handlers.set(event, new Set())
        }
        this.handlers.get(event)!.add(handler as EventHandler<ClientEventName>)
    }

    off<K extends ClientEventName>(event: K, handler: IBbxClientEvents[K]): void {
        const handlers = this.handlers.get(event)
        if (handlers) {
            handlers.delete(handler as EventHandler<ClientEventName>)
        }
    }

    toServerTime(localTimeMs: number): number {
        return localTimeMs * 1000 + this.clockOffset * 1000
    }

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
