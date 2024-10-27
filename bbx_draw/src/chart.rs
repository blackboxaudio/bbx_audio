use nannou::{
    color::{ANTIQUEWHITE, BLACK},
    geom::Point2,
    Draw,
};

use crate::context::DisplayContext;

/// The configuration of a coordinate plane chart.
pub struct ChartConfiguration {
    x_axis_label: String,
    y_axis_label: String,
}

impl ChartConfiguration {
    pub fn new(x_axis_label: &str, y_axis_label: &str) -> ChartConfiguration {
        ChartConfiguration {
            x_axis_label: x_axis_label.to_string(),
            y_axis_label: y_axis_label.to_string(),
        }
    }
}

impl Default for ChartConfiguration {
    fn default() -> Self {
        ChartConfiguration {
            x_axis_label: "".to_string(),
            y_axis_label: "".to_string(),
        }
    }
}

/// Draws a chart with x- and y-axes, labels, etc.
pub fn draw_chart(chart_configuration: ChartConfiguration, display_context: &DisplayContext, draw: &Draw) {
    draw.background().color(ANTIQUEWHITE);

    let x1 = display_context.domain.0 + display_context.padding.0;
    let x2 = display_context.domain.1 - display_context.padding.0;
    let x_axis = (Point2::new(x1, 0.0), Point2::new(x2, 0.0));
    draw.line().color(BLACK).start(x_axis.0).end(x_axis.1);
    draw.text(chart_configuration.x_axis_label.as_str())
        .color(BLACK)
        .xy(Point2::new(x2 + (0.5 * display_context.padding.0), 0.0));

    let y1 = display_context.range.0 + display_context.padding.1;
    let y2 = display_context.range.1 - display_context.padding.1;
    let y_axis = (Point2::new(x1, y1), Point2::new(x1, y2));
    draw.line().color(BLACK).start(y_axis.0).end(y_axis.1);
    draw.text(chart_configuration.y_axis_label.as_str())
        .color(BLACK)
        .xy(Point2::new(x1, y2 + (0.5 * display_context.padding.1)));
}
