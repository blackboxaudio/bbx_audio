use bbx_draw::*;
use nannou::prelude::*;

const CONTEXT: DisplayContext = ctx!(DisplayContext, 1280.0, 720.0, 256);

pub fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .size(CONTEXT.width as u32, CONTEXT.height as u32)
        .run();
}

struct Model {}

/// Initializes the app state (e.g. window, GUI) and performs startup
/// tasks like loading images or other assets.
fn model(_app: &App) -> Model {
    Model {}
}

/// Updates the state of the model (hence the `&mut`), running at a constant time interval.
fn update(_app: &App, _model: &mut Model, _update: Update) {}

/// Presents the state of the model (hence no `&mut`) to a window via the `Frame` object.
fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();

    draw_chart(ChartConfiguration::new("Time", "Amplitude"), &CONTEXT, &draw);
    draw_data(&draw);

    draw.to_frame(app, &frame).unwrap();
}

fn draw_data(draw: &Draw) {
    let mut sample_data: Vec<f32> = Vec::with_capacity(CONTEXT.buffer_size);
    for n in 0..CONTEXT.buffer_size {
        let phase = (n as f32 / CONTEXT.buffer_size as f32) * 2.0 * PI;
        sample_data.push(f32::sin(phase));
    }

    let mut previous_sample = 0.0;
    for (sample_idx, sample) in sample_data.iter().enumerate() {
        if sample_idx > 0 {
            draw.line()
                .color(BLACK)
                .start(map_sample_data_to_point2(previous_sample, sample_idx - 1, &CONTEXT))
                .end(map_sample_data_to_point2(*sample, sample_idx, &CONTEXT));
        }

        previous_sample = *sample;
    }
}
