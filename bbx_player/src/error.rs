use std::fmt;

pub type Result<T> = std::result::Result<T, PlayerError>;

#[derive(Debug)]
pub enum PlayerError {
    NoOutputDevice,
    DeviceInitFailed(String),
    PlaybackFailed(String),
    BackendError(String),
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::NoOutputDevice => write!(f, "No audio output device available"),
            PlayerError::DeviceInitFailed(msg) => write!(f, "Failed to initialize audio device: {msg}"),
            PlayerError::PlaybackFailed(msg) => write!(f, "Playback failed: {msg}"),
            PlayerError::BackendError(msg) => write!(f, "Backend error: {msg}"),
        }
    }
}

impl std::error::Error for PlayerError {}
