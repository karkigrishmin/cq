//! Error types for cq.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for cq operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in cq.
#[derive(Error, Debug)]
pub enum Error {
    /// No input was provided (no file, no stdin, no hex).
    #[error("No input provided. Use: cq <file>, cq <hex>, or pipe CBOR to stdin")]
    NoInput,

    /// The specified file was not found.
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// An I/O error occurred.
    #[error("IO error{}: {source}", path.as_ref().map(|p| format!(" reading {}", p.display())).unwrap_or_default())]
    IoError {
        path: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },

    /// Invalid hex input.
    #[error("Invalid hex input: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    /// Failed to decode CBOR/transaction.
    #[error("Failed to decode transaction: {0}")]
    DecodeFailed(String),

    /// Invalid query syntax.
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    /// Requested field was not found.
    #[error("Field not found: '{0}'")]
    FieldNotFound(String),

    /// Array index out of bounds.
    #[error("Index {0} out of bounds")]
    IndexOutOfBounds(usize),

    /// Output formatting error.
    #[error("Format error: {0}")]
    FormatError(String),

    /// Unsupported transaction era.
    #[error("Unsupported era: only Babbage and Conway transactions are supported")]
    UnsupportedEra,

    /// Network error (e.g., when checking for updates).
    #[error("Network error: {0}")]
    NetworkError(String),
}

impl Error {
    /// Get the appropriate exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            // Validation failure (--check mode)
            Error::DecodeFailed(_) | Error::UnsupportedEra => 1,
            // Parse/decode errors
            Error::InvalidHex(_) => 2,
            // I/O errors
            Error::NoInput | Error::FileNotFound(_) | Error::IoError { .. } => 3,
            // Query errors
            Error::InvalidQuery(_) | Error::FieldNotFound(_) | Error::IndexOutOfBounds(_) => 4,
            // Format errors
            Error::FormatError(_) => 5,
            // Network errors (non-fatal for update check)
            Error::NetworkError(_) => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(Error::NoInput.exit_code(), 3);
        assert_eq!(Error::DecodeFailed("test".into()).exit_code(), 1);
        assert_eq!(Error::InvalidQuery("test".into()).exit_code(), 4);
    }

    #[test]
    fn test_error_display() {
        let err = Error::FieldNotFound("fee".into());
        assert_eq!(err.to_string(), "Field not found: 'fee'");
    }
}
