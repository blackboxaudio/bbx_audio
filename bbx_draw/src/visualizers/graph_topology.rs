//! DSP graph topology visualizer.

use bbx_dsp::{
    BlockCategory,
    graph::{BlockSnapshot, ConnectionSnapshot, GraphTopologySnapshot, ModulationConnectionSnapshot},
};
use nannou::{
    Draw,
    geom::{Point2, Rect, Vec2},
};

use crate::{Visualizer, config::GraphTopologyConfig};

/// Visualizes a DSP graph's topology as connected blocks.
///
/// Blocks are arranged left-to-right based on their topological depth
/// (distance from source blocks).
pub struct GraphTopologyVisualizer {
    topology: GraphTopologySnapshot,
    config: GraphTopologyConfig,
    block_positions: Vec<Point2>,
    depths: Vec<usize>,
}

impl GraphTopologyVisualizer {
    /// Create a new graph topology visualizer with default configuration.
    pub fn new(topology: GraphTopologySnapshot) -> Self {
        let mut visualizer = Self {
            topology,
            config: GraphTopologyConfig::default(),
            block_positions: Vec::new(),
            depths: Vec::new(),
        };
        visualizer.compute_layout();
        visualizer
    }

    /// Create a new graph topology visualizer with custom configuration.
    pub fn with_config(topology: GraphTopologySnapshot, config: GraphTopologyConfig) -> Self {
        let mut visualizer = Self {
            topology,
            config,
            block_positions: Vec::new(),
            depths: Vec::new(),
        };
        visualizer.compute_layout();
        visualizer
    }

    /// Get the current topology snapshot.
    pub fn topology(&self) -> &GraphTopologySnapshot {
        &self.topology
    }

    /// Update the topology (recomputes layout).
    pub fn set_topology(&mut self, topology: GraphTopologySnapshot) {
        self.topology = topology;
        self.compute_layout();
    }

    fn compute_layout(&mut self) {
        let num_blocks = self.topology.blocks.len();
        if num_blocks == 0 {
            self.block_positions.clear();
            self.depths.clear();
            return;
        }

        self.depths = self.compute_depths();
        let max_depth = *self.depths.iter().max().unwrap_or(&0);

        let mut blocks_at_depth: Vec<Vec<usize>> = vec![Vec::new(); max_depth + 1];
        for (block_idx, &depth) in self.depths.iter().enumerate() {
            blocks_at_depth[depth].push(block_idx);
        }

        self.block_positions = vec![Point2::ZERO; num_blocks];

        for (depth, block_indices) in blocks_at_depth.iter().enumerate() {
            let num_at_depth = block_indices.len();
            let x = depth as f32 * (self.config.block_width + self.config.horizontal_spacing);

            for (row, &block_idx) in block_indices.iter().enumerate() {
                let y_offset = (num_at_depth as f32 - 1.0) / 2.0;
                let y = (row as f32 - y_offset) * (self.config.block_height + self.config.vertical_spacing);
                self.block_positions[block_idx] = Point2::new(x, y);
            }
        }
    }

    fn compute_depths(&self) -> Vec<usize> {
        let num_blocks = self.topology.blocks.len();
        let mut depths = vec![0usize; num_blocks];
        let mut changed = true;

        while changed {
            changed = false;
            for conn in &self.topology.connections {
                let new_depth = depths[conn.from_block] + 1;
                if new_depth > depths[conn.to_block] {
                    depths[conn.to_block] = new_depth;
                    changed = true;
                }
            }
        }

        depths
    }

    fn color_for_category(&self, category: BlockCategory) -> nannou::color::Rgb {
        match category {
            BlockCategory::Generator => self.config.generator_color,
            BlockCategory::Effector => self.config.effector_color,
            BlockCategory::Modulator => self.config.modulator_color,
            BlockCategory::IO => self.config.io_color,
        }
    }

    fn draw_block(&self, draw: &Draw, block: &BlockSnapshot, position: Point2, bounds: Rect) {
        let offset_x = bounds.left() + bounds.w() / 2.0;
        let offset_y = bounds.bottom() + bounds.h() / 2.0;
        let adjusted_pos = Point2::new(position.x + offset_x, position.y + offset_y);

        let block_rect = Rect::from_xy_wh(adjusted_pos, [self.config.block_width, self.config.block_height].into());

        if bounds.overlap(block_rect).is_none() {
            return;
        }

        let color = self.color_for_category(block.category);

        draw.rect()
            .xy(adjusted_pos)
            .w_h(self.config.block_width, self.config.block_height)
            .color(color);

        draw.text(&block.name)
            .xy(adjusted_pos)
            .color(self.config.text_color)
            .font_size(12);
    }

