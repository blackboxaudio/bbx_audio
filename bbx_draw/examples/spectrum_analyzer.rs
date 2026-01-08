//! Spectrum analyzer visualization example.
//!
//! Displays FFT spectrum of a multi-frequency signal.

use std::{
    f32::consts::PI,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use bbx_draw::{AudioFrame, SpectrumAnalyzer, Visualizer, audio_bridge};
use nannou::prelude::*;

struct Model {
    visualizer: SpectrumAnalyzer,
    _running: Arc<AtomicBool>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .title("bbx_draw - Spectrum Analyzer")
        .size(1000, 500)
        .view(view)
        .build()
        .unwrap();

    let (mut producer, consumer) = audio_bridge(16);
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    thread::spawn(move || {
        let sample_rate = 44100;
        let buffer_size = 512;
        let frequencies = [220.0, 440.0, 880.0, 1760.0];
        let mut phases = [0.0f32; 4];

        while running_clone.load(Ordering::Relaxed) {
            let mut samples = Vec::with_capacity(buffer_size);

            for _ in 0..buffer_size {
                let mut sample = 0.0;
                for (i, &freq) in frequencies.iter().enumerate() {
                    let amplitude = 1.0 / (i as f32 + 1.0);
                    sample += amplitude * (phases[i] * 2.0 * PI).sin();
                    phases[i] += freq / sample_rate as f32;
                    if phases[i] >= 1.0 {
                        phases[i] -= 1.0;
                    }
                }
                samples.push(sample / 2.0);
            }

            let frame = AudioFrame::new(samples, sample_rate, 1);
            let _ = producer.try_send(frame);

            thread::sleep(Duration::from_micros(
                (1_000_000 * buffer_size as u64) / sample_rate as u64,
            ));
        }
    });

    let visualizer = SpectrumAnalyzer::new(consumer);

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
    draw.background().color(rgb(0.05, 0.02, 0.1));

    let win = app.window_rect();
    let bounds = Rect::from_xy_wh(
        pt2(win.x(), win.y() - win.h() * 0.1),
        vec2(win.w() * 0.9, win.h() * 0.7),
    );

    for i in 0..5 {
        let y = bounds.bottom() + (i as f32 / 4.0) * bounds.h();
        draw.line()
            .start(pt2(bounds.left(), y))
            .end(pt2(bounds.right(), y))
            .weight(1.0)
            .color(rgba(1.0, 1.0, 1.0, 0.1));
    }

    model.visualizer.draw(&draw, bounds);

    draw.text("Spectrum Analyzer")
        .xy(pt2(0.0, win.top() - 30.0))
        .color(WHITE)
        .font_size(20);

    draw.to_frame(app, &frame).unwrap();
}
