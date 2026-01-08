//! MIDI note activity visualizer.

use bbx_midi::message::MidiMessageStatus;
use nannou::{Draw, color::Rgb, geom::Rect};

use crate::{Visualizer, bridge::MidiBridgeConsumer, color::lerp_color, config::MidiActivityConfig};

/// Visualizes MIDI note activity with decay animation.
///
/// Active notes light up based on velocity, then decay after note off.
pub struct MidiActivityVisualizer {
    consumer: MidiBridgeConsumer,
    config: MidiActivityConfig,
    note_activity: [f32; 128],
    note_velocities: [u8; 128],
    decay_per_frame: f32,
}

impl MidiActivityVisualizer {
    /// Create a new MIDI activity visualizer with default configuration.
    pub fn new(consumer: MidiBridgeConsumer) -> Self {
        let config = MidiActivityConfig::default();
        let decay_per_frame = 1.0 / (config.decay_time_ms / 16.67);
        Self {
            consumer,
            config,
            note_activity: [0.0; 128],
            note_velocities: [0; 128],
            decay_per_frame,
        }
    }

    /// Create a new MIDI activity visualizer with custom configuration.
    pub fn with_config(consumer: MidiBridgeConsumer, config: MidiActivityConfig) -> Self {
        let decay_per_frame = 1.0 / (config.decay_time_ms / 16.67);
        Self {
            consumer,
            note_activity: [0.0; 128],
            note_velocities: [0; 128],
            decay_per_frame,
            config,
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &MidiActivityConfig {
        &self.config
    }

    /// Update the configuration.
    pub fn set_config(&mut self, config: MidiActivityConfig) {
        self.decay_per_frame = 1.0 / (config.decay_time_ms / 16.67);
        self.config = config;
    }

    fn note_color(&self, note: u8) -> Rgb {
        let activity = self.note_activity[note as usize];
        if activity <= 0.0 {
            return nannou::color::rgb(
                self.config.note_off_color.red,
                self.config.note_off_color.green,
                self.config.note_off_color.blue,
            );
        }

        let base_color = if self.config.velocity_brightness {
            let velocity_factor = self.note_velocities[note as usize] as f32 / 127.0;
            let r = self.config.note_on_color.red * velocity_factor;
            let g = self.config.note_on_color.green * velocity_factor;
            let b = self.config.note_on_color.blue * velocity_factor;
            nannou::color::Srgb::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
        } else {
            nannou::color::Srgb::new(
                (self.config.note_on_color.red * 255.0) as u8,
                (self.config.note_on_color.green * 255.0) as u8,
                (self.config.note_on_color.blue * 255.0) as u8,
            )
        };

        let off_color = nannou::color::Srgb::new(
            (self.config.note_off_color.red * 255.0) as u8,
            (self.config.note_off_color.green * 255.0) as u8,
            (self.config.note_off_color.blue * 255.0) as u8,
        );

        lerp_color(off_color, base_color, activity)
    }
}

impl Visualizer for MidiActivityVisualizer {
    fn update(&mut self) {
        let messages = self.consumer.drain();

        for msg in messages {
            let status = msg.get_status();
            if let Some(note) = msg.get_note_number() {
                match status {
                    MidiMessageStatus::NoteOn => {
                        if let Some(velocity) = msg.get_velocity() {
                            if velocity > 0 {
                                self.note_activity[note as usize] = 1.0;
                                self.note_velocities[note as usize] = velocity;
                            } else {
                                self.note_activity[note as usize] = 0.9;
                            }
                        }
                    }
                    MidiMessageStatus::NoteOff => {
                        self.note_activity[note as usize] = 0.9;
                    }
                    _ => {}
                }
            }
        }

        for activity in &mut self.note_activity {
            if *activity > 0.0 && *activity < 1.0 {
                *activity -= self.decay_per_frame;
                *activity = activity.max(0.0);
            }
        }
    }

    fn draw(&self, draw: &Draw, bounds: Rect) {
        let (min_note, max_note) = self.config.display_range;
        let num_notes = (max_note - min_note + 1) as usize;

        if num_notes == 0 {
            return;
        }

        let key_width = bounds.w() / num_notes as f32;
        let key_height = bounds.h();

        for i in 0..num_notes {
            let note = min_note + i as u8;
            let x = bounds.left() + (i as f32 + 0.5) * key_width;
            let y = bounds.y();

            let color = self.note_color(note);

            let is_black_key = matches!(note % 12, 1 | 3 | 6 | 8 | 10);
            let actual_height = if is_black_key { key_height * 0.65 } else { key_height };

            draw.rect().x_y(x, y).w_h(key_width * 0.9, actual_height).color(color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::midi_bridge;

    #[test]
    fn test_midi_visualizer_creation() {
        let (_producer, consumer) = midi_bridge(64);
        let visualizer = MidiActivityVisualizer::new(consumer);
        assert_eq!(visualizer.note_activity.len(), 128);
    }

    #[test]
    fn test_display_range() {
        let (_producer, consumer) = midi_bridge(64);
        let config = MidiActivityConfig {
            display_range: (60, 72),
            ..Default::default()
        };
        let visualizer = MidiActivityVisualizer::with_config(consumer, config);
        assert_eq!(visualizer.config.display_range, (60, 72));
    }
}
