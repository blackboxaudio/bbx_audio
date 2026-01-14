/**
 * Error types for bbx_net WebSocket client.
 */

/* eslint-disable @typescript-eslint/no-unused-vars */
const bbxErrorCodes = ['INVALID_ROOM', 'ROOM_FULL', 'CONNECTION_FAILED', 'TIMEOUT', 'WEBSOCKET_ERROR'] as const
export type BbxErrorCode = (typeof bbxErrorCodes)[number]

export class BbxError extends Error {
    public readonly code: BbxErrorCode

    constructor(code: BbxErrorCode, message: string) {
        super(message)
        this.name = 'BbxError'
        this.code = code
    }
}
