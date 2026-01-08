//! FFT-based spectrum analyzer.

use std::f32::consts::PI;

use nannou::{
    Draw,
    geom::{Point2, Rect},
};
use rustfft::{FftPlanner, num_complex::Complex};

use crate::{
    Visualizer,
    bridge::AudioBridgeConsumer,
    config::{SpectrumConfig, SpectrumDisplayMode},
};

/// FFT-based spectrum analyzer visualizer.
///
/// Displays frequency content of audio using bars, line, or filled display modes.
/// Features temporal smoothing and optional peak hold.
pub struct SpectrumAnalyzer {
    consumer: AudioBridgeConsumer,
    config: SpectrumConfig,
    fft_planner: FftPlanner<f32>,
    window: Vec<f32>,
    sample_buffer: Vec<f32>,
    fft_buffer: Vec<Complex<f32>>,
    magnitudes: Vec<f32>,
    smoothed_magnitudes: Vec<f32>,
    peaks: Vec<f32>,
    write_position: usize,
}

impl SpectrumAnalyzer {
    /// Create a new spectrum analyzer with default configuration.
    pub fn new(consumer: AudioBridgeConsumer) -> Self {
        let config = SpectrumConfig::default();
        Self::with_config(consumer, config)
    }

    /// Create a new spectrum analyzer with custom configuration.
    pub fn with_config(consumer: AudioBridgeConsumer, config: SpectrumConfig) -> Self {
        let fft_size = config.fft_size;
        let window = hann_window(fft_size);
        let num_bins = fft_size / 2;

        Self {
            consumer,
            fft_planner: FftPlanner::new(),
            window,
            sample_buffer: vec![0.0; fft_size],
            fft_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            magnitudes: vec![0.0; num_bins],
            smoothed_magnitudes: vec![config.min_db; num_bins],
            peaks: vec![config.min_db; num_bins],
            write_position: 0,
            config,
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &SpectrumConfig {
        &self.config
    }

    /// Update the configuration (may reset internal state).
    pub fn set_config(&mut self, config: SpectrumConfig) {
        if config.fft_size != self.config.fft_size {
            let fft_size = config.fft_size;
            let num_bins = fft_size / 2;
            self.window = hann_window(fft_size);
            self.sample_buffer = vec![0.0; fft_size];
            self.fft_buffer = vec![Complex::new(0.0, 0.0); fft_size];
            self.magnitudes = vec![0.0; num_bins];
            self.smoothed_magnitudes = vec![config.min_db; num_bins];
            self.peaks = vec![config.min_db; num_bins];
            self.write_position = 0;
        }
        self.config = config;
    }

    fn compute_fft(&mut self) {
        let fft_size = self.config.fft_size;
        let fft = self.fft_planner.plan_fft_forward(fft_size);

        for i in 0..fft_size {
            let idx = (self.write_position + i) % fft_size;
            self.fft_buffer[i] = Complex::new(self.sample_buffer[idx] * self.window[i], 0.0);
        }

        fft.process(&mut self.fft_buffer);

        let num_bins = fft_size / 2;
        let scale = 2.0 / fft_size as f32;

        for i in 0..num_bins {
            let magnitude = self.fft_buffer[i].norm() * scale;
            self.magnitudes[i] = amplitude_to_db(magnitude);
        }
    }

    fn apply_smoothing(&mut self) {
        let smoothing = self.config.smoothing;
        for i in 0..self.magnitudes.len() {
            self.smoothed_magnitudes[i] =
                smoothing * self.smoothed_magnitudes[i] + (1.0 - smoothing) * self.magnitudes[i];
        }
    }

    fn update_peaks(&mut self) {
        let decay = self.config.peak_decay;
        for i in 0..self.magnitudes.len() {
            if self.smoothed_magnitudes[i] > self.peaks[i] {
                self.peaks[i] = self.smoothed_magnitudes[i];
            } else {
                self.peaks[i] -= decay;
                self.peaks[i] = self.peaks[i].max(self.config.min_db);
            }
        }
    }

    fn db_to_normalized(&self, db: f32) -> f32 {
        let range = self.config.max_db - self.config.min_db;
        ((db - self.config.min_db) / range).clamp(0.0, 1.0)
    }
}

impl Visualizer for SpectrumAnalyzer {
    fn update(&mut self) {
        let frames = self.consumer.drain();
        let mut has_data = false;

        for frame in frames {
            let channel_samples = frame.channel_samples(0);
            for sample in channel_samples {
                self.sample_buffer[self.write_position] = sample;
                self.write_position = (self.write_position + 1) % self.config.fft_size;
                has_data = true;
            }
        }

        if has_data {
            self.compute_fft();
            self.apply_smoothing();
            if self.config.show_peaks {
                self.update_peaks();
            }
        }
    }

    fn draw(&self, draw: &Draw, bounds: Rect) {
        let num_bins = self.smoothed_magnitudes.len();
        if num_bins == 0 {
            return;
        }

        match self.config.display_mode {
            SpectrumDisplayMode::Bars => {
                let bar_width = bounds.w() / num_bins as f32;

                for (i, &db) in self.smoothed_magnitudes.iter().enumerate() {
                    let height = self.db_to_normalized(db) * bounds.h();
                    let x = bounds.left() + (i as f32 + 0.5) * bar_width;
                    let y = bounds.bottom() + height / 2.0;

                    draw.rect()
                        .x_y(x, y)
                        .w_h(bar_width * 0.8, height)
                        .color(self.config.bar_color);

                    if self.config.show_peaks {
                        let peak_height = self.db_to_normalized(self.peaks[i]) * bounds.h();
                        let peak_y = bounds.bottom() + peak_height;
                        draw.line()
                            .start(Point2::new(x - bar_width * 0.4, peak_y))
                            .end(Point2::new(x + bar_width * 0.4, peak_y))
                            .weight(2.0)
                            .color(self.config.peak_color);
                    }
                }
            }
            SpectrumDisplayMode::Line => {
                let points: Vec<Point2> = self
                    .smoothed_magnitudes
                    .iter()
                    .enumerate()
                    .map(|(i, &db)| {
                        let x = bounds.left() + (i as f32 / num_bins as f32) * bounds.w();
                        let y = bounds.bottom() + self.db_to_normalized(db) * bounds.h();
                        Point2::new(x, y)
                    })
                    .collect();

                if points.len() >= 2 {
                    draw.polyline().weight(2.0).color(self.config.bar_color).points(points);
                }
            }
            SpectrumDisplayMode::Filled => {
                let mut points: Vec<Point2> = Vec::with_capacity(num_bins + 2);

                points.push(Point2::new(bounds.left(), bounds.bottom()));

                for (i, &db) in self.smoothed_magnitudes.iter().enumerate() {
                    let x = bounds.left() + (i as f32 / num_bins as f32) * bounds.w();
                    let y = bounds.bottom() + self.db_to_normalized(db) * bounds.h();
                    points.push(Point2::new(x, y));
                }

                points.push(Point2::new(bounds.right(), bounds.bottom()));

                if points.len() >= 3 {
                    draw.polygon().color(self.config.bar_color).points(points);
                }
            }
        }
    }
}

fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / size as f32).cos()))
        .collect()
}

fn amplitude_to_db(amplitude: f32) -> f32 {
    if amplitude <= 0.0 {
        -120.0
    } else {
        20.0 * amplitude.log10()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::audio_bridge;

    #[test]
    fn test_spectrum_creation() {
        let (_producer, consumer) = audio_bridge(4);
        let analyzer = SpectrumAnalyzer::new(consumer);
        assert_eq!(analyzer.sample_buffer.len(), 2048);
        assert_eq!(analyzer.magnitudes.len(), 1024);
    }

    #[test]
    fn test_hann_window() {
        let window = hann_window(4);
        assert!((window[0] - 0.0).abs() < 0.001);
        assert!((window[1] - 0.5).abs() < 0.001);
        assert!((window[2] - 1.0).abs() < 0.001);
        assert!((window[3] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_amplitude_to_db() {
        assert!((amplitude_to_db(1.0) - 0.0).abs() < 0.001);
        assert!((amplitude_to_db(0.1) - (-20.0)).abs() < 0.001);
    }
}
