use std::fmt;

/// A specialized [`Result`] type for audio player operations.
pub type Result<T> = std::result::Result<T, PlayerError>;

/// Errors that can occur during audio playback operations.
#[derive(Debug)]
pub enum PlayerError {
    /// No audio output device is available on the system.
    NoOutputDevice,
    /// Failed to initialize the audio output device.
    DeviceInitFailed(String),
    /// An error occurred during audio playback.
    PlaybackFailed(String),
    /// An error from the underlying audio backend (rodio or cpal).
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
