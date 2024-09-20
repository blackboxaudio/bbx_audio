pub type Result<T> = std::result::Result<T, BbxAudioMidiError>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum BbxAudioMidiError {
    #[error("missing MIDI input port")]
    MissingMidiInputPort
}
