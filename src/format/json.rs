//! JSON output formatting.

use crate::error::{Error, Result};
use crate::query::QueryResult;

/// Format a query result as JSON.
pub fn format_json(result: &QueryResult) -> Result<String> {
    serde_json::to_string_pretty(result).map_err(|e| Error::FormatError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryValue;

    #[test]
    fn test_format_single_value() {
        let result = QueryResult::Single(QueryValue::String("test".to_string()));
        let output = format_json(&result).unwrap();
        assert_eq!(output.trim(), "\"test\"");
    }

    #[test]
    fn test_format_number() {
        let result = QueryResult::Single(QueryValue::Number(serde_json::Number::from(42)));
        let output = format_json(&result).unwrap();
        assert_eq!(output.trim(), "42");
    }

    #[test]
    fn test_format_multiple() {
        let result = QueryResult::Multiple(vec![
            QueryValue::String("a".to_string()),
            QueryValue::String("b".to_string()),
        ]);
        let output = format_json(&result).unwrap();
        assert!(output.contains("\"a\""));
        assert!(output.contains("\"b\""));
    }
}
