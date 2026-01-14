/**
 * Message type definitions for bbx_net WebSocket protocol.
 * These types match the Rust protocol in bbx_net/src/websocket/protocol.rs.
 */

// Client -> Server message types
/* eslint-disable @typescript-eslint/no-unused-vars */
const clientMessageTypes = ['join', 'param', 'trigger', 'sync', 'ping', 'leave'] as const
export type ClientMessageType = (typeof clientMessageTypes)[number]

// Server -> Client message types
/* eslint-disable @typescript-eslint/no-unused-vars */
const serverMessageTypes = ['welcome', 'state', 'update', 'pong', 'error', 'closed'] as const
export type ServerMessageType = (typeof serverMessageTypes)[number]

// Client Messages

export interface IJoinMessage {
    type: 'join'
    room_code: string
    client_name?: string
}

export interface IParamMessage {
    type: 'param'
    param: string
    value: number
    at?: number
}

export interface ITriggerMessage {
    type: 'trigger'
    name: string
    at?: number
}

export interface ISyncMessage {
    type: 'sync'
}

export interface IPingMessage {
    type: 'ping'
    client_time: number
}

export interface ILeaveMessage {
    type: 'leave'
}

export type ClientMessage = IJoinMessage | IParamMessage | ITriggerMessage | ISyncMessage | IPingMessage | ILeaveMessage

// Server Messages

export interface IWelcomeMessage {
    type: 'welcome'
    node_id: string
    server_time: number
}

export interface IParamState {
    name: string
    value: number
    min: number
    max: number
}

export interface IStateMessage {
    type: 'state'
    params: IParamState[]
}

export interface IUpdateMessage {
    type: 'update'
    param: string
    value: number
}

export interface IPongMessage {
    type: 'pong'
    client_time: number
    server_time: number
}

export interface IErrorMessage {
    type: 'error'
    code: string
    message: string
}

export interface IRoomClosedMessage {
    type: 'closed'
}

export type ServerMessage =
    | IWelcomeMessage
    | IStateMessage
    | IUpdateMessage
    | IPongMessage
    | IErrorMessage
    | IRoomClosedMessage
