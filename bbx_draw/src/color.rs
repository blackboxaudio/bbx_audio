//! Color palette utilities for visualizations.

use nannou::color::{Rgb, Srgb};

/// Default color palette for visualizations.
///
/// Returns color values as `(r, g, b)` tuples for use in visualizations.
pub struct Palette;

impl Palette {
    /// Generator block color (blue).
    pub fn generator() -> Srgb<u8> {
        Srgb::new(66, 133, 244)
    }

    /// Effector block color (green).
    pub fn effector() -> Srgb<u8> {
        Srgb::new(52, 168, 83)
    }

    /// Modulator block color (purple).
    pub fn modulator() -> Srgb<u8> {
        Srgb::new(156, 39, 176)
    }

    /// I/O block color (orange).
    pub fn io() -> Srgb<u8> {
        Srgb::new(251, 140, 0)
    }

    /// Connection line color (gray).
    pub fn connection() -> Srgb<u8> {
        Srgb::new(158, 158, 158)
    }

    /// Primary text color (white).
    pub fn text() -> Srgb<u8> {
        Srgb::new(255, 255, 255)
    }

    /// Background color (dark gray).
    pub fn background() -> Srgb<u8> {
        Srgb::new(30, 30, 30)
    }

    /// Waveform line color (cyan).
    pub fn waveform() -> Srgb<u8> {
        Srgb::new(0, 188, 212)
    }

    /// Spectrum bar color (magenta).
    pub fn spectrum() -> Srgb<u8> {
        Srgb::new(233, 30, 99)
    }

    /// Spectrum peak color (yellow).
    pub fn spectrum_peak() -> Srgb<u8> {
        Srgb::new(255, 235, 59)
    }

    /// MIDI note on color (green).
    pub fn midi_note_on() -> Srgb<u8> {
        Srgb::new(76, 175, 80)
    }

    /// MIDI note off color (dimmed).
    pub fn midi_note_off() -> Srgb<u8> {
        Srgb::new(66, 66, 66)
    }
}

/// Convert an Srgb<u8> color to normalized Rgb<f32> for nannou.
pub fn to_rgb(color: Srgb<u8>) -> Rgb {
    Rgb::new(
        color.red as f32 / 255.0,
        color.green as f32 / 255.0,
        color.blue as f32 / 255.0,
    )
}

/// Interpolate between two colors.
pub fn lerp_color(a: Srgb<u8>, b: Srgb<u8>, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    Rgb::new(
        (a.red as f32 + (b.red as f32 - a.red as f32) * t) / 255.0,
        (a.green as f32 + (b.green as f32 - a.green as f32) * t) / 255.0,
        (a.blue as f32 + (b.blue as f32 - a.blue as f32) * t) / 255.0,
    )
}
