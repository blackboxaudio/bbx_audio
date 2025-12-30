# Stack Vector

A fixed-capacity vector that stores elements on the stack, avoiding heap allocations.

## Overview

`StackVec` is a vector-like container with:

- Fixed maximum capacity (compile-time constant)
- No heap allocations
- Safe push/pop operations
- Panic-free overflow handling

## API

### Creating a StackVec

```rust
use bbx_core::StackVec;

// Create an empty stack vector with capacity for 8 f32s
let mut vec: StackVec<f32, 8> = StackVec::new();

// Create from an array
let vec = StackVec::from([1.0, 2.0, 3.0]);
```

### Adding Elements

```rust
use bbx_core::StackVec;

let mut vec: StackVec<f32, 4> = StackVec::new();

// Push returns Ok if there's space
assert!(vec.push(1.0).is_ok());
assert!(vec.push(2.0).is_ok());

// Or use try_push for fallible insertion
if vec.try_push(3.0).is_some() {
    // Element was added
}
```

### Removing Elements

```rust
use bbx_core::StackVec;

let mut vec: StackVec<f32, 4> = StackVec::from([1.0, 2.0, 3.0]);

// Pop from the end
assert_eq!(vec.pop(), Some(3.0));
assert_eq!(vec.pop(), Some(2.0));

// Clear all elements
vec.clear();
assert!(vec.is_empty());
```

### Accessing Elements

```rust
use bbx_core::StackVec;

let mut vec: StackVec<f32, 4> = StackVec::from([1.0, 2.0, 3.0]);

// Index access
assert_eq!(vec[0], 1.0);

// Safe access with get
if let Some(value) = vec.get(1) {
    println!("Second element: {}", value);
}

// Mutable access
vec[0] = 10.0;
```

### Iteration

```rust
use bbx_core::StackVec;

let vec: StackVec<f32, 4> = StackVec::from([1.0, 2.0, 3.0]);

// Immutable iteration
for value in &vec {
    println!("{}", value);
}

// Mutable iteration
let mut vec = vec;
for value in &mut vec {
    *value *= 2.0;
}
```

## Usage in Audio Processing

### Per-Block Buffers

```rust
use bbx_core::StackVec;

const MAX_INPUTS: usize = 8;

fn collect_inputs(inputs: &[f32]) -> StackVec<f32, MAX_INPUTS> {
    let mut result = StackVec::new();
    for &input in inputs.iter().take(MAX_INPUTS) {
        let _ = result.push(input);
    }
    result
}
```

### Modulation Value Collection

```rust
use bbx_core::StackVec;

const MAX_MODULATORS: usize = 4;

struct ModulationContext {
    values: StackVec<f32, MAX_MODULATORS>,
}

impl ModulationContext {
    fn add_modulator(&mut self, value: f32) {
        let _ = self.values.push(value);
    }

    fn total_modulation(&self) -> f32 {
        self.values.iter().sum()
    }
}
```

## Comparison with Other Types

| Type | Heap Allocation | Fixed Size | Growable |
|------|-----------------|------------|----------|
| `Vec<T>` | Yes | No | Yes |
| `[T; N]` | No | Yes | No |
| `StackVec<T, N>` | No | Yes (max) | Yes (up to N) |
| `ArrayVec<T, N>` | No | Yes (max) | Yes (up to N) |

`StackVec` is similar to `arrayvec::ArrayVec` but is part of bbx_core with no external dependencies.

## Limitations

- Maximum capacity is fixed at compile time
- Capacity is part of the type (`StackVec<T, 4>` vs `StackVec<T, 8>`)
- Not suitable for large or unknown-size collections
