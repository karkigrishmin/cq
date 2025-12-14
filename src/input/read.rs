//! Input reading implementation.

use crate::cli::InputSpec;
use crate::error::{Error, Result};
use crate::input::InputSource;
use std::fs;
use std::io::{self, Read};

/// Read input bytes from the specified source.
pub fn read_input(spec: &InputSpec) -> Result<Vec<u8>> {
    let source = InputSource::from_spec(spec)?;

    match source {
        InputSource::File(path) => fs::read(&path).map_err(|e| Error::IoError {
            path: Some(path),
            source: e,
        }),

        InputSource::Bytes(bytes) => Ok(bytes),

        InputSource::Stdin => {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|e| Error::IoError {
                    path: None,
                    source: e,
                })?;

            // Try to detect if stdin is hex-encoded or binary CBOR
            detect_and_decode_stdin(buffer)
        }
    }
}

/// Detect if stdin content is hex-encoded and decode if necessary.
fn detect_and_decode_stdin(buffer: Vec<u8>) -> Result<Vec<u8>> {
    // Try to interpret as UTF-8 text
    let Ok(text) = String::from_utf8(buffer.clone()) else {
        // Not valid UTF-8, assume binary CBOR
        return Ok(buffer);
    };

    let trimmed = text.trim();

    // Empty input
    if trimmed.is_empty() {
        return Err(Error::NoInput);
    }

    // Strip optional 0x prefix
    let hex_candidate = trimmed.strip_prefix("0x").unwrap_or(trimmed);

    // Check if it looks like hex input
    // Use >=4 chars (2 bytes) as minimum - reasonable for hex piped to stdin
    if hex_candidate.chars().all(|c| c.is_ascii_hexdigit()) && hex_candidate.len() >= 4 {
        // Decode as hex
        hex::decode(hex_candidate).map_err(Error::from)
    } else {
        // Assume binary CBOR (the original bytes)
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hex_stdin() {
        let input = b"84a400".to_vec();
        let result = detect_and_decode_stdin(input).unwrap();
        assert_eq!(result, vec![0x84, 0xa4, 0x00]);
    }

    #[test]
    fn test_detect_hex_stdin_with_prefix() {
        let input = b"0x84a400".to_vec();
        let result = detect_and_decode_stdin(input).unwrap();
        assert_eq!(result, vec![0x84, 0xa4, 0x00]);
    }

    #[test]
    fn test_detect_hex_stdin_with_whitespace() {
        let input = b"  84a400  \n".to_vec();
        let result = detect_and_decode_stdin(input).unwrap();
        assert_eq!(result, vec![0x84, 0xa4, 0x00]);
    }

    #[test]
    fn test_detect_binary_stdin() {
        // Binary data that's not valid UTF-8
        let input = vec![0x84, 0xa4, 0x00, 0xff];
        let result = detect_and_decode_stdin(input.clone()).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_empty_stdin() {
        let input = b"   ".to_vec();
        let result = detect_and_decode_stdin(input);
        assert!(matches!(result, Err(Error::NoInput)));
    }
}
