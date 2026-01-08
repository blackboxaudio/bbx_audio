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
        const BUFFER_SIZE: usize = 512;
        let frequencies = [220.0, 440.0, 880.0, 1760.0];
        let mut phases = [0.0f32; 4];
        let mut samples = [0.0f32; BUFFER_SIZE];

        while running_clone.load(Ordering::Relaxed) {
            for i in 0..BUFFER_SIZE {
                let mut sample = 0.0;
                for (j, &freq) in frequencies.iter().enumerate() {
                    let amplitude = 1.0 / (j as f32 + 1.0);
                    sample += amplitude * (phases[j] * 2.0 * PI).sin();
                    phases[j] += freq / sample_rate as f32;
                    if phases[j] >= 1.0 {
                        phases[j] -= 1.0;
                    }
                }
                samples[i] = sample / 2.0;
            }

            let frame = AudioFrame::new(&samples, sample_rate, 1);
            let _ = producer.try_send(frame);

            thread::sleep(Duration::from_micros(
                (1_000_000 * BUFFER_SIZE as u64) / sample_rate as u64,
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
