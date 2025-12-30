# Handle Management

How Rust objects are managed across the FFI boundary.

## The Box Pattern

Rust objects are boxed and converted to raw pointers:

```rust
// Create: Rust -> C
let inner = Box::new(GraphInner::new());
let ptr = Box::into_raw(inner);
return ptr as *mut BbxGraph;

// Destroy: C -> Rust
let inner = Box::from_raw(ptr as *mut GraphInner);
drop(inner);  // Automatically called when Box goes out of scope
```

## Type Erasure

The C type is opaque:

```c
typedef struct BbxGraph BbxGraph;
```

Rust knows the actual type:

```rust
type PluginGraphInner = GraphInner<PluginGraph>;
```

Conversion is safe because:
1. We control both sides
2. Types match at compile time via generics
3. Handle is never dereferenced in C

## Null Safety

All functions check for null:

```rust
pub extern "C" fn bbx_graph_prepare(handle: *mut BbxGraph, ...) -> BbxError {
    if handle.is_null() {
        return BbxError::NullPointer;
    }
    // ...
}
```

## Ownership Transfer

```
create():  Rust owns -> Box::into_raw -> C owns handle
use():     C passes handle -> Rust borrows -> returns to C
destroy(): C passes handle -> Rust reclaims -> deallocates
```

### Create

```rust
Box::into_raw(inner)  // Rust gives up ownership
```

### Use

```rust
let inner = &mut *(handle as *mut GraphInner);  // Borrow, don't take ownership
```

### Destroy

```rust
Box::from_raw(handle)  // Rust reclaims ownership
// Box dropped, memory freed
```

## RAII in C++

The C++ wrapper manages the handle:

```cpp
class Graph {
public:
    Graph() : m_handle(bbx_graph_create()) {}
    ~Graph() { if (m_handle) bbx_graph_destroy(m_handle); }

private:
    BbxGraph* m_handle;
};
```
