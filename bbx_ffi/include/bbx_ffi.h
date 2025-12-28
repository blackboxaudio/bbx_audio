/* bbx_ffi - C FFI bindings for bbx_audio DSP library */

#ifndef BBX_FFI_H
#define BBX_FFI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Parameter Index Constants
 * ============================================================================ */

/** Oscillator base frequency in Hz. */
#define PARAM_OSC_FREQUENCY 0

/** Oscillator pitch offset in semitones. */
#define PARAM_OSC_PITCH_OFFSET 1

/** Envelope attack time in seconds. */
#define PARAM_ENV_ATTACK 2

/** Envelope decay time in seconds. */
#define PARAM_ENV_DECAY 3

/** Envelope sustain level (0.0 to 1.0). */
#define PARAM_ENV_SUSTAIN 4

/** Envelope release time in seconds. */
#define PARAM_ENV_RELEASE 5

/** LFO frequency in Hz. */
#define PARAM_LFO_FREQUENCY 6

/** LFO depth (0.0 to 1.0). */
#define PARAM_LFO_DEPTH 7

/** Overdrive drive amount. */
#define PARAM_DRIVE 8

/** Output level (0.0 to 1.0). */
#define PARAM_LEVEL 9

/** Total number of parameters. */
#define PARAM_COUNT 10

/* ============================================================================
 * Types
 * ============================================================================ */

/**
 * Error codes for bbx_audio operations.
 */
typedef enum BbxError {
    BBX_ERROR_OK = 0,
    BBX_ERROR_NULL_POINTER = 1,
    BBX_ERROR_INVALID_PARAMETER = 2,
    BBX_ERROR_INVALID_BUFFER_SIZE = 3,
    BBX_ERROR_GRAPH_NOT_PREPARED = 4,
    BBX_ERROR_ALLOCATION_FAILED = 5,
} BbxError;

/**
 * MIDI message status byte types.
 */
typedef enum MidiMessageStatus {
    MIDI_MESSAGE_STATUS_UNKNOWN = 0,
    MIDI_MESSAGE_STATUS_NOTE_OFF = 1,
    MIDI_MESSAGE_STATUS_NOTE_ON = 2,
    MIDI_MESSAGE_STATUS_POLYPHONIC_AFTERTOUCH = 3,
    MIDI_MESSAGE_STATUS_CONTROL_CHANGE = 4,
    MIDI_MESSAGE_STATUS_PROGRAM_CHANGE = 5,
    MIDI_MESSAGE_STATUS_CHANNEL_AFTERTOUCH = 6,
    MIDI_MESSAGE_STATUS_PITCH_WHEEL = 7,
} MidiMessageStatus;

/**
 * Opaque handle representing a DSP graph.
 */
typedef struct BbxGraph BbxGraph;

/**
 * MIDI message structure.
 */
typedef struct MidiMessage {
    uint8_t channel;
    MidiMessageStatus status;
    uint8_t data_1;
    uint8_t data_2;
} MidiMessage;

/* ============================================================================
 * Lifecycle Functions
 * ============================================================================ */

/**
 * Create a new DSP graph.
 *
 * @return Handle to the graph, or NULL if allocation fails.
 */
BbxGraph* bbx_graph_create(void);

/**
 * Destroy a DSP graph and free all associated resources.
 *
 * @param handle Graph handle (safe to call with NULL).
 */
void bbx_graph_destroy(BbxGraph* handle);

/**
 * Prepare the graph for playback with the given audio specifications.
 *
 * @param handle Graph handle.
 * @param sample_rate Sample rate in Hz (e.g., 44100.0, 48000.0).
 * @param buffer_size Number of samples per buffer.
 * @param num_channels Number of audio channels.
 * @return BBX_ERROR_OK on success, or an error code on failure.
 */
BbxError bbx_graph_prepare(BbxGraph* handle,
                           double sample_rate,
                           uint32_t buffer_size,
                           uint32_t num_channels);

/**
 * Reset the graph state.
 *
 * @param handle Graph handle.
 * @return BBX_ERROR_OK on success, or an error code on failure.
 */
BbxError bbx_graph_reset(BbxGraph* handle);

/* ============================================================================
 * Audio Processing Functions
 * ============================================================================ */

/**
 * Process a block of audio.
 *
 * @param handle Graph handle.
 * @param inputs Array of input channel pointers (can be NULL for synths).
 * @param outputs Array of output channel pointers.
 * @param num_channels Number of audio channels.
 * @param num_samples Number of samples per channel.
 * @param params Pointer to flat float array of parameter values.
 * @param num_params Number of parameters in the array.
 */
void bbx_graph_process(BbxGraph* handle,
                       const float* const* inputs,
                       float* const* outputs,
                       uint32_t num_channels,
                       uint32_t num_samples,
                       const float* params,
                       uint32_t num_params);

/* ============================================================================
 * MIDI Functions
 * ============================================================================ */

/**
 * Add MIDI events to the graph's buffer for processing.
 *
 * @param handle Graph handle.
 * @param events Pointer to array of MIDI messages.
 * @param num_events Number of events in the array.
 * @return BBX_ERROR_OK on success, or an error code on failure.
 */
BbxError bbx_graph_add_midi_events(BbxGraph* handle,
                                   const MidiMessage* events,
                                   uint32_t num_events);

/**
 * Clear accumulated MIDI events.
 *
 * @param handle Graph handle.
 */
void bbx_graph_clear_midi(BbxGraph* handle);

#ifdef __cplusplus
}
#endif

#endif  /* BBX_FFI_H */
