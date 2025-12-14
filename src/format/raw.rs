//! Raw CBOR diagnostic notation output.

use crate::error::{Error, Result};
use crate::query::{QueryResult, QueryValue};

/// Format a query result as raw output (CBOR diagnostic notation for bytes).
pub fn format_raw(result: &QueryResult) -> Result<String> {
    match result {
        QueryResult::FullTransaction(json) => {
            // For full transaction, output JSON since we don't have raw CBOR here
            serde_json::to_string_pretty(json).map_err(|e| Error::FormatError(e.to_string()))
        }
        QueryResult::Single(value) => format_value_raw(value),
        QueryResult::Multiple(values) => {
            let formatted: Result<Vec<String>> = values.iter().map(format_value_raw).collect();
            Ok(formatted?.join("\n"))
        }
    }
}

/// Format a single value in raw mode.
fn format_value_raw(value: &QueryValue) -> Result<String> {
    match value {
        QueryValue::Null => Ok("null".to_string()),
        QueryValue::Bool(b) => Ok(b.to_string()),
        QueryValue::Number(n) => Ok(n.to_string()),
        QueryValue::String(s) => {
            // Check if it looks like hex (could be bytes)
            if s.chars().all(|c| c.is_ascii_hexdigit()) && s.len() >= 2 && s.len() % 2 == 0 {
                // Format as CBOR diagnostic bytes notation
                Ok(format!("h'{}'", s))
            } else {
                Ok(format!("\"{}\"", s))
            }
        }
        QueryValue::Array(arr) => {
            let items: Result<Vec<String>> = arr.iter().map(format_value_raw).collect();
            Ok(format!("[{}]", items?.join(", ")))
        }
        QueryValue::Object(map) => {
            let entries: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    let v_str = serde_json::to_string(v).unwrap_or_else(|_| "?".to_string());
                    format!("\"{}\": {}", k, v_str)
                })
                .collect();
            Ok(format!("{{{}}}", entries.join(", ")))
        }
    }
}

/// Convert bytes to CBOR diagnostic notation.
#[allow(dead_code)]
pub fn bytes_to_diagnostic(bytes: &[u8]) -> Result<String> {
    // Try to parse as CBOR and convert to diagnostic notation
    let value: ciborium::Value =
        ciborium::from_reader(bytes).map_err(|e| Error::DecodeFailed(e.to_string()))?;

    Ok(cbor_value_to_diagnostic(&value))
}

/// Convert a ciborium Value to CBOR diagnostic notation.
fn cbor_value_to_diagnostic(value: &ciborium::Value) -> String {
    match value {
        ciborium::Value::Integer(n) => {
            // ciborium::Integer can be converted to i128
            let i: i128 = (*n).into();
            i.to_string()
        }
        ciborium::Value::Bytes(b) => format!("h'{}'", hex::encode(b)),
        ciborium::Value::Text(s) => format!("\"{}\"", s),
        ciborium::Value::Array(items) => {
            let inner: Vec<String> = items.iter().map(cbor_value_to_diagnostic).collect();
            format!("[{}]", inner.join(", "))
        }
        ciborium::Value::Map(entries) => {
            let inner: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}: {}",
                        cbor_value_to_diagnostic(k),
                        cbor_value_to_diagnostic(v)
                    )
                })
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
        ciborium::Value::Tag(tag, inner) => {
            format!("{}({})", tag, cbor_value_to_diagnostic(inner))
        }
        ciborium::Value::Bool(b) => b.to_string(),
        ciborium::Value::Null => "null".to_string(),
        ciborium::Value::Float(f) => format!("{}", f),
        _ => "?".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hex_string() {
        let value = QueryValue::String("84a400".to_string());
        let output = format_value_raw(&value).unwrap();
        assert_eq!(output, "h'84a400'");
    }

    #[test]
    fn test_format_text_string() {
        let value = QueryValue::String("hello world".to_string());
        let output = format_value_raw(&value).unwrap();
        assert_eq!(output, "\"hello world\"");
    }

    #[test]
    fn test_cbor_diagnostic() {
        // Simple CBOR integer
        let cbor = vec![0x18, 0x64]; // Integer 100
        let output = bytes_to_diagnostic(&cbor).unwrap();
        assert_eq!(output, "100");
    }

    #[test]
    fn test_cbor_diagnostic_array() {
        // CBOR array [1, 2, 3]
        let cbor = vec![0x83, 0x01, 0x02, 0x03];
        let output = bytes_to_diagnostic(&cbor).unwrap();
        assert_eq!(output, "[1, 2, 3]");
    }
}
