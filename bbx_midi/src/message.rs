//! MIDI message types and parsing.

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "std")]
use core::fmt::{Display, Formatter};
#[cfg(feature = "std")]
use std::time::SystemTime;

#[cfg(feature = "alloc")]
const NOTES: [&str; 12] = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];

/// A parsed MIDI message with channel, status, and data bytes.
///
/// Uses `#[repr(C)]` for C-compatible memory layout, enabling FFI usage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MidiMessage {
    channel: u8,
    status: MidiMessageStatus,
    data_1: u8,
    data_2: u8,
}

/// MIDI message status byte types.
///
/// Uses `#[repr(C)]` for C-compatible memory layout, enabling FFI usage.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MidiMessageStatus {
    /// Unrecognized or system message.
    Unknown = 0,
    /// Note released (0x80-0x8F).
    NoteOff = 1,
    /// Note pressed (0x90-0x9F).
    NoteOn = 2,
    /// Per-note pressure change (0xA0-0xAF).
    PolyphonicAftertouch = 3,
    /// Controller value change (0xB0-0xBF).
    ControlChange = 4,
    /// Instrument/patch change (0xC0-0xCF).
    ProgramChange = 5,
    /// Channel-wide pressure (0xD0-0xDF).
    ChannelAftertouch = 6,
    /// Pitch bend wheel (0xE0-0xEF).
    PitchWheel = 7,
}

/// A MIDI event with sample-accurate timing for audio buffer processing.
///
/// Combines a MIDI message with a sample offset indicating when the event
/// should be processed within the current audio buffer.
///
/// Uses `#[repr(C)]` for C-compatible memory layout, enabling FFI usage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MidiEvent {
    /// The MIDI message data.
    pub message: MidiMessage,
    /// Sample offset within the current buffer (0 to buffer_size - 1).
    pub sample_offset: u32,
}

impl MidiEvent {
    /// Create a new MIDI event with the given message and sample offset.
    pub fn new(message: MidiMessage, sample_offset: u32) -> Self {
        Self { message, sample_offset }
    }
}

impl From<u8> for MidiMessageStatus {
    fn from(byte: u8) -> Self {
        match byte {
            // 128 - 144
            0x80..0x90 => MidiMessageStatus::NoteOff,
            // 144 - 160
            0x90..0xA0 => MidiMessageStatus::NoteOn,
            // 160 - 176
            0xA0..0xB0 => MidiMessageStatus::PolyphonicAftertouch,
            // 176 - 192
            0xB0..0xC0 => MidiMessageStatus::ControlChange,
            // 192 - 208
            0xC0..0xD0 => MidiMessageStatus::ProgramChange,
            // 208 - 224
            0xD0..0xE0 => MidiMessageStatus::ChannelAftertouch,
            // 224 - 255
            0xE0..=0xFF => MidiMessageStatus::PitchWheel,
            _ => MidiMessageStatus::Unknown,
        }
    }
}

impl MidiMessage {
    /// Create a new MIDI message from raw bytes.
    pub fn new(bytes: [u8; 3]) -> Self {
        MidiMessage {
            channel: (bytes[0] & 0x0F) + 1,
            status: MidiMessageStatus::from(bytes[0]),
            data_1: bytes[1],
            data_2: bytes[2],
        }
    }
}

impl MidiMessage {
    /// Get the message status type.
    pub fn get_status(&self) -> MidiMessageStatus {
        self.status
    }

    /// Get the MIDI channel (1-16).
    pub fn get_channel(&self) -> u8 {
        self.channel
    }