    fn draw_audio_connection(&self, draw: &Draw, conn: &ConnectionSnapshot, bounds: Rect) {
        let from_pos = self.block_positions.get(conn.from_block);
        let to_pos = self.block_positions.get(conn.to_block);

        if let (Some(&from), Some(&to)) = (from_pos, to_pos) {
            let offset_x = bounds.left() + bounds.w() / 2.0;
            let offset_y = bounds.bottom() + bounds.h() / 2.0;

            let start = Point2::new(from.x + offset_x + self.config.block_width / 2.0, from.y + offset_y);
            let end = Point2::new(to.x + offset_x - self.config.block_width / 2.0, to.y + offset_y);

            let control_offset = (end.x - start.x) * 0.4;
            let control1 = Point2::new(start.x + control_offset, start.y);
            let control2 = Point2::new(end.x - control_offset, end.y);

            let points = bezier_points(start, control1, control2, end, 20);

            draw.path()
                .stroke()
                .weight(self.config.audio_connection_weight)
                .color(self.config.audio_connection_color)
                .points(points.clone());

            if self.config.show_arrows && points.len() >= 2 {
                let arrow_end = points[points.len() - 1];
                let arrow_prev = points[points.len() - 2];
                draw_arrow_head(
                    draw,
                    arrow_prev,
                    arrow_end,
                    self.config.arrow_size,
                    self.config.audio_connection_color,
                );
            }
        }
    }

    fn draw_modulation_connection(&self, draw: &Draw, conn: &ModulationConnectionSnapshot, bounds: Rect) {
        let from_pos = self.block_positions.get(conn.from_block);
        let to_pos = self.block_positions.get(conn.to_block);

        if let (Some(&from), Some(&to)) = (from_pos, to_pos) {
            let offset_x = bounds.left() + bounds.w() / 2.0;
            let offset_y = bounds.bottom() + bounds.h() / 2.0;

            let from_center = Point2::new(from.x + offset_x, from.y + offset_y);
            let to_center = Point2::new(to.x + offset_x, to.y + offset_y);

            let (start, end, control1, control2, label_pos) = if (from.x - to.x).abs() < 1.0 {
                let start = Point2::new(
                    from_center.x - self.config.block_width / 2.0,
                    from_center.y - self.config.block_height / 2.0,
                );
                let end = Point2::new(
                    to_center.x - self.config.block_width / 2.0,
                    to_center.y + self.config.block_height / 2.0,
                );
                let curve_offset = self.config.horizontal_spacing * 0.5;
                let control1 = Point2::new(start.x - curve_offset, start.y);
                let control2 = Point2::new(end.x - curve_offset, end.y);
                let label_pos = Point2::new(start.x - curve_offset - 5.0, (start.y + end.y) / 2.0);
                (start, end, control1, control2, label_pos)
            } else {
                let start = Point2::new(from_center.x + self.config.block_width / 2.0, from_center.y);
                let end = Point2::new(
                    to_center.x - self.config.block_width / 2.0,
                    to_center.y - self.config.block_height * 0.25,
                );
                let control_offset = (end.x - start.x).abs() * 0.4;
                let control1 = Point2::new(start.x + control_offset, start.y);
                let control2 = Point2::new(end.x - control_offset, end.y);
                let label_pos = Point2::new(end.x + 5.0, end.y);
                (start, end, control1, control2, label_pos)
            };

            let points = bezier_points(start, control1, control2, end, 40);
            let dashed = dashed_bezier_points(&points, self.config.dash_length, self.config.dash_gap);

            for segment in &dashed {
                draw.path()
                    .stroke()
                    .weight(self.config.modulation_connection_weight)
                    .color(self.config.modulation_connection_color)
                    .points(segment.clone());
            }

            if self.config.show_arrows && points.len() >= 2 {
                let arrow_end = points[points.len() - 1];
                let arrow_prev = points[points.len() - 2];
                draw_arrow_head(
                    draw,
                    arrow_prev,
                    arrow_end,
                    self.config.arrow_size * 0.8,
                    self.config.modulation_connection_color,
                );
            }

            draw.text(&conn.parameter_name)
                .xy(label_pos)
                .color(self.config.modulation_connection_color)
                .font_size(10)
                .right_justify();
        }
    }
}

impl Visualizer for GraphTopologyVisualizer {
    fn update(&mut self) {}

