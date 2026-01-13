# bbx_draw

Audio visualization primitives for nannou sketches.

## Overview

bbx_draw provides embeddable visualizers for audio and DSP applications:

- Real-time visualization with lock-free communication
- Four built-in visualizers for common use cases
- Configurable appearance and behavior
- Compatible with nannou's model-update-view architecture

## Installation

```toml
[dependencies]
bbx_draw = "0.1"
```

## Features

| Feature | Description |
|---------|-------------|
| [Visualizer Trait](draw/visualizer.md) | Core trait for all visualizers |
| [Audio Bridge](draw/audio-bridge.md) | Lock-free thread communication |
| [Graph Topology](draw/graph-topology.md) | DSP graph layout display |
| [Waveform](draw/waveform.md) | Oscilloscope-style waveform |
| [Spectrum](draw/spectrum.md) | FFT-based spectrum analyzer |
| [MIDI Activity](draw/midi-activity.md) | Piano keyboard note display |

## Quick Example

```rust
use bbx_draw::{GraphTopologyVisualizer, Visualizer};
use bbx_dsp::{blocks::OscillatorBlock, graph::GraphBuilder, waveform::Waveform};
use nannou::prelude::*;

fn model(app: &App) -> Model {
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
    builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let topology = builder.capture_topology();

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

## Visualizers

### GraphTopologyVisualizer

Displays DSP graph structure with blocks arranged by topological depth. Color-codes blocks by category (generator, effector, modulator, I/O) and shows audio and modulation connections.

### WaveformVisualizer

Oscilloscope-style waveform display with zero-crossing trigger detection for stable display. Connects to audio via `AudioBridgeConsumer`.

### SpectrumAnalyzer

FFT-based frequency spectrum display with three modes (bars, line, filled). Supports temporal smoothing and peak hold with configurable decay.

### MidiActivityVisualizer

Piano keyboard display showing MIDI note activity. Velocity-based brightness and configurable decay animation after note-off.

## Threading Model

```
┌─────────────────────┐         SPSC Ring Buffer        ┌─────────────────────┐
│    Audio Thread     │ ────────────────────────────▶   │  nannou Thread      │
│  (rodio callback)   │     AudioFrame packets          │  (model/update/view)│
│                     │                                 │                     │
│  try_send()         │                                 │  try_pop()          │
│  (non-blocking)     │                                 │  (consume all)      │
└─────────────────────┘                                 └─────────────────────┘
```

The audio thread uses non-blocking `try_send()`. Frames are dropped if the buffer is full, which is acceptable for visualization purposes.

See [Visualization Threading](../architecture/visualization-threading.md) for details.
