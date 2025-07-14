pub type Result<T> = std::result::Result<T, BbxAudioError>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum BbxAudioError {
    #[error("Invalid WAV file")]
    InvalidWavFile,
}
