pub enum FileType {
    Wav,
    // Aiff,
    // Mp3,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            // "aiff" => Some(FileType::Aiff),
            // "mp3" => Some(FileType::Mp3),
            "wav" => Some(FileType::Wav),
            _ => None,
        }
    }
}
