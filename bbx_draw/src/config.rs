//! Visualization configuration types.

use nannou::color::Rgb;

use crate::color::{Palette, to_rgb};

/// Configuration for the graph topology visualizer.
#[derive(Clone)]
pub struct GraphTopologyConfig {
    /// Width of each block rectangle.
    pub block_width: f32,
    /// Height of each block rectangle.
    pub block_height: f32,
    /// Horizontal spacing between depth columns.
    pub horizontal_spacing: f32,
    /// Vertical spacing between blocks in same column.
    pub vertical_spacing: f32,
    /// Color for generator blocks.
    pub generator_color: Rgb,
    /// Color for effector blocks.
    pub effector_color: Rgb,
    /// Color for modulator blocks.
    pub modulator_color: Rgb,
    /// Color for I/O blocks.
    pub io_color: Rgb,
    /// Color for audio connection lines.
    pub audio_connection_color: Rgb,
    /// Color for modulation connection lines.
    pub modulation_connection_color: Rgb,
    /// Color for block label text.
    pub text_color: Rgb,
    /// Audio connection line weight.
    pub audio_connection_weight: f32,
    /// Modulation connection line weight.
    pub modulation_connection_weight: f32,
    /// Whether to show directional arrows on connections.
    pub show_arrows: bool,
    /// Size of the arrow head.
    pub arrow_size: f32,
    /// Dash length for modulation connections.
    pub dash_length: f32,
    /// Gap length between dashes for modulation connections.
    pub dash_gap: f32,
}

impl Default for GraphTopologyConfig {
    fn default() -> Self {
        Self {
            block_width: 120.0,
            block_height: 50.0,
            horizontal_spacing: 80.0,
            vertical_spacing: 30.0,
            generator_color: to_rgb(Palette::generator()),
            effector_color: to_rgb(Palette::effector()),
            modulator_color: to_rgb(Palette::modulator()),
            io_color: to_rgb(Palette::io()),
            audio_connection_color: to_rgb(Palette::audio_connection()),
            modulation_connection_color: to_rgb(Palette::modulation_connection()),
            text_color: to_rgb(Palette::text()),
            audio_connection_weight: 2.0,
            modulation_connection_weight: 1.5,
            show_arrows: true,
            arrow_size: 8.0,
            dash_length: 8.0,
            dash_gap: 4.0,
        }
    }
}

/// Configuration for the waveform visualizer.
#[derive(Clone)]
pub struct WaveformConfig {
    /// Color for the waveform line.
    pub line_color: Rgb,
    /// Line weight/thickness.
    pub line_weight: f32,
    /// Optional background color (None for transparent).
    pub background_color: Option<Rgb>,
    /// Trigger level for zero-crossing detection (-1.0 to 1.0).
    pub trigger_level: f32,
    /// Number of samples to display in the window.
    pub time_window_samples: usize,
}

impl Default for WaveformConfig {
    fn default() -> Self {
        Self {
            line_color: to_rgb(Palette::waveform()),
            line_weight: 2.0,
            background_color: None,
            trigger_level: 0.0,
            time_window_samples: 1024,
        }
    }
}

/// Display mode for the spectrum analyzer.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SpectrumDisplayMode {
    /// Vertical bars for each frequency bin.
    #[default]
    Bars,
    /// Connected line through bin peaks.
    Line,
    /// Filled area under the spectrum curve.
    Filled,
}

/// Configuration for the spectrum analyzer.
#[derive(Clone)]
pub struct SpectrumConfig {
    /// FFT size (must be power of 2: 512, 1024, 2048, 4096).
    pub fft_size: usize,
    /// Color for spectrum bars/line.
    pub bar_color: Rgb,
    /// Color for peak hold indicators.
    pub peak_color: Rgb,
    /// Minimum dB level to display.
    pub min_db: f32,
    /// Maximum dB level to display.
    pub max_db: f32,
    /// Temporal smoothing factor (0.0 = no smoothing, 1.0 = frozen).
    pub smoothing: f32,
    /// Display mode (bars, line, or filled).
    pub display_mode: SpectrumDisplayMode,
    /// Enable peak hold.
    pub show_peaks: bool,
    /// Peak decay rate (dB per frame).
    pub peak_decay: f32,
}

impl Default for SpectrumConfig {
    fn default() -> Self {
        Self {
            fft_size: 2048,
            bar_color: to_rgb(Palette::spectrum()),
            peak_color: to_rgb(Palette::spectrum_peak()),
            min_db: -80.0,
            max_db: 0.0,
            smoothing: 0.8,
            display_mode: SpectrumDisplayMode::Bars,
            show_peaks: true,
            peak_decay: 0.5,
        }
    }
}

/// Configuration for the MIDI activity visualizer.
#[derive(Clone)]
pub struct MidiActivityConfig {
    /// Color for active (note on) notes.
    pub note_on_color: Rgb,
    /// Color for inactive notes.
    pub note_off_color: Rgb,
    /// Scale brightness by velocity.
    pub velocity_brightness: bool,
    /// MIDI note range to display (min, max). Default is piano range (21-108).
    pub display_range: (u8, u8),
    /// Time for note activity to decay after note off (milliseconds).
    pub decay_time_ms: f32,
}

impl Default for MidiActivityConfig {
    fn default() -> Self {
        Self {
            note_on_color: to_rgb(Palette::midi_note_on()),
            note_off_color: to_rgb(Palette::midi_note_off()),
            velocity_brightness: true,
            display_range: (21, 108),
            decay_time_ms: 200.0,
        }
    }
}