    fn get_data(&self, data_field: usize, statuses: &[MidiMessageStatus]) -> Option<u8> {
        if statuses.contains(&self.status) {
            match data_field {
                1 => Some(self.data_1),
                2 => Some(self.data_2),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the note name (e.g., "C4", "F#3") for note messages.
    #[cfg(feature = "alloc")]
    pub fn get_note(&self) -> Option<String> {
        use alloc::format;

        let note_number = self.get_note_number()?;
        // Determine the note name (C, C#, D, etc.)
        let note_index = (note_number % 12) as usize;
        let note_name = NOTES[note_index];

        // Determine the octave (MIDI note 60 is C4, middle C)
        let octave = (note_number / 12) as i8 - 1;

        // Return the formatted string (e.g., "C4", "D#3")
        Some(format!("{note_name}{octave}"))
    }

    /// Get the note frequency in Hz (A4 = 440 Hz) for note messages.
    #[cfg(feature = "std")]
    pub fn get_note_frequency(&self) -> Option<f32> {
        let note_number = self.get_note_number()?;
        Some(440.0 * 2.0f32.powf((note_number as f32 - 69.0) / 12.0))
    }

    /// Get the MIDI note number (0-127) for note messages.
    pub fn get_note_number(&self) -> Option<u8> {
        self.get_data(1, &[MidiMessageStatus::NoteOn, MidiMessageStatus::NoteOff])
    }

    /// Get the velocity (0-127) for note messages.
    pub fn get_velocity(&self) -> Option<u8> {
        self.get_data(2, &[MidiMessageStatus::NoteOn, MidiMessageStatus::NoteOff])
    }

    /// Get the pressure value (0-127) for polyphonic aftertouch.
    pub fn get_pressure(&self) -> Option<u8> {
        self.get_data(2, &[MidiMessageStatus::PolyphonicAftertouch])
    }

    /// Get the control value (0-127) for control change messages.
    pub fn get_control_change_data(&self) -> Option<u8> {
        self.get_data(2, &[MidiMessageStatus::ControlChange])
    }

    /// Get the pitch wheel data (LSB, MSB) for pitch bend messages.
    pub fn get_pitch_wheel_data(&self) -> Option<(u8, u8)> {
        let least_significant_byte = self.get_data(1, &[MidiMessageStatus::PitchWheel])?;
        let most_significant_byte = self.get_data(2, &[MidiMessageStatus::PitchWheel])?;
        Some((least_significant_byte, most_significant_byte))
    }
}

#[cfg(feature = "std")]
impl Display for MidiMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        match self.status {
            MidiMessageStatus::NoteOff | MidiMessageStatus::NoteOn => {
                write!(
                    f,
                    "[{}] Ch {} {:?}\t Note = {} ({}Hz)\t Velocity = {}",
                    now,
                    self.channel,
                    self.status,
                    self.get_note().unwrap(),
                    self.get_note_frequency().unwrap(),
                    self.get_velocity().unwrap()
                )
            }
            _ => {
                write!(
                    f,
                    "[{}] Ch {} {:?}\t Data 1 = {}\t Data 2 = {}",
                    now, self.channel, self.status, self.data_1, self.data_2
                )
            }
        }
    }
}

impl From<&[u8]> for MidiMessage {
    fn from(bytes: &[u8]) -> Self {
        match bytes.len() {
            1 => MidiMessage {
                channel: (bytes[0] & 0x0F) + 1,
                status: MidiMessageStatus::from(bytes[0]),
                data_1: 0,
                data_2: 0,
            },
            2 => MidiMessage {
                channel: (bytes[0] & 0x0F) + 1,
                status: MidiMessageStatus::from(bytes[0]),
                data_1: bytes[1],
                data_2: 0,
            },
            3 => MidiMessage {
                channel: (bytes[0] & 0x0F) + 1,
                status: MidiMessageStatus::from(bytes[0]),
                data_1: bytes[1],
                data_2: bytes[2],
            },
            _ => MidiMessage {
                channel: (bytes[0] & 0x0F) + 1,
                status: MidiMessageStatus::Unknown,
                data_1: 0,
                data_2: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_on_parsing() {
        // Note On, channel 1, note 60 (C4), velocity 100
        let msg = MidiMessage::new([0x90, 60, 100]);
        assert_eq!(msg.get_status(), MidiMessageStatus::NoteOn);
        assert_eq!(msg.get_channel(), 1);
        assert_eq!(msg.get_note_number(), Some(60));
        assert_eq!(msg.get_velocity(), Some(100));
        assert_eq!(msg.get_note(), Some("C4".to_string()));
    }

    #[test]
    fn test_note_off_parsing() {
        // Note Off, channel 3, note 64 (E4), velocity 64
        let msg = MidiMessage::new([0x82, 64, 64]);
        assert_eq!(msg.get_status(), MidiMessageStatus::NoteOff);
        assert_eq!(msg.get_channel(), 3);
        assert_eq!(msg.get_note_number(), Some(64));
        assert_eq!(msg.get_velocity(), Some(64));
        assert_eq!(msg.get_note(), Some("E4".to_string()));
    }

    #[test]
    fn test_control_change_parsing() {
        // Control Change, channel 1, controller 7 (volume), value 127
        let msg = MidiMessage::new([0xB0, 7, 127]);
        assert_eq!(msg.get_status(), MidiMessageStatus::ControlChange);
        assert_eq!(msg.get_channel(), 1);
        assert_eq!(msg.get_control_change_data(), Some(127));
        assert_eq!(msg.get_note_number(), None);
    }

    #[test]
    fn test_channel_extraction() {
        // Channels are 1-16 in user-facing API (internal byte is 0-15)
        for ch in 0..16u8 {
            let msg = MidiMessage::new([0x90 | ch, 60, 100]);
            assert_eq!(msg.get_channel(), ch + 1);
        }
    }

    #[test]
    fn test_note_frequency() {
        // A4 (note 69) should be 440 Hz
        let msg = MidiMessage::new([0x90, 69, 100]);
        let freq = msg.get_note_frequency().unwrap();
        assert!((freq - 440.0).abs() < 0.01);
    }

    #[test]
    fn test_from_slice() {
        // Test parsing from byte slice
        let bytes: &[u8] = &[0x90, 60, 100];
        let msg = MidiMessage::from(bytes);
        assert_eq!(msg.get_status(), MidiMessageStatus::NoteOn);
        assert_eq!(msg.get_note_number(), Some(60));
    }

    #[test]
    fn test_pitch_wheel() {
        // Pitch wheel message
        let msg = MidiMessage::new([0xE0, 0x00, 0x40]);
        assert_eq!(msg.get_status(), MidiMessageStatus::PitchWheel);
        assert_eq!(msg.get_pitch_wheel_data(), Some((0x00, 0x40)));
    }
}
