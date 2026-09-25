//! algorithmクレート全体で使うエラー型

use std::fmt;

#[derive(Debug)]
pub enum AlgorithmError {
    Mmz(mmz_format::MmzError),
}

impl fmt::Display for AlgorithmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlgorithmError::Mmz(e) => write!(f, "mmz decode error: {}", e),
        }
    }
}

impl std::error::Error for AlgorithmError {}

impl From<mmz_format::MmzError> for AlgorithmError {
    fn from(e: mmz_format::MmzError) -> Self {
        AlgorithmError::Mmz(e)
    }
}