/**
 * Error types for bbx_net WebSocket client.
 */

/* eslint-disable @typescript-eslint/no-unused-vars */
const bbxErrorCodes = ['INVALID_ROOM', 'ROOM_FULL', 'CONNECTION_FAILED', 'TIMEOUT', 'WEBSOCKET_ERROR'] as const

/**
 * Error codes returned by the bbx_net server or client.
 *
 * - `INVALID_ROOM` - The room code doesn't exist or has expired
 * - `ROOM_FULL` - The room has reached its maximum client capacity
 * - `CONNECTION_FAILED` - WebSocket connection could not be established
 * - `TIMEOUT` - Connection or operation timed out
 * - `WEBSOCKET_ERROR` - Generic WebSocket error occurred
 */
export type BbxErrorCode = (typeof bbxErrorCodes)[number]

/**
 * Custom error class for bbx_net client errors.
 * Extends `Error` with a typed error code for programmatic error handling.
 *
 * @example
 * ```ts
 * try {
 *     await client.connect()
 * } catch (err) {
 *     if (err instanceof BbxError && err.code === 'INVALID_ROOM') {
 *         console.error('Room not found')
 *     }
 * }
 * ```
 */
export class BbxError extends Error {
    /** The error code identifying the type of error. */
    public readonly code: BbxErrorCode

    /**
     * @param code - The error code
     * @param message - Human-readable error description
     */
    constructor(code: BbxErrorCode, message: string) {
        super(message)
        this.name = 'BbxError'
        this.code = code
    }
}