    fn draw(&self, draw: &Draw, bounds: Rect) {
        for conn in &self.topology.connections {
            self.draw_audio_connection(draw, conn, bounds);
        }

        for conn in &self.topology.modulation_connections {
            self.draw_modulation_connection(draw, conn, bounds);
        }

        for (idx, block) in self.topology.blocks.iter().enumerate() {
            if let Some(&pos) = self.block_positions.get(idx) {
                self.draw_block(draw, block, pos, bounds);
            }
        }
    }
}

fn bezier_points(p0: Point2, p1: Point2, p2: Point2, p3: Point2, segments: usize) -> Vec<Point2> {
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;

            Point2::new(
                mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
                mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
            )
        })
        .collect()
}

fn draw_arrow_head(draw: &Draw, from: Point2, to: Point2, size: f32, color: nannou::color::Rgb) {
    let dir = Vec2::new(to.x - from.x, to.y - from.y);
    let len = (dir.x * dir.x + dir.y * dir.y).sqrt();
    if len < 0.001 {
        return;
    }

    let dir = Vec2::new(dir.x / len, dir.y / len);
    let perp = Vec2::new(-dir.y, dir.x);

    let tip = to;
    let left = Point2::new(
        tip.x - dir.x * size + perp.x * size * 0.5,
        tip.y - dir.y * size + perp.y * size * 0.5,
    );
    let right = Point2::new(
        tip.x - dir.x * size - perp.x * size * 0.5,
        tip.y - dir.y * size - perp.y * size * 0.5,
    );

    draw.tri().points(tip, left, right).color(color);
}

fn dashed_bezier_points(points: &[Point2], dash_length: f32, gap_length: f32) -> Vec<Vec<Point2>> {
    let mut segments = Vec::new();
    let mut current_segment = Vec::new();
    let mut distance_in_pattern = 0.0;
    let pattern_length = dash_length + gap_length;

    for i in 0..points.len() {
        let is_dash = (distance_in_pattern % pattern_length) < dash_length;

        if is_dash {
            current_segment.push(points[i]);
        } else if !current_segment.is_empty() {
            segments.push(current_segment);
            current_segment = Vec::new();
        }

        if i + 1 < points.len() {
            let dx = points[i + 1].x - points[i].x;
            let dy = points[i + 1].y - points[i].y;
            let dist = (dx * dx + dy * dy).sqrt();
            distance_in_pattern += dist;
        }
    }

    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_topology() {
        let topology = GraphTopologySnapshot {
            blocks: vec![],
            connections: vec![],
            modulation_connections: vec![],
        };
        let visualizer = GraphTopologyVisualizer::new(topology);
        assert!(visualizer.block_positions.is_empty());
    }

    #[test]
    fn test_single_block() {
        let topology = GraphTopologySnapshot {
            blocks: vec![BlockSnapshot {
                id: 0,
                name: "Oscillator".to_string(),
                category: BlockCategory::Generator,
                input_count: 0,
                output_count: 1,
            }],
            connections: vec![],
            modulation_connections: vec![],
        };
        let visualizer = GraphTopologyVisualizer::new(topology);
        assert_eq!(visualizer.block_positions.len(), 1);
        assert_eq!(visualizer.depths[0], 0);
    }

    #[test]
    fn test_depth_calculation() {
        let topology = GraphTopologySnapshot {
            blocks: vec![
                BlockSnapshot {
                    id: 0,
                    name: "Osc".to_string(),
                    category: BlockCategory::Generator,
                    input_count: 0,
                    output_count: 1,
                },
                BlockSnapshot {
                    id: 1,
                    name: "Gain".to_string(),
                    category: BlockCategory::Effector,
                    input_count: 1,
                    output_count: 1,
                },
                BlockSnapshot {
                    id: 2,
                    name: "Output".to_string(),
                    category: BlockCategory::IO,
                    input_count: 1,
                    output_count: 0,
                },
            ],
            connections: vec![
                ConnectionSnapshot {
                    from_block: 0,
                    from_output: 0,
                    to_block: 1,
                    to_input: 0,
                },
                ConnectionSnapshot {
                    from_block: 1,
                    from_output: 0,
                    to_block: 2,
                    to_input: 0,
                },
            ],
            modulation_connections: vec![],
        };
        let visualizer = GraphTopologyVisualizer::new(topology);
        assert_eq!(visualizer.depths[0], 0);
        assert_eq!(visualizer.depths[1], 1);
        assert_eq!(visualizer.depths[2], 2);
    }
}
