use std::fmt::{Display, Formatter};

pub struct MidiMessage {
    bytes: [u8; 3],
}

impl MidiMessage {
    pub fn new(bytes: [u8; 3]) -> Self {
        MidiMessage {
            bytes,
        }
    }
}

impl Display for MidiMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status: {}\nNote: {}\nVelocity: {}\n", self.bytes[0], self.bytes[1], self.bytes[2])
    }
}

impl From<&[u8]> for MidiMessage {
    fn from(value: &[u8]) -> Self {
        if value.len() == 3 {
            MidiMessage {
                bytes: [value[0], value[1], value[2]],
            }
        } else {
            MidiMessage {
                bytes: [0; 3],
            }
        }
    }
}
