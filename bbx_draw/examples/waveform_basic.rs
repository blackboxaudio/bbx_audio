//! Basic waveform visualization example.
//!
//! Displays a sine wave oscilloscope using the WaveformVisualizer.

use std::{
    f32::consts::PI,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use bbx_draw::{AudioFrame, Visualizer, WaveformVisualizer, audio_bridge};
use nannou::prelude::*;

struct Model {
    visualizer: WaveformVisualizer,
    _running: Arc<AtomicBool>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .title("bbx_draw - Waveform")
        .size(800, 400)
        .view(view)
        .build()
        .unwrap();

    let (mut producer, consumer) = audio_bridge(16);
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    thread::spawn(move || {
        let sample_rate = 44100;
        let frequency = 440.0;
        let buffer_size = 256;
        let mut phase = 0.0f32;

        while running_clone.load(Ordering::Relaxed) {
            let mut samples = Vec::with_capacity(buffer_size);

            for _ in 0..buffer_size {
                samples.push((phase * 2.0 * PI).sin());
                phase += frequency / sample_rate as f32;
                if phase >= 1.0 {
                    phase -= 1.0;
                }
            }

            let frame = AudioFrame::new(samples, sample_rate, 1);
            let _ = producer.try_send(frame);

            thread::sleep(Duration::from_micros(
                (1_000_000 * buffer_size as u64) / sample_rate as u64,
            ));
        }
    });

    let visualizer = WaveformVisualizer::new(consumer);

    Model {
        visualizer,
        _running: running,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.visualizer.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(rgb(0.05, 0.05, 0.1));

    let win = app.window_rect();
    let bounds = Rect::from_xy_wh(win.xy(), win.wh() * 0.9);

    draw.line()
        .start(pt2(bounds.left(), bounds.y()))
        .end(pt2(bounds.right(), bounds.y()))
        .weight(1.0)
        .color(rgba(1.0, 1.0, 1.0, 0.2));

    model.visualizer.draw(&draw, bounds);

    draw.to_frame(app, &frame).unwrap();
}
