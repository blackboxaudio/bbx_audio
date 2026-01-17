# Real-Time Safety

Constraints and patterns for audio thread processing.

## What is Real-Time Safe?

Audio processing must complete within a fixed time budget (buffer duration). Operations that can cause unbounded delays are forbidden.

## Forbidden in Audio Thread

### Memory Allocation

```rust
// BAD - Heap allocation
let vec = Vec::new();
let boxed = Box::new(value);

// GOOD - Pre-allocated
let mut vec: StackVec<f32, 8> = StackVec::new();
```

### Locking

```rust
// BAD - Can block
let guard = mutex.lock().unwrap();

// GOOD - Lock-free
let value = atomic.load(Ordering::Relaxed);
```

### System Calls

```rust
// BAD - Unbounded time
std::fs::read_to_string("file.txt");
std::thread::sleep(duration);

// GOOD - Pre-loaded data
let data = &self.preloaded_buffer[..];
```

## bbx_audio's Approach

### Stack Allocation

Using `StackVec` for bounded collections:

```rust
const MAX_BLOCK_INPUTS: usize = 8;
let mut input_slices: StackVec<&[S], MAX_BLOCK_INPUTS> = StackVec::new();
```

### Pre-computed Lookups

Connection indices computed during prepare:

```rust
// Computed once in prepare()
self.block_input_buffers = vec![Vec::new(); self.blocks.len()];

// O(1) lookup during processing
let input_indices = &self.block_input_buffers[block_id.0];
```

### Pre-allocated Buffers

All audio buffers allocated upfront:

```rust
// During add_block()
self.audio_buffers.push(AudioBuffer::new(self.buffer_size));

// During process - just clear
buffer.zeroize();
```

## Performance Considerations

### Cache Efficiency

- Contiguous buffer storage
- Sequential processing order
- Minimal pointer chasing

### Branch Prediction

- Consistent code paths
- Avoid data-dependent branches
- Use branchless algorithms where possible

### SIMD Potential

- Buffer alignment
- Processing in chunks
- Sample-type genericity
