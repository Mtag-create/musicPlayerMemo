use std::fmt;
use std::io;

#[derive(Debug)]
pub enum AnalyzerError {
    FileNotFound,
    UnsupportedFormat,
    InvalidWav,
    UnsupportedBitDepth,
    Serialization(String),
    Io(io::Error),
}

impl From<rmp_serde::encode::Error> for AnalyzerError {
    fn from(error: rmp_serde::encode::Error) -> Self {
        AnalyzerError::Serialization(error.to_string())
    }
}

impl From<rmp_serde::decode::Error> for AnalyzerError {
    fn from(error: rmp_serde::decode::Error) -> Self {
        AnalyzerError::Serialization(error.to_string())
    }
}

impl From<io::Error> for AnalyzerError {
    fn from(error: io::Error) -> Self {
        AnalyzerError::Io(error)
    }
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalyzerError::FileNotFound =>
                write!(f, "File not found"),
            AnalyzerError::UnsupportedFormat =>
                write!(f, "Unsupported file format"),
            AnalyzerError::InvalidWav =>
                write!(f, "Invalid WAV file"),
            AnalyzerError::UnsupportedBitDepth =>
                write!(f, "Unsupported bit depth"),
            AnalyzerError::Io(error) =>
                write!(f, "I/O error: {}", error),
            AnalyzerError::Serialization(message) =>
                write!(f, "JSON serialization error: {}", message),
        }
    }
}