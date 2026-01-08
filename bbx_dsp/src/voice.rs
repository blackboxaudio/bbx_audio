//! Voice state management for MIDI-controlled synthesis.
//!
//! This module provides monophonic voice state tracking with support
//! for legato playing (last-note priority).

/// Converts a MIDI note number to frequency in Hz.
///
/// Uses A4 = 440 Hz as the reference.
#[inline]
pub fn midi_note_to_frequency(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

/// Monophonic voice state for MIDI-controlled synthesis.
///
/// Tracks the currently active note, velocity, gate state, and frequency.
/// Supports legato playing with last-note priority: when multiple notes
/// are held, releasing a note will return to the previous held note
/// without retriggering the envelope.
#[derive(Debug, Clone)]
pub struct VoiceState {
    /// The currently active MIDI note number, if any.
    pub active_note: Option<u8>,
    /// Velocity of the active note (0.0 to 1.0).
    pub velocity: f32,
    /// Gate state: true while a note is held.
    pub gate: bool,
    /// Frequency in Hz for the current note.
    pub frequency: f32,
    /// Stack of held notes for legato playing: (note, velocity).
    note_stack: Vec<(u8, u8)>,
}

impl VoiceState {
    /// Create a new voice state with pre-allocated note stack.
    pub fn new() -> Self {
        Self {
            active_note: None,
            velocity: 0.0,
            gate: false,
            frequency: 440.0,
            note_stack: Vec::with_capacity(16),
        }
    }

    /// Process a note-on event.
    ///
    /// Updates the active note, velocity, gate state, and frequency.
    /// The note is also pushed onto the note stack for legato handling.
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        let vel_normalized = velocity as f32 / 127.0;

        // Add to stack
        self.note_stack.push((note, velocity));

        // Update active state
        self.active_note = Some(note);
        self.velocity = vel_normalized;
        self.gate = true;
        self.frequency = midi_note_to_frequency(note);
    }

    /// Process a note-off event.
    ///
    /// Returns `true` if the voice should enter release stage (no more notes held),
    /// or `false` if switching to a previous legato note.
    pub fn note_off(&mut self, note: u8) -> bool {
        // Remove note from stack
        self.note_stack.retain(|(n, _)| *n != note);

        if self.active_note == Some(note) {
            // Check if there are other held notes (legato)
            if let Some(&(prev_note, prev_vel)) = self.note_stack.last() {
                // Switch to previous note (legato - don't retrigger envelope)
                self.active_note = Some(prev_note);
                self.velocity = prev_vel as f32 / 127.0;
                self.frequency = midi_note_to_frequency(prev_note);
                false // Don't release, just change pitch
            } else {
                // No more notes held, release
                self.active_note = None;
                self.gate = false;
                true // Trigger release
            }
        } else {
            false // Note wasn't active anyway
        }
    }

    /// Reset the voice state.
    ///
    /// Clears all state and the note stack.
    pub fn reset(&mut self) {
        self.active_note = None;
        self.velocity = 0.0;
        self.gate = false;
        self.frequency = 440.0;
        self.note_stack.clear();
    }

    /// Returns true if a note is currently active.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active_note.is_some()
    }

    /// Returns the number of notes currently held.
    #[inline]
    pub fn held_note_count(&self) -> usize {
        self.note_stack.len()
    }
}

impl Default for VoiceState {
    fn default() -> Self {
        Self::new()
    }
}
