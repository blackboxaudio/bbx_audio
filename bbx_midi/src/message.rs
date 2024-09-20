use std::fmt::{Display, Formatter};

pub struct MidiMessage {
    status: MidiMessageStatus,
    data_1: u8,
    data_2: u8,
}

#[derive(Debug, PartialEq)]
pub enum MidiMessageStatus {
    Unknown,
    NoteOff,
    NoteOn,
    PolyphonicAftertouch,
    ControlChange,
    ProgramChange,
    ChannelAftertouch,
    PitchWheel
}

impl MidiMessage {
    pub fn new(bytes: [u8; 3]) -> Self {
        MidiMessage {
            status: get_midi_message_status(bytes[0]),
            data_1: bytes[1],
            data_2: bytes[2],
        }
    }
}

impl MidiMessage {
    pub fn get_status(self) -> MidiMessageStatus {
        self.status
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

    pub fn get_note(&self) -> Option<u8> {
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
        write!(f, "{:?}, Note = {}, Velocity = {}", self.status, self.get_note().unwrap(), self.get_velocity().unwrap())
    }
}

impl From<&[u8]> for MidiMessage {
    fn from(value: &[u8]) -> Self {
        match value.len() {
            1 => {
                MidiMessage {
                    status: get_midi_message_status(value[0]),
                    data_1: 0,
                    data_2: 0,
                }
            },
            2 => {
                MidiMessage {
                    status: get_midi_message_status(value[0]),
                    data_1: value[1],
                    data_2: 0,
                }
            },
            3 => {
                MidiMessage {
                    status: get_midi_message_status(value[0]),
                    data_1: value[1],
                    data_2: value[2],
                }
            },
            _ => {
                MidiMessage {
                    status: MidiMessageStatus::Unknown,
                    data_1: 0,
                    data_2: 0,
                }
            }
        }
    }
}

fn get_midi_message_status(byte: u8) -> MidiMessageStatus {
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
