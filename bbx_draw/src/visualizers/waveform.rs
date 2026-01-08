//! Oscilloscope-style waveform visualizer.

use nannou::{
    Draw,
    geom::{Point2, Rect},
};

use crate::{Visualizer, bridge::AudioBridgeConsumer, config::WaveformConfig};

/// Displays audio waveform in oscilloscope style.
///
/// Features zero-crossing trigger detection for stable display of periodic waveforms.
pub struct WaveformVisualizer {
    consumer: AudioBridgeConsumer,
    config: WaveformConfig,
    sample_buffer: Vec<f32>,
    write_position: usize,
}

impl WaveformVisualizer {
    /// Create a new waveform visualizer with default configuration.
    pub fn new(consumer: AudioBridgeConsumer) -> Self {
        let config = WaveformConfig::default();
        let buffer_size = config.time_window_samples * 2;
        Self {
            consumer,
            config: config.clone(),
            sample_buffer: vec![0.0; buffer_size],
            write_position: 0,
        }
    }

    /// Create a new waveform visualizer with custom configuration.
    pub fn with_config(consumer: AudioBridgeConsumer, config: WaveformConfig) -> Self {
        let buffer_size = config.time_window_samples * 2;
        Self {
            consumer,
            sample_buffer: vec![0.0; buffer_size],
            write_position: 0,
            config,
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &WaveformConfig {
        &self.config
    }

    /// Update the configuration.
    pub fn set_config(&mut self, config: WaveformConfig) {
        let buffer_size = config.time_window_samples * 2;
        if buffer_size != self.sample_buffer.len() {
            self.sample_buffer.resize(buffer_size, 0.0);
            self.write_position = 0;
        }
        self.config = config;
    }

    fn find_trigger_point(&self) -> usize {
        let buffer_len = self.sample_buffer.len();
        let search_range = buffer_len.saturating_sub(self.config.time_window_samples);
        let trigger = self.config.trigger_level;

        for i in 0..search_range {
            let idx = (self.write_position + buffer_len - search_range + i) % buffer_len;
            let next_idx = (idx + 1) % buffer_len;

            let curr = self.sample_buffer[idx];
            let next = self.sample_buffer[next_idx];

            if curr <= trigger && next > trigger {
                return idx;
            }
        }

        (self.write_position + buffer_len - self.config.time_window_samples) % buffer_len
    }
}

impl Visualizer for WaveformVisualizer {
    fn update(&mut self) {
        let frames = self.consumer.drain();
        for frame in frames {
            let channel_samples = frame.channel_samples(0);
            for sample in channel_samples {
                self.sample_buffer[self.write_position] = sample;
                self.write_position = (self.write_position + 1) % self.sample_buffer.len();
            }
        }
    }

    fn draw(&self, draw: &Draw, bounds: Rect) {
        if let Some(bg) = self.config.background_color {
            draw.rect().xy(bounds.xy()).wh(bounds.wh()).color(bg);
        }

        let trigger_point = self.find_trigger_point();
        let buffer_len = self.sample_buffer.len();
        let window_samples = self.config.time_window_samples;

        let points: Vec<Point2> = (0..window_samples)
            .map(|i| {
                let idx = (trigger_point + i) % buffer_len;
                let sample = self.sample_buffer[idx];

                let x = bounds.left() + (i as f32 / window_samples as f32) * bounds.w();
                let y = bounds.y() + sample * (bounds.h() / 2.0);

                Point2::new(x, y)
            })
            .collect();

        if points.len() >= 2 {
            draw.polyline()
                .weight(self.config.line_weight)
                .color(self.config.line_color)
                .points(points);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::audio_bridge;

    #[test]
    fn test_waveform_creation() {
        let (_producer, consumer) = audio_bridge(4);
        let visualizer = WaveformVisualizer::new(consumer);
        assert_eq!(visualizer.sample_buffer.len(), 2048);
    }

    #[test]
    fn test_custom_config() {
        let (_producer, consumer) = audio_bridge(4);
        let config = WaveformConfig {
            time_window_samples: 512,
            ..Default::default()
        };
        let visualizer = WaveformVisualizer::with_config(consumer, config);
        assert_eq!(visualizer.sample_buffer.len(), 1024);
    }
}
