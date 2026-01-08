# bbx_draw

Audio visualization primitives for nannou sketches.

## Overview

`bbx_draw` provides embeddable visualizers for audio and DSP applications, with lock-free communication between audio and visualization threads via `bbx_core::SpscRingBuffer`.

## Visualizers

- **GraphTopologyVisualizer** - Displays DSP graph topology with blocks arranged left-to-right by depth
- **WaveformVisualizer** - Oscilloscope-style waveform display with zero-crossing trigger
- **SpectrumAnalyzer** - FFT-based spectrum display (bars, line, or filled modes)
- **MidiActivityVisualizer** - Piano-roll style MIDI note activity with velocity-based brightness

## Quick Start

```rust
use bbx_draw::{GraphTopologyVisualizer, Visualizer};
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};
use nannou::prelude::*;

fn model(app: &App) -> Model {
    // Build a DSP graph
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
    builder.add_oscillator(440.0, Waveform::Sine, None);

    // Capture topology snapshot
    let topology = builder.capture_topology();

    // Create visualizer
    Model {
        visualizer: GraphTopologyVisualizer::new(topology),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.visualizer.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let bounds = app.window_rect();
    model.visualizer.draw(&draw, bounds);
    draw.to_frame(app, &frame).unwrap();
}
```

## Audio Bridge

For real-time audio visualization, use the lock-free audio bridge:

```rust
use bbx_draw::{audio_bridge, AudioFrame, WaveformVisualizer};

// Create bridge pair
let (mut producer, consumer) = audio_bridge(16);

// In audio thread: send frames
let frame = AudioFrame::new(samples, 44100, 2);
producer.try_send(frame);

// In viz thread: create visualizer with consumer
let visualizer = WaveformVisualizer::new(consumer);
```

## Examples

```bash
# Graph topology visualization
cargo run --example graph_view -p bbx_draw

# Waveform oscilloscope
cargo run --example waveform_basic -p bbx_draw

# Spectrum analyzer
cargo run --example spectrum_analyzer -p bbx_draw
```

## Features

- `sketch-registry` (default) - Enables sketch discovery and caching via `SketchRegistry`

## Threading Model

```
┌─────────────────────┐         SPSC Ring Buffer        ┌─────────────────────┐
│    Audio Thread     │ ────────────────────────────▶   │  nannou Thread      │
│  (rodio callback)   │     AudioFrame packets          │  (model/update/view)│
│                     │                                 │                     │
│  try_send()         │                                 │  drain()            │
│  (non-blocking)     │                                 │  (consume all)      │
└─────────────────────┘                                 └─────────────────────┘
```

Audio thread uses non-blocking `try_send()` - frames are dropped if buffer is full (acceptable for visualization purposes).
