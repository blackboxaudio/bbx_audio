//! Graph topology visualization example.
//!
//! Displays a DSP graph's topology with connected blocks arranged left-to-right.

use bbx_draw::{GraphTopologyVisualizer, Visualizer};
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};
use nannou::prelude::*;

struct Model {
    visualizer: GraphTopologyVisualizer,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .title("bbx_draw - Graph Topology")
        .size(1200, 600)
        .view(view)
        .build()
        .unwrap();

    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

    let osc1 = builder.add_oscillator(440.0, Waveform::Sine, None);
    let osc2 = builder.add_oscillator(220.0, Waveform::Square, None);
    let lfo = builder.add_lfo(2.0, 0.5, None);
    let overdrive = builder.add_overdrive(0.7, 0.8, 0.5, 44100.0);

    builder.connect(osc1, 0, overdrive, 0);
    builder.connect(osc2, 0, overdrive, 0);
    builder.modulate(lfo, osc1, "frequency");

    let topology = builder.capture_topology();

    let visualizer = GraphTopologyVisualizer::new(topology);

    Model { visualizer }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.visualizer.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(rgb(0.12, 0.12, 0.12));

    let win = app.window_rect();
    let bounds = Rect::from_xy_wh(win.xy(), win.wh() * 0.9);

    model.visualizer.draw(&draw, bounds);

    draw.to_frame(app, &frame).unwrap();
}
