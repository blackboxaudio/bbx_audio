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
 * Parameter Index Constants (matching template-plugin/parameters.json)
 * ============================================================================ */

/** Invert left channel phase (0.0 = off, 1.0 = on). */
#define PARAM_INVERT_LEFT 0

/** Invert right channel phase (0.0 = off, 1.0 = on). */
#define PARAM_INVERT_RIGHT 1

/** Channel routing mode (0 = Stereo, 1 = Left, 2 = Right, 3 = Swap). */
#define PARAM_CHANNEL_MODE 2

/** Sum to mono (0.0 = off, 1.0 = on). */
#define PARAM_MONO 3

/** Gain level in dB (-60 to +30). */
#define PARAM_GAIN 4

/** Pan position (-100 to +100). */
#define PARAM_PAN 5

/** DC offset removal enabled (0.0 = off, 1.0 = on). */
#define PARAM_DC_OFFSET 6

/** Total number of parameters. */
#define PARAM_COUNT 7

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
 * Opaque handle representing a DSP effects chain.
 */
typedef struct BbxGraph BbxGraph;

/* ============================================================================
 * Lifecycle Functions
 * ============================================================================ */

/**
 * Create a new DSP effects chain.
 *
 * @return Handle to the effects chain, or NULL if allocation fails.
 */
BbxGraph* bbx_graph_create(void);

/**
 * Destroy a DSP effects chain and free all associated resources.
 *
 * @param handle Effects chain handle (safe to call with NULL).
 */
void bbx_graph_destroy(BbxGraph* handle);

/**
 * Prepare the effects chain for playback with the given audio specifications.
 *
 * @param handle Effects chain handle.
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
 * Reset the effects chain state.
 *
 * @param handle Effects chain handle.
 * @return BBX_ERROR_OK on success, or an error code on failure.
 */
BbxError bbx_graph_reset(BbxGraph* handle);

/* ============================================================================
 * Audio Processing Functions
 * ============================================================================ */

/**
 * Process a block of audio through the effects chain.
 *
 * @param handle Effects chain handle.
 * @param inputs Array of input channel pointers.
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

#ifdef __cplusplus
}
#endif

#endif  /* BBX_FFI_H */
