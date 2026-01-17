//! Voice state management for MIDI-controlled synthesis.
//!
//! This module provides monophonic voice state tracking with support
//! for legato playing (last-note priority).

use bbx_core::StackVec;

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
    note_stack: StackVec<(u8, u8), 16>,
}

impl VoiceState {
    /// Create a new voice state.
    pub fn new() -> Self {
        Self {
            active_note: None,
            velocity: 0.0,
            gate: false,
            frequency: 440.0,
            note_stack: StackVec::new(),
        }
    }

    /// Process a note-on event.
    ///
    /// Updates the active note, velocity, gate state, and frequency.
    /// The note is also pushed onto the note stack for legato handling.
    /// If the note stack is full (16 notes), the note is silently dropped
    /// from the stack but still becomes the active note.
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        let vel_normalized = velocity as f32 / 127.0;

        let _ = self.note_stack.push((note, velocity));

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
        let mut i = 0;
        while i < self.note_stack.len() {
            if self.note_stack[i].0 == note {
                for j in i..self.note_stack.len() - 1 {
                    self.note_stack[j] = self.note_stack[j + 1];
                }
                self.note_stack.pop();
            } else {
                i += 1;
            }
        }

        if self.active_note == Some(note) {
            if let Some(&(prev_note, prev_vel)) = self.note_stack.as_slice().last() {
                self.active_note = Some(prev_note);
                self.velocity = prev_vel as f32 / 127.0;
                self.frequency = midi_note_to_frequency(prev_note);
                false
            } else {
                self.active_note = None;
                self.gate = false;
                true
            }
        } else {
            false
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== midi_note_to_frequency tests ====================

    #[test]
    fn midi_note_to_frequency_a4_is_440hz() {
        let freq = midi_note_to_frequency(69);
        assert!((freq - 440.0).abs() < 0.001);
    }

    #[test]
    fn midi_note_to_frequency_a3_is_220hz() {
        let freq = midi_note_to_frequency(57);
        assert!((freq - 220.0).abs() < 0.001);
    }

    #[test]
    fn midi_note_to_frequency_a5_is_880hz() {
        let freq = midi_note_to_frequency(81);
        assert!((freq - 880.0).abs() < 0.001);
    }

    #[test]
    fn midi_note_to_frequency_c4() {
        let freq = midi_note_to_frequency(60);
        assert!((freq - 261.626).abs() < 0.01);
    }

    #[test]
    fn midi_note_to_frequency_octave_doubles() {
        let freq_a3 = midi_note_to_frequency(57);
        let freq_a4 = midi_note_to_frequency(69);
        let freq_a5 = midi_note_to_frequency(81);

        assert!((freq_a4 / freq_a3 - 2.0).abs() < 0.001);
        assert!((freq_a5 / freq_a4 - 2.0).abs() < 0.001);
    }

    // ==================== VoiceState::new tests ====================

    #[test]
    fn new_voice_state_has_correct_defaults() {
        let state = VoiceState::new();

        assert_eq!(state.active_note, None);
        assert_eq!(state.velocity, 0.0);
        assert!(!state.gate);
        assert_eq!(state.frequency, 440.0);
        assert_eq!(state.held_note_count(), 0);
    }

    #[test]
    fn default_equals_new() {
        let new_state = VoiceState::new();
        let default_state = VoiceState::default();

        assert_eq!(new_state.active_note, default_state.active_note);
        assert_eq!(new_state.velocity, default_state.velocity);
        assert_eq!(new_state.gate, default_state.gate);
        assert_eq!(new_state.frequency, default_state.frequency);
    }

    // ==================== note_on tests ====================

    #[test]
    fn note_on_sets_active_note() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);

