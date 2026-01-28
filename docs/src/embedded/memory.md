# Memory Constraints

The STM32H750 on Daisy boards has a complex memory architecture with multiple regions optimized for different purposes. Understanding this layout is critical for realtime audio performance.

## STM32H750 Memory Map

| Region | Address | Size | Purpose |
|--------|---------|------|---------|
| FLASH | `0x08000000` | 128 KB | Program code (internal) |
| DTCM | `0x20000000` | 128 KB | Stack, fast variables |
| AXI SRAM | `0x24000000` | 512 KB | General purpose RAM |
| SRAM1 | `0x30000000` | 128 KB | DMA-accessible |
| SRAM2 | `0x30020000` | 128 KB | DMA-accessible |
| SRAM3 | `0x30040000` | 32 KB | Audio DMA buffers |
| SRAM4 | `0x38000000` | 64 KB | Battery-backed |
| Backup | `0x38800000` | 4 KB | Persistent storage |

### Memory Regions Explained

**DTCM (Data Tightly Coupled Memory)**: The fastest RAM, directly connected to the CPU. Used for the stack and performance-critical variables. Place frequently accessed data here.

**AXI SRAM**: Large general-purpose memory for heap allocations and large buffers not requiring DMA access.

**SRAM1/SRAM2**: D2 domain memory accessible by DMA1/DMA2. Use for large audio buffers that need DMA transfer.

**SRAM3**: Ideal for audio DMA buffers. The SAI peripheral uses DMA to transfer samples here.

**SRAM4/Backup**: Battery-backed memory that persists across resets. Use for storing presets or calibration data.

## Audio Buffer Memory Usage

Typical memory usage for audio processing:

| Component | Size (512 samples) | Location |
|-----------|-------------------|----------|
| `FrameBuffer` | 4 KB | SRAM3 (DMA) |
| `StaticSampleBuffer` | 2 KB | DTCM or stack |
| DSP block state | varies | DTCM |
| Lookup tables | varies | FLASH or AXI |

For a stereo 512-sample buffer at 32-bit float:
- `FrameBuffer`: 512 frames × 2 channels × 4 bytes = 4,096 bytes
- `StaticSampleBuffer`: 512 samples × 4 bytes = 2,048 bytes

## Stack vs Heap Patterns

### Prefer Stack Allocation

Embedded audio code should avoid heap allocation entirely:

```rust
// Good: Stack-allocated buffer
let mut buffer = StaticSampleBuffer::<512>::new();

// Avoid: Heap allocation (requires alloc crate, not realtime-safe)
let mut buffer = Vec::with_capacity(512);
```

### Use const generics for buffer sizes

```rust
struct DelayLine<const N: usize> {
    buffer: [f32; N],
    write_pos: usize,
}

impl<const N: usize> DelayLine<N> {
    const fn new() -> Self {
        Self {
            buffer: [0.0; N],
            write_pos: 0,
        }
    }
}
```

## Memory Optimization Tips

### 1. Use lookup tables for expensive functions

Pre-compute sine waves, filter coefficients, or other expensive calculations:

```rust
const SINE_TABLE_SIZE: usize = 1024;
static SINE_TABLE: [f32; SINE_TABLE_SIZE] = {
    let mut table = [0.0; SINE_TABLE_SIZE];
    let mut i = 0;
    while i < SINE_TABLE_SIZE {
        // const-compatible sin approximation
        table[i] = /* ... */;
        i += 1;
    }
    table
};
```

### 2. Pack struct fields efficiently

Order fields from largest to smallest to minimize padding:

```rust
// Good: 16 bytes
struct Oscillator {
    phase: f64,      // 8 bytes
    frequency: f32,  // 4 bytes
    waveform: u8,    // 1 byte + 3 padding
}

// Bad: 24 bytes (poor alignment)
struct OscillatorBad {
    waveform: u8,    // 1 byte + 7 padding
    phase: f64,      // 8 bytes
    frequency: f32,  // 4 bytes + 4 padding
}
```

### 3. Use f32 instead of f64

The Cortex-M7 FPU handles f32 natively but emulates f64:

```rust
// Fast: native FPU
let sample: f32 = 0.5;

// Slow: software emulation
let sample: f64 = 0.5;
```

### 4. Place DMA buffers in correct memory region

Use linker section attributes for DMA buffers:

```rust
#[link_section = ".sram3"]
static mut DMA_BUFFER: [u32; 512] = [0; 512];
```

## Monitoring Memory Usage

Check binary size and memory layout:

```bash
# Show section sizes
arm-none-eabi-size target/thumbv7em-none-eabihf/release/examples/02_oscillator

# Detailed memory map
arm-none-eabi-objdump -h target/thumbv7em-none-eabihf/release/examples/02_oscillator
```

Typical output:
```
   text    data     bss     dec     hex filename
  12345     100     200   12645    3165 02_oscillator
```

- **text**: Code and read-only data (in FLASH)
- **data**: Initialized global/static variables
- **bss**: Zero-initialized global/static variables
