# Troubleshooting

Common issues and solutions when developing with bbx_daisy.

## Compilation Errors

### "can't find crate for `std`"

The code is trying to use the standard library, which isn't available on embedded targets.

**Solution**: Ensure your crate has `#![no_std]` at the top and doesn't depend on std-requiring crates.

```rust
#![no_std]
#![no_main]
```

### "linker `rust-lld` not found"

The ARM linker isn't available.

**Solution**: Install the ARM target:
```bash
rustup target add thumbv7em-none-eabihf
```

### "memory.x not found" or linker errors about `_stack_start`

The linker script isn't being found.

**Solution**: Ensure you're building from the workspace root or that `memory.x` is in your crate's root directory. The `build.rs` script should copy it to the output directory.

### "FLASH region overflow"

Your binary exceeds the 128 KB internal flash.

**Solutions**:
- Enable `--release` mode for optimizations
- Enable LTO in your Cargo.toml:
  ```toml
  [profile.release]
  lto = true
  ```
- Remove unused dependencies
- Use `#[inline(never)]` on rarely-called functions to prevent bloat from inlining

### "undefined symbol: __aeabi_*" or floating point errors

Missing software floating point support.

**Solution**: Ensure you're using the correct target with hardware FPU:
```bash
cargo build --target thumbv7em-none-eabihf  # Correct (with FPU)
cargo build --target thumbv7em-none-eabi    # Wrong (software float)
```

## Flashing Issues

### "no probe was found"

probe-rs can't detect your debug probe.

**Solutions**:
1. Check USB connection
2. Install udev rules (Linux):
   ```bash
   curl -fsSL https://probe.rs/files/69-probe-rs.rules | sudo tee /etc/udev/rules.d/69-probe-rs.rules
   sudo udevadm control --reload-rules
   ```
3. Try a different USB port or cable
4. Verify the probe is recognized: `probe-rs list`

### "target chip not found" or wrong chip detected

probe-rs is detecting a different chip than expected.

**Solution**: Specify the chip explicitly:
```bash
probe-rs run --chip STM32H750VBTx target/thumbv7em-none-eabihf/release/examples/02_oscillator
```

### DFU: "No DFU capable USB device available"

The Daisy isn't in DFU mode.

**Solution**: Enter DFU mode correctly:
1. Hold the **BOOT** button
2. Tap the **RESET** button (while still holding BOOT)
3. Release the **BOOT** button
4. The device should now appear in `dfu-util -l`

### DFU: "Cannot claim interface"

Another program is using the USB device.

**Solutions**:
- Close any other programs accessing the Daisy
- On Linux, you may need sudo: `sudo dfu-util ...`
- Install udev rules for non-root access

## Audio Issues

### No sound output

**Check these in order**:

1. **Connections**: Verify audio output is connected to the correct pins
2. **Codec initialization**: Ensure the codec is being initialized (the `bbx_daisy_audio!` macro handles this)
3. **Output levels**: Check your code isn't outputting zeros or very small values
4. **Sample rate mismatch**: Verify `DEFAULT_SAMPLE_RATE` matches your calculations

### Crackling or distorted audio

**Causes and solutions**:

1. **Buffer underruns**: Your `process()` function is taking too long
   - Simplify DSP calculations
   - Use lookup tables instead of expensive functions
   - Ensure you're using `--release` builds

2. **Denormals**: Very small floating point values cause CPU spikes
   - Use the denormal-flushing utilities from `bbx_core`
   - Add small DC offset to signals that fade to zero

3. **Integer overflow in samples**: Output values exceed [-1.0, 1.0]
   - Add clipping or saturation to your output stage

4. **Wrong buffer size**: Mismatch between expected and actual buffer sizes
   - Use `BLOCK_SIZE` constant consistently

### Audio is at wrong pitch

**Causes**:

1. **Sample rate mismatch**: Code assumes different sample rate than hardware
   ```rust
   // Correct: use the constant
   let phase_inc = frequency / DEFAULT_SAMPLE_RATE;

   // Wrong: hardcoded value
   let phase_inc = frequency / 44100.0;
   ```

2. **Stereo vs mono confusion**: Processing only left channel or duplicating samples incorrectly

## Debugging Techniques

### Using defmt/RTT

Add debug output without UART:

```rust
use defmt::info;

impl AudioProcessor for MyProcessor {
    fn process(&mut self, input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        info!("Processing block, first sample: {}", input.left(0));
        // ...
    }
}
```

### LED indicators

Use the onboard LED for status indication:

```rust
use bbx_daisy::prelude::*;

// In your main or init code
let mut led = Led::new(board.gpioc.pc7.into_push_pull_output());

// Toggle on each buffer (shows audio is running)
led.toggle();

// Or indicate errors
if error_condition {
    led.on();
}
```

### Cycle counting

Measure performance using the DWT cycle counter:

```rust
use cortex_m::peripheral::DWT;

// In init
let mut dwt = core.DWT;
dwt.enable_cycle_counter();

// In process
let start = DWT::cycle_count();
// ... do work ...
let cycles = DWT::cycle_count().wrapping_sub(start);
defmt::info!("Cycles: {}", cycles);
```

## Getting Help

If you're still stuck:

1. Check the [bbx_daisy examples](https://github.com/blackboxaudio/bbx_audio/tree/develop/bbx_daisy/examples) for working reference code
2. Search existing [GitHub issues](https://github.com/blackboxaudio/bbx_audio/issues)
3. Open a new issue with:
   - Your board variant (Seed, Pod, etc.)
   - Full error message
   - Minimal code to reproduce
   - Cargo.toml dependencies
