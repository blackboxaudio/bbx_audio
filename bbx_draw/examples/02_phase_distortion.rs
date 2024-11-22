use rand::Rng;
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
    slider_groups: Vec<(SliderState, SliderState)>,
}

/// Initializes the app state (e.g. window, GUI) and performs startup
/// tasks like loading images or other assets.
fn model(app: &App) -> Model {
    let window_id = app.new_window().view(view).raw_event(raw_window_event).build().unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    let mut phasor = Phasor::new();
    let inflections: Vec<(f32, f32)> = vec![
        // Add pre-determined inflections here
    ];
    for inflection in &inflections {
        phasor.add_inflection(inflection.0, inflection.1);
    }
    let slider_groups = inflections.iter().map(|inflection| (SliderState { resolution: inflection.0 }, SliderState { resolution: inflection.1 })).collect();

    Model {
        egui,
        phasor,
        slider_groups,
    }
}

/// Updates the state of the model (hence the `&mut`), running at a constant time interval.
fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let phasor = &mut model.phasor;

    let slider_groups = &mut model.slider_groups;
    for (group_idx, group) in slider_groups.iter().enumerate() {
        let (x_slider, y_slider) = group;
        phasor.set_inflection(group_idx + 1, x_slider.resolution, y_slider.resolution);
    }

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        let clicked = ui.button("Add inflection").clicked();
        if clicked {
            let mut rng = rand::thread_rng();
            let new_x = rng.gen::<f32>();
            phasor.add_inflection(new_x, new_x);
            slider_groups.push((SliderState { resolution: new_x }, SliderState { resolution: new_x }));
        }

        for (group_idx, group) in slider_groups.iter_mut().enumerate() {
            let (x_slider, y_slider) = group;
            ui.label(format!("Inflection {}:", group_idx + 1));
            ui.add(egui::Slider::new(&mut x_slider.resolution, 0.001..=0.999).fixed_decimals(3).step_by(0.001));
            ui.add(egui::Slider::new(&mut y_slider.resolution, 0.001..=0.999).fixed_decimals(3).step_by(0.001));
        }
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
    for idx in 0..model.phasor.get_inflections().len() - 1 {
        let i1 = *model.phasor.get_inflection(idx);
        let i2 = *model.phasor.get_inflection(idx + 1);
        draw.line().color(CORNFLOWERBLUE).start(map_normalized_point_to_display_point(Point2::from(i1), &CONTEXT)).end(map_normalized_point_to_display_point(Point2::from(i2), &CONTEXT));
    }
}

fn draw_sample_data(draw: &Draw, model: &Model) {
    let phasor = &model.phasor;

    let mut sample_data: Vec<f32> = Vec::with_capacity(CONTEXT.buffer_size);
    let mut inflection_indices: Vec<usize> = Vec::new();
    let mut inflection_count: usize = 1;

    for n in 0..CONTEXT.buffer_size {
        let normalized_phase = n as f32 / CONTEXT.buffer_size as f32;
        let inflection = phasor.get_inflection(inflection_count);
        if normalized_phase > inflection.0 {
            inflection_count += 1;
            inflection_indices.push(n - 1);
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

        if sample_idx == sample_data.len() - 1 {
            draw.line()
                .color(BLACK)
                .start(map_sample_data_to_display_point(*sample, sample_idx, &CONTEXT))
                .end(map_sample_data_to_display_point(1.0, sample_idx + 1, &CONTEXT));
        }

        previous_sample = *sample;
    }

    for (inflection_idx, sample_idx) in inflection_indices.iter().enumerate() {
        let sample = sample_data[*sample_idx];
        let inflection = phasor.get_inflection(inflection_idx + 1);
        draw.line().color(MEDIUMPURPLE).start(map_sample_data_to_display_point(sample, *sample_idx, &CONTEXT)).end(map_normalized_point_to_display_point(Point2::from(*inflection), &CONTEXT));
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}
