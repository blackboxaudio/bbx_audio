pub type Result<T> = std::result::Result<T, BbxAudioFileError>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum BbxAudioFileError {
    #[error("invalid WAV file")]
    InvalidWavFile,
}
