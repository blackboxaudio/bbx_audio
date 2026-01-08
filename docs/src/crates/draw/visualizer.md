# Visualizer Trait

The core trait that all visualizers implement, following a two-phase update/draw pattern.

## Trait Definition

```rust
pub trait Visualizer {
    /// Update internal state (called each frame before drawing).
    fn update(&mut self);

    /// Draw the visualization within the given bounds.
    fn draw(&self, draw: &nannou::Draw, bounds: Rect);
}
```

## Methods

### update

```rust
fn update(&mut self);
```

Called once per frame before drawing. Visualizers should:

- Consume data from their SPSC bridges
- Process incoming audio frames or MIDI messages
- Update internal buffers and state

Keep this method efficient as it runs on the visualization thread at frame rate (typically 60 Hz).

### draw

```rust
fn draw(&self, draw: &nannou::Draw, bounds: Rect);
```

Renders the visualization within the given bounds rectangle.

| Parameter | Description |
|-----------|-------------|
| `draw` | nannou's Draw API for rendering |
| `bounds` | Rectangle defining the render area |

The `bounds` parameter enables layout composition. Multiple visualizers can render side-by-side by subdividing the window rectangle.

## Integration with nannou

The trait maps directly to nannou's model-update-view pattern:

```rust
use bbx_draw::{Visualizer, WaveformVisualizer, audio_bridge};

struct Model {
    visualizer: WaveformVisualizer,
}

fn model(app: &App) -> Model {
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
    let bounds = app.window_rect();
    model.visualizer.draw(&draw, bounds);
    draw.to_frame(app, &frame).unwrap();
}
```

## Multiple Visualizers

Arrange visualizers using `Rect` subdivision:

```rust
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let win = app.window_rect();

    // Split window horizontally
    let (left, right) = win.split_left(win.w() * 0.5);

    model.waveform.draw(&draw, left);
    model.spectrum.draw(&draw, right);

    draw.to_frame(app, &frame).unwrap();
}
```

## Implementing a Custom Visualizer

```rust
use bbx_draw::Visualizer;
use nannou::prelude::*;

pub struct LevelMeter {
    level: f32,
    consumer: AudioBridgeConsumer,
}

impl Visualizer for LevelMeter {
    fn update(&mut self) {
        while let Some(frame) = self.consumer.try_pop() {
            let peak = frame.samples.iter()
                .map(|s| s.abs())
                .fold(0.0f32, f32::max);
            self.level = self.level.max(peak) * 0.95; // decay
        }
    }

    fn draw(&self, draw: &Draw, bounds: Rect) {
        let height = bounds.h() * self.level;
        draw.rect()
            .xy(bounds.mid_bottom() + vec2(0.0, height * 0.5))
            .w_h(bounds.w(), height)
            .color(GREEN);
    }
}
```

## Real-Time Safety

The `update()` method runs on the visualization thread, not the audio thread. However, follow these guidelines:

- Drain all available data from bridges each frame
- Avoid allocations in tight loops
- Use fixed-size buffers where possible
- Keep processing lightweight (60 Hz budget)