        assert_eq!(state.active_note, Some(60));
    }

    #[test]
    fn note_on_sets_gate_true() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);

        assert!(state.gate);
    }

    #[test]
    fn note_on_sets_frequency() {
        let mut state = VoiceState::new();
        state.note_on(69, 100); // A4

        assert!((state.frequency - 440.0).abs() < 0.001);
    }

    #[test]
    fn note_on_velocity_max_normalized_to_1() {
        let mut state = VoiceState::new();
        state.note_on(60, 127);

        assert!((state.velocity - 1.0).abs() < 0.001);
    }

    #[test]
    fn note_on_velocity_zero_normalized_to_0() {
        let mut state = VoiceState::new();
        state.note_on(60, 0);

        assert_eq!(state.velocity, 0.0);
    }

    #[test]
    fn note_on_velocity_mid_normalized() {
        let mut state = VoiceState::new();
        state.note_on(60, 64);

        let expected = 64.0 / 127.0;
        assert!((state.velocity - expected).abs() < 0.001);
    }

    #[test]
    fn note_on_adds_to_note_stack() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);

        assert_eq!(state.held_note_count(), 1);
    }

    #[test]
    fn multiple_note_on_last_is_active() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.note_on(64, 90);
        state.note_on(67, 80);

        assert_eq!(state.active_note, Some(67));
        assert_eq!(state.held_note_count(), 3);
    }

    // ==================== note_off tests ====================

    #[test]
    fn note_off_active_note_returns_true_when_no_held_notes() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);

        let released = state.note_off(60);

        assert!(released);
        assert!(!state.gate);
        assert_eq!(state.active_note, None);
    }

    #[test]
    fn note_off_non_active_note_returns_false() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.note_on(64, 90);

        let released = state.note_off(60);

        assert!(!released);
        assert!(state.gate);
        assert_eq!(state.active_note, Some(64));
    }

    #[test]
    fn note_off_removes_from_stack() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.note_on(64, 90);

        state.note_off(60);

        assert_eq!(state.held_note_count(), 1);
    }

    #[test]
    fn note_off_legato_switches_to_previous() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.note_on(64, 90);
        state.note_on(67, 80);

        let released = state.note_off(67);

        assert!(!released);
        assert!(state.gate);
        assert_eq!(state.active_note, Some(64));
    }

    #[test]
    fn note_off_legato_restores_previous_velocity() {
        let mut state = VoiceState::new();
        state.note_on(60, 127);
        state.note_on(64, 64);

        state.note_off(64);

        assert!((state.velocity - 1.0).abs() < 0.001);
    }

    #[test]
    fn note_off_legato_restores_previous_frequency() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.note_on(64, 100);

        state.note_off(64);

        let expected_freq = midi_note_to_frequency(60);
        assert!((state.frequency - expected_freq).abs() < 0.001);
    }

    #[test]
    fn note_off_note_not_in_stack_does_nothing() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);

        let released = state.note_off(72);

        assert!(!released);
        assert!(state.gate);
        assert_eq!(state.active_note, Some(60));
        assert_eq!(state.held_note_count(), 1);
    }

    // ==================== reset tests ====================

    #[test]
    fn reset_clears_active_note() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.reset();

        assert_eq!(state.active_note, None);
    }

    #[test]
    fn reset_clears_gate() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.reset();

        assert!(!state.gate);
    }

    #[test]
    fn reset_clears_velocity() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.reset();

        assert_eq!(state.velocity, 0.0);
    }

    #[test]
    fn reset_resets_frequency_to_440() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.reset();

        assert_eq!(state.frequency, 440.0);
    }

    #[test]
    fn reset_clears_note_stack() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.note_on(64, 90);
        state.note_on(67, 80);
        state.reset();

        assert_eq!(state.held_note_count(), 0);
    }

    // ==================== is_active tests ====================

    #[test]
    fn is_active_false_when_new() {
        let state = VoiceState::new();
        assert!(!state.is_active());
    }

    #[test]
    fn is_active_true_after_note_on() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);

        assert!(state.is_active());
    }

    #[test]
    fn is_active_false_after_last_note_off() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.note_off(60);

        assert!(!state.is_active());
    }

    // ==================== held_note_count tests ====================

    #[test]
    fn held_note_count_zero_when_new() {
        let state = VoiceState::new();
        assert_eq!(state.held_note_count(), 0);
    }

    #[test]
    fn held_note_count_increments_on_note_on() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        assert_eq!(state.held_note_count(), 1);

        state.note_on(64, 90);
        assert_eq!(state.held_note_count(), 2);

        state.note_on(67, 80);
        assert_eq!(state.held_note_count(), 3);
    }

    #[test]
    fn held_note_count_decrements_on_note_off() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);
        state.note_on(64, 90);
        state.note_off(60);

        assert_eq!(state.held_note_count(), 1);
    }

    // ==================== edge case tests ====================

    #[test]
    fn note_stack_overflow_still_becomes_active() {
        let mut state = VoiceState::new();

        for note in 0..20 {
            state.note_on(note, 100);
        }

        assert_eq!(state.active_note, Some(19));
        assert_eq!(state.held_note_count(), 16);
    }

    #[test]
    fn releasing_same_note_twice_is_safe() {
        let mut state = VoiceState::new();
        state.note_on(60, 100);

        state.note_off(60);
        let second_release = state.note_off(60);

        assert!(!second_release);
        assert!(!state.gate);
    }

    #[test]
    fn complex_legato_sequence() {
        let mut state = VoiceState::new();

        state.note_on(60, 100);
        state.note_on(64, 90);
        state.note_on(67, 80);

        assert_eq!(state.active_note, Some(67));

        state.note_off(64);
        assert_eq!(state.active_note, Some(67));

        state.note_off(67);
        assert_eq!(state.active_note, Some(60));

        let released = state.note_off(60);
        assert!(released);
        assert_eq!(state.active_note, None);
    }
}
