use nannou::geom::Point2;

use crate::context::DisplayContext;

pub fn map_normalized_point_to_display_point(point: Point2, display_context: &DisplayContext) -> Point2 {
    let x_axis_range = (
        display_context.domain.0 + display_context.padding.0,
        display_context.domain.1 - display_context.padding.0,
    );
    let range_midpoint = display_context.range.0 + (display_context.range.1 - display_context.range.0) / 2.0;
    let y_axis_range = (range_midpoint, display_context.range.1 - display_context.padding.1);
    Point2::new(
        scale_number(point.x, 0.0, 1.0, x_axis_range.0, x_axis_range.1),
        scale_number(point.y, 0.0, 1.0, y_axis_range.0, y_axis_range.1),
    )
}

/// Create a 2D point from a sample's value and index.
pub fn map_sample_data_to_display_point(
    sample_value: f32,
    sample_index: usize,
    display_context: &DisplayContext,
) -> Point2 {
    let x_axis_range = (
        display_context.domain.0 + display_context.padding.0,
        display_context.domain.1 - display_context.padding.0,
    );
    let y_axis_range = (
        display_context.range.0 + display_context.padding.1,
        display_context.range.1 - display_context.padding.1,
    );
    Point2::new(
        scale_number(
            sample_index as f32,
            0.0,
            display_context.buffer_size as f32,
            x_axis_range.0,
            x_axis_range.1,
        ),
        scale_number(sample_value, -1.0, 1.0, y_axis_range.0, y_axis_range.1),
    )
}

/// Scale a number from one range to another.
pub fn scale_number(n: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let proportion: f32 = (n - in_min) / (in_max - in_min);
    out_min + proportion * (out_max - out_min)
}
