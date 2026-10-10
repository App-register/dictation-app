pub mod audio;
pub mod stt;
pub mod dictionary;
pub mod formatting;
pub mod output;
pub mod config;

pub use config::Config;
pub use dictionary::Dictionary;
pub use formatting::FormattingEngine;
pub use audio::AudioCapture;
pub use stt::SttEngine;
pub use output::OutputController;

/// Result type alias
pub type Result<T> = std::result::Result<T, DictoError>;

#[derive(thiserror::Error, Debug)]
pub enum DictoError {
    #[error("Audio error: {0}")]
    Audio(String),
    #[error("STT error: {0}")]
    Stt(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Post-processing pipeline result
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub raw_text: String,
    pub processed_text: String,
    pub language: String,
    pub duration_ms: u64,
}