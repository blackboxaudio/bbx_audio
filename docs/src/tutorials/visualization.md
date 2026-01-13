# Real-Time Visualization

This tutorial shows how to visualize audio in real-time using bbx_draw with nannou.

## Prerequisites

Add bbx_draw and nannou to your project:

```toml
[dependencies]
bbx_draw = "0.1"
bbx_dsp = "0.1"
nannou = "0.19"
```

> **Prior knowledge**: This tutorial assumes familiarity with:
> - [Your First DSP Graph](first-graph.md) - Building and processing graphs
> - [nannou](https://nannou.cc) - Basic nannou application structure

## Setting Up the Audio Bridge

The audio bridge connects your audio processing to the visualization thread:

```rust
use bbx_draw::{audio_bridge, AudioBridgeProducer, AudioBridgeConsumer};

// Create a bridge with capacity for 16 audio frames
let (producer, consumer) = audio_bridge(16);
```

The producer sends frames from the audio thread, the consumer receives them in the visualization thread.

## Your First Waveform Visualizer

Let's create a basic waveform display:

```rust
use bbx_draw::{audio_bridge, WaveformVisualizer, Visualizer};
use nannou::prelude::*;

struct Model {
    visualizer: WaveformVisualizer,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();

    let (_producer, consumer) = audio_bridge(16);

    Model {
        visualizer: WaveformVisualizer::new(consumer),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.visualizer.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    model.visualizer.draw(&draw, app.window_rect());
    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    nannou::app(model).update(update).run();
}
```

## Sending Audio to the Visualizer

In your audio processing code, send frames to the producer:

```rust
use bbx_draw::AudioFrame;
use bbx_dsp::Frame;

fn audio_callback(producer: &mut AudioBridgeProducer, samples: &[f32]) {
    let frame = Frame::new(samples, 44100, 2);
    producer.try_send(frame);  // Non-blocking
}
```

`try_send()` never blocks. If the buffer is full, frames are dropped, which is acceptable for visualization.

## Adding a Spectrum Analyzer

Display frequency content alongside the waveform:

```rust
use bbx_draw::{
    audio_bridge, WaveformVisualizer, SpectrumAnalyzer, Visualizer,
    config::{SpectrumConfig, SpectrumDisplayMode},
};
use nannou::prelude::*;

struct Model {
    waveform: WaveformVisualizer,
    spectrum: SpectrumAnalyzer,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();

    // Share data between visualizers using separate bridges
    let (_prod1, cons1) = audio_bridge(16);
    let (_prod2, cons2) = audio_bridge(16);

    let spectrum_config = SpectrumConfig {
        display_mode: SpectrumDisplayMode::Filled,
        ..Default::default()
    };

    Model {
        waveform: WaveformVisualizer::new(cons1),
        spectrum: SpectrumAnalyzer::with_config(cons2, spectrum_config),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.waveform.update();
    model.spectrum.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let win = app.window_rect();
    let (top, bottom) = win.split_top(win.h() * 0.5);

    model.waveform.draw(&draw, top);
    model.spectrum.draw(&draw, bottom);

    draw.to_frame(app, &frame).unwrap();
}
```

## Visualizing a DSP Graph

Display the structure of your DSP graph:

```rust
use bbx_draw::{GraphTopologyVisualizer, Visualizer};
use bbx_dsp::{blocks::{GainBlock, OscillatorBlock}, graph::GraphBuilder, waveform::Waveform};
use nannou::prelude::*;

struct Model {
    visualizer: GraphTopologyVisualizer,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();

    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
    let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(-6.0, None));
    builder.connect(osc, 0, gain, 0);

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
    draw.background().color(BLACK);
    model.visualizer.draw(&draw, app.window_rect());
    draw.to_frame(app, &frame).unwrap();
}
```

## Complete Example with Audio

Here's a full example with audio generation and visualization:

```rust
use bbx_draw::{audio_bridge, AudioBridgeProducer, WaveformVisualizer, Visualizer};
use bbx_dsp::{blocks::OscillatorBlock, graph::GraphBuilder, waveform::Waveform, Frame};
use nannou::prelude::*;
use std::sync::{Arc, Mutex};

struct Model {
    visualizer: WaveformVisualizer,
    producer: Arc<Mutex<AudioBridgeProducer>>,
    graph: bbx_dsp::graph::Graph<f32>,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();

    let (producer, consumer) = audio_bridge(16);

    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
    builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let graph = builder.build();

    Model {
        visualizer: WaveformVisualizer::new(consumer),
        producer: Arc::new(Mutex::new(producer)),
        graph,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Generate audio
    let mut left = vec![0.0f32; 512];
    let mut right = vec![0.0f32; 512];
    let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
    model.graph.process_buffers(&mut outputs);

    // Send to visualizer
    let frame = Frame::new(&left, 44100, 1);
    if let Ok(mut producer) = model.producer.lock() {
        producer.try_send(frame);
    }

    model.visualizer.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    model.visualizer.draw(&draw, app.window_rect());
    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    nannou::app(model).update(update).run();
}
```

## Next Steps

- [Visualizer Trait](../crates/draw/visualizer.md) - Create custom visualizers
- [Audio Bridge](../crates/draw/audio-bridge.md) - Bridge configuration details
- [Sketch Discovery](sketchbook.md) - Manage multiple sketches
- [Visualization Threading](../architecture/visualization-threading.md) - Threading model details
