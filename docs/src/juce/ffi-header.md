# C FFI Header Reference

Complete reference for `bbx_ffi.h`.

## Header Overview

```c
#ifndef BBX_FFI_H
#define BBX_FFI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Types and functions...

#ifdef __cplusplus
}
#endif

#endif  /* BBX_FFI_H */
```

## Types

### BbxError

Error codes for FFI operations:

```c
typedef enum BbxError {
    BBX_ERROR_OK = 0,
    BBX_ERROR_NULL_POINTER = 1,
    BBX_ERROR_INVALID_PARAMETER = 2,
    BBX_ERROR_INVALID_BUFFER_SIZE = 3,
    BBX_ERROR_GRAPH_NOT_PREPARED = 4,
    BBX_ERROR_ALLOCATION_FAILED = 5,
} BbxError;
```

| Error | Value | Description |
|-------|-------|-------------|
| `BBX_ERROR_OK` | 0 | Operation succeeded |
| `BBX_ERROR_NULL_POINTER` | 1 | Handle was NULL |
| `BBX_ERROR_INVALID_PARAMETER` | 2 | Invalid parameter value |
| `BBX_ERROR_INVALID_BUFFER_SIZE` | 3 | Buffer size was 0 |
| `BBX_ERROR_GRAPH_NOT_PREPARED` | 4 | Graph not prepared before processing |
| `BBX_ERROR_ALLOCATION_FAILED` | 5 | Memory allocation failed |

### BbxGraph

Opaque handle to the Rust DSP:

```c
typedef struct BbxGraph BbxGraph;
```

Never dereference or inspect this pointer - it's an opaque handle.

### BbxMidiStatus

MIDI message status types:

```c
typedef enum BbxMidiStatus {
    BBX_MIDI_STATUS_UNKNOWN = 0,
    BBX_MIDI_STATUS_NOTE_OFF = 1,
    BBX_MIDI_STATUS_NOTE_ON = 2,
    BBX_MIDI_STATUS_POLYPHONIC_AFTERTOUCH = 3,
    BBX_MIDI_STATUS_CONTROL_CHANGE = 4,
    BBX_MIDI_STATUS_PROGRAM_CHANGE = 5,
    BBX_MIDI_STATUS_CHANNEL_AFTERTOUCH = 6,
    BBX_MIDI_STATUS_PITCH_WHEEL = 7,
} BbxMidiStatus;
```

### BbxMidiMessage

MIDI message structure (matches Rust `MidiMessage` with `repr(C)`):

```c
typedef struct BbxMidiMessage {
    uint8_t channel;       // MIDI channel (0-15)
    BbxMidiStatus status;  // Message type
    uint8_t data_1;        // First data byte (e.g., note number)
    uint8_t data_2;        // Second data byte (e.g., velocity)
} BbxMidiMessage;
```

### BbxMidiEvent

MIDI event with sample-accurate timing:

```c
typedef struct BbxMidiEvent {
    BbxMidiMessage message;   // The MIDI message
    uint32_t sample_offset;   // Sample offset within the buffer
} BbxMidiEvent;
```

The `sample_offset` allows sample-accurate MIDI timing within a buffer.

## Functions

### bbx_graph_create

```c
BbxGraph* bbx_graph_create(void);
```

Create a new DSP effects chain.

**Returns**: Handle to the effects chain, or `NULL` if allocation fails.

**Usage**:
```c
BbxGraph* handle = bbx_graph_create();
if (handle == NULL) {
    // Allocation failed
}
```

### bbx_graph_destroy

```c
void bbx_graph_destroy(BbxGraph* handle);
```

Destroy a DSP effects chain and free all resources.

**Parameters**:
- `handle` - Effects chain handle (safe to call with `NULL`)

**Usage**:
```c
bbx_graph_destroy(handle);
handle = NULL;  // Avoid dangling pointer
```

### bbx_graph_prepare

```c
BbxError bbx_graph_prepare(
    BbxGraph* handle,
    double sample_rate,
    uint32_t buffer_size,
    uint32_t num_channels
);
```

Prepare the effects chain for playback.

**Parameters**:
- `handle` - Effects chain handle
- `sample_rate` - Sample rate in Hz (e.g., 44100.0, 48000.0)
- `buffer_size` - Number of samples per buffer
- `num_channels` - Number of audio channels

**Returns**: `BBX_ERROR_OK` on success, or an error code.

**Usage**:
```c
BbxError err = bbx_graph_prepare(handle, 44100.0, 512, 2);
if (err != BBX_ERROR_OK) {
    // Handle error
}
```

### bbx_graph_reset

```c
BbxError bbx_graph_reset(BbxGraph* handle);
```

Reset the effects chain state (clear filters, delay lines, etc.).

**Parameters**:
- `handle` - Effects chain handle

**Returns**: `BBX_ERROR_OK` on success.

**Usage**:
```c
bbx_graph_reset(handle);
```

### bbx_graph_process

```c
void bbx_graph_process(
    BbxGraph* handle,
    const float* const* inputs,
    float* const* outputs,
    uint32_t num_channels,
    uint32_t num_samples,
    const float* params,
    uint32_t num_params,
    const BbxMidiEvent* midi_events,
    uint32_t num_midi_events
);
```

Process a block of audio with optional MIDI events.

**Parameters**:
- `handle` - Effects chain handle
- `inputs` - Array of input channel pointers
- `outputs` - Array of output channel pointers
- `num_channels` - Number of audio channels
- `num_samples` - Number of samples per channel
- `params` - Pointer to flat array of parameter values
- `num_params` - Number of parameters
- `midi_events` - Pointer to array of MIDI events (may be `NULL` for effects)
- `num_midi_events` - Number of MIDI events in the array

**Usage**:
```c
float params[PARAM_COUNT] = { gainValue, panValue, ... };

// For effects (no MIDI):
bbx_graph_process(
    handle,
    inputPtrs,
    outputPtrs,
    numChannels,
    numSamples,
    params,
    PARAM_COUNT,
    NULL,
    0
);

// For synthesizers (with MIDI):
BbxMidiEvent midiEvents[16];
uint32_t numMidiEvents = convertJuceMidi(midiBuffer, midiEvents, 16);

bbx_graph_process(
    handle,
    inputPtrs,
    outputPtrs,
    numChannels,
    numSamples,
    params,
    PARAM_COUNT,
    midiEvents,
    numMidiEvents
);
```

Note: Parameter index constants (`PARAM_*`) are defined in the generated `bbx_params.h` header, not in `bbx_ffi.h`. See [Parameter Code Generation](parameters-codegen.md) for details.
