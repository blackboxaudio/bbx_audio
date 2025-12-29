# FFI Design

The design of bbx_audio's C FFI layer.

## Goals

1. **Safety** - Prevent memory errors across language boundary
2. **Simplicity** - Minimal API surface
3. **Performance** - Zero-copy audio processing
4. **Portability** - Works with any C-compatible language

## Opaque Handle Pattern

Rust types are hidden behind opaque pointers:

```c
typedef struct BbxGraph BbxGraph;  // Opaque - never dereference
```

C++ only sees the handle, never the Rust struct:

```cpp
BbxGraph* handle = bbx_graph_create();
// handle is a type-erased pointer
```

## Handle Lifecycle

### Creation

```rust
#[no_mangle]
pub extern "C" fn bbx_graph_create() -> *mut BbxGraph {
    let inner = Box::new(GraphInner::new());
    Box::into_raw(inner) as *mut BbxGraph
}
```

### Destruction

```rust
#[no_mangle]
pub extern "C" fn bbx_graph_destroy(handle: *mut BbxGraph) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle as *mut GraphInner));
        }
    }
}
```

## Error Handling

Return codes instead of exceptions:

```c
typedef enum BbxError {
    BBX_ERROR_OK = 0,
    BBX_ERROR_NULL_POINTER = 1,
    BBX_ERROR_INVALID_PARAMETER = 2,
    // ...
} BbxError;
```

Check in C++:

```cpp
BbxError err = bbx_graph_prepare(handle, ...);
if (err != BBX_ERROR_OK) {
    // Handle error
}
```

## Zero-Copy Processing

Audio buffers are passed by pointer:

```c
void bbx_graph_process(
    BbxGraph* handle,
    const float* const* inputs,   // Pointer to pointer
    float* const* outputs,
    ...
);
```

Rust converts to slices without copying:

```rust
unsafe {
    let input_slice = std::slice::from_raw_parts(inputs[ch], num_samples);
    let output_slice = std::slice::from_raw_parts_mut(outputs[ch], num_samples);
}
```
