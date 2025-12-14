//! Input source detection.

use crate::cli::InputSpec;
use crate::error::{Error, Result};
use std::io::IsTerminal;
use std::path::PathBuf;

/// Resolved input source ready for reading.
#[derive(Debug)]
pub enum InputSource {
    /// Read from a file path.
    File(PathBuf),
    /// Hex string already decoded to bytes.
    Bytes(Vec<u8>),
    /// Read from stdin.
    Stdin,
}

impl InputSource {
    /// Create an InputSource from an InputSpec.
    pub fn from_spec(spec: &InputSpec) -> Result<Self> {
        match spec {
            InputSpec::Stdin => {
                // Check if stdin is a terminal (interactive mode with no piped input)
                if std::io::stdin().is_terminal() {
                    return Err(Error::NoInput);
                }
                Ok(InputSource::Stdin)
            }

            InputSpec::File(path) => {
                if !path.exists() {
                    return Err(Error::FileNotFound(path.clone()));
                }
                Ok(InputSource::File(path.clone()))
            }

            InputSpec::Hex(hex_str) => {
                let bytes = hex::decode(hex_str)?;
                Ok(InputSource::Bytes(bytes))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_input() {
        let spec = InputSpec::Hex("84a400".to_string());
        let source = InputSource::from_spec(&spec).unwrap();
        match source {
            InputSource::Bytes(b) => assert_eq!(b, vec![0x84, 0xa4, 0x00]),
            _ => panic!("Expected Bytes"),
        }
    }

    #[test]
    fn test_invalid_hex() {
        let spec = InputSpec::Hex("not_hex".to_string());
        assert!(InputSource::from_spec(&spec).is_err());
    }

    #[test]
    fn test_file_not_found() {
        let spec = InputSpec::File(PathBuf::from("/nonexistent/file.cbor"));
        let result = InputSource::from_spec(&spec);
        assert!(matches!(result, Err(Error::FileNotFound(_))));
    }
}
