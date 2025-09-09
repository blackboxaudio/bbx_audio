use std::{
    fmt::{Display, Formatter},
    time::SystemTime,
};

const NOTES: [&str; 12] = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];

pub struct MidiMessage {
    channel: u8,
    status: MidiMessageStatus,
    data_1: u8,
    data_2: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MidiMessageStatus {
    Unknown,
    NoteOff,
    NoteOn,
    PolyphonicAftertouch,
    ControlChange,
    ProgramChange,
    ChannelAftertouch,
    PitchWheel,
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
    pub fn get_status(&self) -> MidiMessageStatus {
        self.status
    }

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

    pub fn get_note(&self) -> Option<String> {
        let note_number = self.get_note_number()?;
        // Determine the note name (C, C#, D, etc.)
        let note_index = (note_number % 12) as usize;
        let note_name = NOTES[note_index];

        // Determine the octave (MIDI note 60 is C4, middle C)
        let octave = (note_number / 12) as i8 - 1;

        // Return the formatted string (e.g., "C4", "D#3")
        Some(format!("{note_name}{octave}"))
    }

    pub fn get_note_frequency(&self) -> Option<f32> {
        let note_number = self.get_note_number()?;
        Some(440.0 * 2.0f32.powf((note_number as f32 - 69.0) / 12.0))
    }

    pub fn get_note_number(&self) -> Option<u8> {
        self.get_data(1, &[MidiMessageStatus::NoteOn, MidiMessageStatus::NoteOff])
    }

    pub fn get_velocity(&self) -> Option<u8> {
        self.get_data(2, &[MidiMessageStatus::NoteOn, MidiMessageStatus::NoteOff])
    }

    pub fn get_pressure(&self) -> Option<u8> {
        self.get_data(2, &[MidiMessageStatus::PolyphonicAftertouch])
    }

    pub fn get_control_change_data(&self) -> Option<u8> {
        self.get_data(2, &[MidiMessageStatus::ControlChange])
    }

    pub fn get_pitch_wheel_data(&self) -> Option<(u8, u8)> {
        let least_significant_byte = self.get_data(1, &[MidiMessageStatus::PitchWheel])?;
        let most_significant_byte = self.get_data(2, &[MidiMessageStatus::PitchWheel])?;
        Some((least_significant_byte, most_significant_byte))
    }
}

impl Display for MidiMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
