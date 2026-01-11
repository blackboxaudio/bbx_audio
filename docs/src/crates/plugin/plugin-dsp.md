# PluginDsp Trait

The `PluginDsp` trait defines the interface for plugin DSP implementations.

## Definition

```rust
pub trait PluginDsp: Default + Send + 'static {
    fn new() -> Self;
    fn prepare(&mut self, context: &DspContext);
    fn reset(&mut self);
    fn apply_parameters(&mut self, params: &[f32]);
    fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]],
               midi_events: &[MidiEvent], context: &DspContext);

    // Optional MIDI callbacks with sample-accurate timing (default no-ops)
    fn note_on(&mut self, note: u8, velocity: u8, sample_offset: u32) {}
    fn note_off(&mut self, note: u8, sample_offset: u32) {}
    fn control_change(&mut self, cc: u8, value: u8, sample_offset: u32) {}
    fn pitch_bend(&mut self, value: i16, sample_offset: u32) {}
}
```

The `sample_offset` parameter in MIDI callbacks indicates the sample position within the current buffer where the event occurs, enabling sample-accurate MIDI timing.

## Trait Bounds

- **`Default`** - Required for FFI instantiation
- **`Send`** - Safe to transfer to audio thread
- **`'static`** - No borrowed references

## Methods

### new

Create a new instance with default state:

```rust
fn new() -> Self {
    Self {
        gain: 0.0,
        filter_state: 0.0,
    }
}
```

### prepare

Called when audio specs change:

```rust
fn prepare(&mut self, context: &DspContext) {
    // Update sample-rate dependent calculations
    self.filter_coefficient = calculate_coefficient(
        self.cutoff_hz,
        context.sample_rate,
    );
}
```

### reset

Clear all DSP state:

```rust
fn reset(&mut self) {
    self.filter_state = 0.0;
    self.delay_buffer.fill(0.0);
}
```

### apply_parameters

Map parameter array to DSP state:

```rust
fn apply_parameters(&mut self, params: &[f32]) {
    if let Some(&gain) = params.get(0) {
        self.gain = gain;
    }
    if let Some(&pan) = params.get(1) {
        self.pan = pan;
    }
}
```

### process

Process audio buffers with MIDI events:

```rust
fn process(
    &mut self,
    inputs: &[&[f32]],
    outputs: &mut [&mut [f32]],
    midi_events: &[MidiEvent],
    context: &DspContext,
) {
    // Handle MIDI for synthesizers
    for event in midi_events {
        match event.message.get_status() {
            MidiMessageStatus::NoteOn => self.note_on(event.message.get_note().unwrap(), event.message.get_velocity().unwrap()),
            MidiMessageStatus::NoteOff => self.note_off(event.message.get_note().unwrap()),
            _ => {}
        }
    }

    // Process audio
    for ch in 0..inputs.len().min(outputs.len()) {
        for i in 0..context.buffer_size {
            outputs[ch][i] = inputs[ch][i] * self.gain;
        }
    }
}
```

## Implementation Requirements

### Default Implementation

You must implement `Default`:

```rust
impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}
```

### No Allocations in process()

The `process` method runs on the audio thread. Avoid:

- Memory allocation (`Vec::new()`, `Box::new()`)
- Locking (`Mutex::lock()`)
- I/O operations

### Thread Safety

The plugin may be moved between threads. Use:

- `AtomicF32` for cross-thread values
- SPSC queues for message passing
- No thread-local storage

## Complete Example

```rust
use bbx_plugin::{PluginDsp, DspContext, bbx_plugin_ffi};
use bbx_midi::MidiEvent;

const PARAM_GAIN: usize = 0;
const PARAM_PAN: usize = 1;

pub struct StereoGain {
    gain: f32,
    pan: f32,
}

impl Default for StereoGain {
    fn default() -> Self { Self::new() }
}

impl PluginDsp for StereoGain {
    fn new() -> Self {
        Self { gain: 1.0, pan: 0.0 }
    }

    fn prepare(&mut self, _context: &DspContext) {
        // Nothing sample-rate dependent
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn apply_parameters(&mut self, params: &[f32]) {
        self.gain = 10.0_f32.powf(params.get(PARAM_GAIN).copied().unwrap_or(0.0) / 20.0);
        self.pan = params.get(PARAM_PAN).copied().unwrap_or(0.0);
    }

    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _midi_events: &[MidiEvent],
        context: &DspContext,
    ) {
        let pan_rad = self.pan * f32::FRAC_PI_4;
        let left_gain = self.gain * (f32::FRAC_PI_4 - pan_rad).cos();
        let right_gain = self.gain * (f32::FRAC_PI_4 + pan_rad).cos();

        if inputs.len() >= 2 && outputs.len() >= 2 {
            for i in 0..context.buffer_size {
                outputs[0][i] = inputs[0][i] * left_gain;
                outputs[1][i] = inputs[1][i] * right_gain;
            }
        }
    }
}

bbx_plugin_ffi!(StereoGain);
```
