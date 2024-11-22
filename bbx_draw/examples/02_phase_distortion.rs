use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};
use bbx_draw::*;
use bbx_dsp::phasor::Phasor;

const CONTEXT: DisplayContext = ctx!(DisplayContext, 1280.0, 720.0, 256);

pub fn main() {
    nannou::app(model)
        .update(update)
        .size(CONTEXT.width as u32, CONTEXT.height as u32)
        .run();
}

#[derive(Debug)]
struct SliderState {
    resolution: f32,
}

struct Model {
    egui: Egui,
    phasor: Phasor,
    slider_state: SliderState,
}

/// Initializes the app state (e.g. window, GUI) and performs startup
/// tasks like loading images or other assets.
fn model(app: &App) -> Model {
    let window_id = app.new_window().view(view).raw_event(raw_window_event).build().unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    Model {
        egui,
        phasor: Phasor::new(),
        slider_state: SliderState {
            resolution: 0.5,
        },
    }
}

/// Updates the state of the model (hence the `&mut`), running at a constant time interval.
fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let phasor = &mut model.phasor;
    let state = &mut model.slider_state;
    phasor.set_pivot(state.resolution, 0.5);

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label("Inflection:");
        ui.add(egui::Slider::new(&mut state.resolution, 0.01..=0.99).fixed_decimals(2).step_by(0.01))
    });
}

/// Presents the state of the model (hence no `&mut`) to a window via the `Frame` object.
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw_chart(ChartConfiguration::new("Time", "Amplitude"), &CONTEXT, &draw);
    draw_phasor_lines(&draw, model);
    draw_sample_data(&draw, model);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn draw_phasor_lines(draw: &Draw, model: &Model) {
    let (ix, iy) = model.phasor.get_inflection();
    draw.line().color(CORNFLOWERBLUE).start(map_normalized_point_to_display_point(Point2::new(0.0, 0.0), &CONTEXT)).end(map_normalized_point_to_display_point(Point2::new(ix, iy), &CONTEXT));
    draw.line().color(CORNFLOWERBLUE).start(map_normalized_point_to_display_point(Point2::new(ix, iy), &CONTEXT)).end(map_normalized_point_to_display_point(Point2::new(1.0, 1.0), &CONTEXT));
}

fn draw_sample_data(draw: &Draw, model: &Model) {
    let phasor = &model.phasor;
    let mut phasor_sample_idx: usize = 0;
    let mut phasor_sample_marked: bool = false;
    let inflection = phasor.get_inflection();

    let mut sample_data: Vec<f32> = Vec::with_capacity(CONTEXT.buffer_size);
    for n in 0..CONTEXT.buffer_size {
        let normalized_phase = n as f32 / CONTEXT.buffer_size as f32;
        if normalized_phase > inflection.0 && !phasor_sample_marked {
            phasor_sample_idx = n;
            phasor_sample_marked = true;
        }

        let phase = phasor.apply(normalized_phase) * 2.0 * PI;
        sample_data.push(f32::cos(phase));
    }

    let mut previous_sample = 0.0;
    for (sample_idx, sample) in sample_data.iter().enumerate() {
        if sample_idx > 0 {
            draw.line()
                .color(BLACK)
                .start(map_sample_data_to_display_point(previous_sample, sample_idx - 1, &CONTEXT))
                .end(map_sample_data_to_display_point(*sample, sample_idx, &CONTEXT));
        }

        if sample_idx == phasor_sample_idx {
            draw.line().color(MEDIUMPURPLE)
                .start(map_sample_data_to_display_point(*sample, sample_idx, &CONTEXT))
                .end(map_normalized_point_to_display_point(Point2::new(inflection.0, inflection.1), &CONTEXT));
        }

        previous_sample = *sample;
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}
