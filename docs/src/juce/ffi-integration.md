# FFI Integration

The FFI (Foreign Function Interface) layer bridges Rust DSP code with C++ JUCE code.

## Overview

bbx_audio uses a C FFI for maximum compatibility:

```
Rust                     C FFI                    C++
-----                    -----                    ---
PluginDsp impl    -->    bbx_ffi.h    -->    bbx::Graph wrapper
```

## Key Components

### Rust Side

- **`bbx_plugin_ffi!` macro** - Generates all FFI exports
- **`BbxGraph` handle** - Opaque pointer type for C
- **`BbxError` enum** - Error codes for FFI returns

### C/C++ Side

- **`bbx_ffi.h`** - C header with function declarations
- **`bbx_graph.h`** - C++ RAII wrapper class

## Generated Functions

The `bbx_plugin_ffi!` macro generates these extern "C" functions:

| Function | Description |
|----------|-------------|
| `bbx_graph_create()` | Allocate and return a new DSP handle |
| `bbx_graph_destroy()` | Free the DSP handle and its resources |
| `bbx_graph_prepare()` | Initialize for given sample rate/buffer size |
| `bbx_graph_reset()` | Clear all DSP state |
| `bbx_graph_process()` | Process a block of audio |

## Error Handling

FFI functions return `BbxError` codes:

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

Check return values in C++:

```cpp
BbxError err = bbx_graph_prepare(handle, sampleRate, bufferSize, numChannels);
if (err != BBX_ERROR_OK) {
    // Handle error
}
```

## Memory Management

- **Allocation**: `bbx_graph_create()` allocates Rust memory
- **Ownership**: The handle owns the Rust struct
- **Deallocation**: `bbx_graph_destroy()` frees all Rust memory
- **Null Safety**: Functions are safe to call with NULL handles

## Thread Safety

- The DSP handle is `Send` (can be transferred between threads)
- Audio processing is single-threaded (call from audio thread only)
- Do not share handles between threads without synchronization

## See Also

- [C FFI Header Reference](ffi-header.md) - Complete `bbx_ffi.h` documentation
- [C++ RAII Wrapper](cpp-wrapper.md) - Using `bbx::Graph` in JUCE
