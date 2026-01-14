/**
 * Message type definitions for bbx_net WebSocket protocol.
 * These types match the Rust protocol in bbx_net/src/websocket/protocol.rs.
 */

import type { BbxErrorCode } from './errors.ts'

// Client -> Server message types
/* eslint-disable @typescript-eslint/no-unused-vars */
const clientMessageTypes = ['join', 'param', 'trigger', 'sync', 'ping', 'leave'] as const

/** Message types that clients can send to the server. */
export type ClientMessageType = (typeof clientMessageTypes)[number]

// Server -> Client message types
/* eslint-disable @typescript-eslint/no-unused-vars */
const serverMessageTypes = ['welcome', 'state', 'update', 'pong', 'error', 'closed'] as const

/** Message types that the server sends to clients. */
export type ServerMessageType = (typeof serverMessageTypes)[number]

// Client Messages

/** Request to join a room. Sent automatically when connecting. */
export interface IJoinMessage {
    type: 'join'
    /** The room code to join. */
    room_code: string
    /** Optional display name for this client. */
    client_name?: string
}

/** Request to change a parameter value. */
export interface IParamMessage {
    type: 'param'
    /** The parameter name to change. */
    param: string
    /** The new value for the parameter. */
    value: number
    /** Optional server timestamp (µs) for scheduled execution. */
    at?: number
}

/** Request to trigger a named event. */
export interface ITriggerMessage {
    type: 'trigger'
    /** The trigger name. */
    name: string
    /** Optional server timestamp (µs) for scheduled execution. */
    at?: number
}

/** Request current parameter state from the server. */
export interface ISyncMessage {
    type: 'sync'
}

/** Ping message for latency measurement and clock synchronization. */
export interface IPingMessage {
    type: 'ping'
    /** Client timestamp in microseconds when the ping was sent. */
    client_time: number
}

/** Request to leave the current room. */
export interface ILeaveMessage {
    type: 'leave'
}

/** Union of all client-to-server message types. */
export type ClientMessage = IJoinMessage | IParamMessage | ITriggerMessage | ISyncMessage | IPingMessage | ILeaveMessage

// Server Messages

/** Confirmation that the client successfully joined a room. */
export interface IWelcomeMessage {
    type: 'welcome'
    /** Unique identifier assigned to this client for the session. */
    node_id: string
    /** Server timestamp in microseconds, used for clock synchronization. */
    server_time: number
}

/** Snapshot of a single parameter's state. */
export interface IParamState {
    /** The parameter name. */
    name: string
    /** Current value of the parameter. */
    value: number
    /** Minimum allowed value. */
    min: number
    /** Maximum allowed value. */
    max: number
}

/** Full state sync containing all parameters. Sent after joining and on sync requests. */
export interface IStateMessage {
    type: 'state'
    /** Array of all parameter states. */
    params: IParamState[]
}

/** Notification that a parameter was updated (by another client or the server). */
export interface IUpdateMessage {
    type: 'update'
    /** The parameter name that changed. */
    param: string
    /** The new value. */
    value: number
}

/** Response to a ping message for latency calculation. */
export interface IPongMessage {
    type: 'pong'
    /** The client timestamp from the original ping (echoed back). */
    client_time: number
    /** Server timestamp when the pong was sent. */
    server_time: number
}

/** Error notification from the server. */
export interface IErrorMessage {
    type: 'error'
    /** Error code identifying the error type. */
    code: BbxErrorCode | string
    /** Human-readable error description. */
    message: string
}

/** Notification that the room was closed by the host. */
export interface IRoomClosedMessage {
    type: 'closed'
}

/** Union of all server-to-client message types. */
export type ServerMessage =
    | IWelcomeMessage
    | IStateMessage
    | IUpdateMessage
    | IPongMessage
    | IErrorMessage
    | IRoomClosedMessage
