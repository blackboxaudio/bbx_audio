# Memory Safety Across Boundaries

Ensuring memory safety in the FFI layer.

## Safety Invariants

### 1. Valid Handles

Handles are valid from `create()` to `destroy()`:

```rust
pub extern "C" fn bbx_graph_process(handle: *mut BbxGraph, ...) {
    if handle.is_null() {
        return;  // Silent no-op for safety
    }
    // Handle is valid
}
```

### 2. Non-Overlapping Buffers

Input and output buffers never overlap:

```rust
// SAFETY: Our buffer indexing guarantees:
// 1. Input indices come from other blocks' outputs
// 2. Output indices are unique to this block
// 3. Therefore, input and output NEVER overlap
unsafe {
    let input_slices = /* from input buffers */;
    let output_slices = /* from output buffers */;
    block.process(input_slices, output_slices, ...);
}
```

### 3. Valid Pointer Lengths

Buffer lengths match the provided count:

```rust
unsafe {
    let slice = std::slice::from_raw_parts(
        inputs[ch],
        num_samples as usize,  // Must be accurate!
    );
}
```

## Unsafe Blocks

All unsafe code is documented:

```rust
// SAFETY: [explanation of why this is safe]
unsafe {
    // unsafe operation
}
```

## C++ Responsibilities

The C++ side must:

1. **Never use handle after destroy**
2. **Provide valid buffer pointers**
3. **Match buffer sizes to declared counts**
4. **Not call from multiple threads simultaneously**

## Defense in Depth

Multiple layers of protection:

1. **Null checks** - Explicit handle validation
2. **Bounds checks** - Array access validation
3. **Type system** - Compile-time generic checking
4. **Debug asserts** - Development-time validation

```rust
debug_assert!(
    output_count <= MAX_BLOCK_OUTPUTS,
    "Block output count exceeds limit"
);
```
