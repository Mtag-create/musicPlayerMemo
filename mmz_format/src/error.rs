//! mmz_format独自のエラー型。
//! MyMusicAnalyzerのAnalyzerErrorとは独立しており、
//! 利用側で必要に応じて変換する。

use std::fmt;

#[derive(Debug)]
pub enum MmzError {
    Io(std::io::Error),
    Serialization(String),
}

impl fmt::Display for MmzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MmzError::Io(e) => write!(f, "I/O error: {}", e),
            MmzError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for MmzError {}

impl From<std::io::Error> for MmzError {
    fn from(e: std::io::Error) -> Self {
        MmzError::Io(e)
    }
}