# Graph Topology Visualizer

Displays DSP graph structure with blocks arranged by topological depth.

## Overview

`GraphTopologyVisualizer` renders a static snapshot of a DSP graph, showing:

- Blocks colored by category (generator, effector, modulator, I/O)
- Audio connections as solid bezier curves
- Modulation connections as dashed lines with parameter labels
- Block names as text labels

## Creating a Visualizer

### With Default Configuration

```rust
use bbx_draw::GraphTopologyVisualizer;
use bbx_dsp::graph::GraphBuilder;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
builder.add_oscillator(440.0, Waveform::Sine, None);

let topology = builder.capture_topology();
let visualizer = GraphTopologyVisualizer::new(topology);
```

### With Custom Configuration

```rust
use bbx_draw::{GraphTopologyVisualizer, config::GraphTopologyConfig};

let config = GraphTopologyConfig {
    block_width: 150.0,
    block_height: 60.0,
    show_arrows: false,
    ..Default::default()
};

let visualizer = GraphTopologyVisualizer::with_config(topology, config);
```

## Configuration Options

### GraphTopologyConfig

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `block_width` | `f32` | `120.0` | Width of block rectangles |
| `block_height` | `f32` | `50.0` | Height of block rectangles |
| `horizontal_spacing` | `f32` | `80.0` | Space between depth columns |
| `vertical_spacing` | `f32` | `30.0` | Space between blocks in column |

### Colors

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `generator_color` | `Rgb` | Blue | Generator block fill |
| `effector_color` | `Rgb` | Green | Effector block fill |
| `modulator_color` | `Rgb` | Purple | Modulator block fill |
| `io_color` | `Rgb` | Orange | I/O block fill |
| `audio_connection_color` | `Rgb` | Gray | Audio connection lines |
| `modulation_connection_color` | `Rgb` | Pink | Modulation connection lines |
| `text_color` | `Rgb` | White | Block label text |

### Connections

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `audio_connection_weight` | `f32` | `2.0` | Audio line thickness |
| `modulation_connection_weight` | `f32` | `1.5` | Modulation line thickness |
| `show_arrows` | `bool` | `true` | Show directional arrows |
| `arrow_size` | `f32` | `8.0` | Arrow head size |
| `dash_length` | `f32` | `8.0` | Modulation dash length |
| `dash_gap` | `f32` | `4.0` | Gap between dashes |

## Layout Algorithm

Blocks are positioned using topological depth:

1. Source blocks (no inputs) have depth 0
2. Each block's depth is `max(input depths) + 1`
3. Blocks are arranged left-to-right by depth
4. Blocks at the same depth are stacked vertically

This ensures signal flow reads left-to-right.

## Updating Topology

For dynamic graphs, update the topology at runtime:

```rust
// After modifying the graph
let new_topology = builder.capture_topology();
visualizer.set_topology(new_topology);
```

## Example

```rust
use bbx_draw::{GraphTopologyVisualizer, Visualizer};
use bbx_dsp::{block::BlockType, blocks::GainBlock, graph::GraphBuilder, waveform::Waveform};
use nannou::prelude::*;

struct Model {
    visualizer: GraphTopologyVisualizer,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();

    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));
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

fn main() {
    nannou::app(model).update(update).run();
}
```
