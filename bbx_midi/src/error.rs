pub type Result<T> = std::result::Result<T, BbxAudioMidiError>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum BbxAudioMidiError {
}
