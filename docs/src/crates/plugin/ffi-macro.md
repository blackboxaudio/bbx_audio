# FFI Macro

The `bbx_plugin_ffi!` macro generates C FFI exports for a `PluginDsp` implementation.

## Usage

```rust
use bbx_plugin::{PluginDsp, bbx_plugin_ffi};

pub struct MyPlugin { /* ... */ }

impl PluginDsp for MyPlugin { /* ... */ }

// Generate all FFI functions
bbx_plugin_ffi!(MyPlugin);
```

## Generated Functions

The macro generates these extern "C" functions:

### bbx_graph_create

```c
BbxGraph* bbx_graph_create(void);
```

Creates a new DSP instance. Returns NULL on allocation failure.

### bbx_graph_destroy

```c
void bbx_graph_destroy(BbxGraph* handle);
```

Destroys the DSP instance. Safe to call with NULL.

### bbx_graph_prepare

```c
BbxError bbx_graph_prepare(
    BbxGraph* handle,
    double sample_rate,
    uint32_t buffer_size,
    uint32_t num_channels
);
```

Prepares for playback. Calls `PluginDsp::prepare()`.

### bbx_graph_reset

```c
BbxError bbx_graph_reset(BbxGraph* handle);
```

Resets DSP state. Calls `PluginDsp::reset()`.

### bbx_graph_process

```c
void bbx_graph_process(
    BbxGraph* handle,
    const float* const* inputs,
    float* const* outputs,
    uint32_t num_channels,
    uint32_t num_samples,
    const float* params,
    uint32_t num_params
);
```

Processes audio. Calls `PluginDsp::apply_parameters()` then `PluginDsp::process()`.

## Internal Types

### BbxGraph

Opaque handle to the Rust DSP:

```rust
#[repr(C)]
pub struct BbxGraph {
    _private: [u8; 0],
}
```

Never dereference - it's a type-erased pointer to `GraphInner<T>`.

### BbxError

Error codes:

```rust
#[repr(C)]
pub enum BbxError {
    Ok = 0,
    NullPointer = 1,
    InvalidParameter = 2,
    InvalidBufferSize = 3,
    GraphNotPrepared = 4,
    AllocationFailed = 5,
}
```

## Macro Expansion

The macro expands to roughly:

```rust
type PluginGraphInner = GraphInner<MyPlugin>;

#[no_mangle]
pub extern "C" fn bbx_graph_create() -> *mut BbxGraph {
    let inner = Box::new(PluginGraphInner::new());
    handle_from_graph(inner)
}

#[no_mangle]
pub extern "C" fn bbx_graph_destroy(handle: *mut BbxGraph) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle as *mut PluginGraphInner));
        }
    }
}

// ... other functions
```

## Safety

The macro handles:

- Null pointer checks
- Parameter validation
- Safe type conversions
- Memory ownership transfer

## Custom Function Names

Currently, function names are fixed (`bbx_graph_*`). For custom names, you would need to write the FFI layer manually.
