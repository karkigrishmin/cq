//! Query path parsing.

use crate::error::{Error, Result};

/// A segment in a query path.
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// Named field access (e.g., "body", "fee").
    Field(String),
    /// Array index access (e.g., "0", "1").
    Index(usize),
    /// Wildcard for all array elements (e.g., "*").
    Wildcard,
    /// Filter expression (e.g., "[value.coin > 1000000]").
    Filter(FilterExpr),
}

/// A filter expression for array filtering.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterExpr {
    /// Field path to compare (dot-notation within the element).
    pub field: String,
    /// Comparison operator.
    pub op: FilterOp,
    /// Value to compare against.
    pub value: FilterValue,
}

/// Filter comparison operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterOp {
    /// Greater than (>).
    Gt,
    /// Less than (<).
    Lt,
    /// Greater than or equal (>=).
    Gte,
    /// Less than or equal (<=).
    Lte,
    /// Equal (==).
    Eq,
    /// Not equal (!=).
    Ne,
    /// String contains (~).
    Contains,
}

/// Filter comparison value.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    /// Numeric value.
    Number(f64),
    /// String value.
    String(String),
    /// Null (for existence checks).
    Null,
}

/// A parsed query path.
#[derive(Debug, Clone)]
pub struct QueryPath {
    /// The segments that make up this path.
    pub segments: Vec<PathSegment>,
}

impl QueryPath {
    /// Parse a dot-notation query path.
    ///
    /// # Examples
    ///
    /// - `"body.fee"` → `[Field("body"), Field("fee")]`
    /// - `"outputs.0.address"` → `[Field("outputs"), Index(0), Field("address")]`
    /// - `"outputs.*.value"` → `[Field("outputs"), Wildcard, Field("value")]`
    /// - `"outputs[value.coin > 1000000]"` → `[Field("outputs"), Filter(...)]`
    pub fn parse(input: &str) -> Result<Self> {
        if input.is_empty() {
            return Ok(QueryPath { segments: vec![] });
        }

        let mut segments = Vec::new();
        let mut remaining = input;

        while !remaining.is_empty() {
            // Check for filter syntax: field[filter]
            if let Some(bracket_start) = remaining.find('[') {
                // Parse field name before bracket
                let field_part = &remaining[..bracket_start];
                if !field_part.is_empty() {
                    // Handle dot-separated fields before the filter
                    for part in field_part.split('.') {
                        if !part.is_empty() {
                            segments.push(Self::parse_segment(part)?);
                        }
                    }
                }

                // Find matching closing bracket
                let bracket_end = remaining
                    .find(']')
                    .ok_or_else(|| Error::InvalidQuery("Unclosed bracket in filter".to_string()))?;

                // Parse filter expression
                let filter_str = &remaining[bracket_start + 1..bracket_end];
                let filter = Self::parse_filter(filter_str)?;
                segments.push(PathSegment::Filter(filter));

                // Continue with rest after bracket
                remaining = &remaining[bracket_end + 1..];
                if remaining.starts_with('.') {
                    remaining = &remaining[1..];
                }
            } else {
                // No more filters, parse remaining as dot-notation
                let parts: Vec<&str> = remaining.split('.').collect();
                for (i, part) in parts.iter().enumerate() {
                    if part.is_empty() {
                        // Allow trailing empty (e.g., from "foo.") but not consecutive dots
                        if i < parts.len() - 1 {
                            return Err(Error::InvalidQuery(
                                "Empty path segment (consecutive dots?)".to_string(),
                            ));
                        }
                    } else {
                        segments.push(Self::parse_segment(part)?);
                    }
                }
                break;
            }
        }

        Ok(QueryPath { segments })
    }

    /// Parse a single path segment (without filter).
    fn parse_segment(s: &str) -> Result<PathSegment> {
        if s.is_empty() {
            return Err(Error::InvalidQuery(
                "Empty path segment (consecutive dots?)".to_string(),
            ));
        }

        // Wildcard
        if s == "*" {
            return Ok(PathSegment::Wildcard);
        }

        // Try to parse as array index
        if let Ok(idx) = s.parse::<usize>() {
            return Ok(PathSegment::Index(idx));
        }

        // Otherwise it's a field name
        Ok(PathSegment::Field(s.to_string()))
    }

    /// Parse a filter expression inside brackets.
    /// Syntax: `field.path op value`
    /// Examples: `value.coin > 1000000`, `address ~ "addr1"`, `datum != null`
    fn parse_filter(s: &str) -> Result<FilterExpr> {
        let s = s.trim();

        // Find operator (order matters: >= before >, etc.)
        let ops = [
            (">=", FilterOp::Gte),
            ("<=", FilterOp::Lte),
            ("!=", FilterOp::Ne),
            ("==", FilterOp::Eq),
            (">", FilterOp::Gt),
            ("<", FilterOp::Lt),
            ("~", FilterOp::Contains),
        ];

        for (op_str, op) in ops {
            if let Some(pos) = s.find(op_str) {
                let field = s[..pos].trim().to_string();
                let value_str = s[pos + op_str.len()..].trim();

                if field.is_empty() {
                    return Err(Error::InvalidQuery("Filter field is empty".to_string()));
                }

                let value = Self::parse_filter_value(value_str)?;

                return Ok(FilterExpr { field, op, value });
            }
        }

        Err(Error::InvalidQuery(format!(
            "Invalid filter syntax: '{}'. Expected: field op value",
            s
        )))
    }

    /// Parse a filter value (number, string, or null).
    fn parse_filter_value(s: &str) -> Result<FilterValue> {
        let s = s.trim();

        // Null
        if s == "null" {
            return Ok(FilterValue::Null);
        }

        // Quoted string
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            let inner = &s[1..s.len() - 1];
            return Ok(FilterValue::String(inner.to_string()));
        }

        // Try number
        if let Ok(n) = s.parse::<f64>() {
            return Ok(FilterValue::Number(n));
        }

        // Treat as unquoted string
        Ok(FilterValue::String(s.to_string()))
    }

    /// Check if this path contains any wildcards.
    pub fn has_wildcard(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, PathSegment::Wildcard))
    }

    /// Check if this path contains any filters.
    pub fn has_filter(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, PathSegment::Filter(_)))
    }

    /// Check if this path has a filter followed by more segments.
    /// This requires recursive execution since filters return arrays.
    pub fn has_filter_with_continuation(&self) -> bool {
        for (i, segment) in self.segments.iter().enumerate() {
            if matches!(segment, PathSegment::Filter(_)) && i < self.segments.len() - 1 {
                return true;
            }
        }
        false
    }

    /// Check if this path is empty (no segments).
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_path() {
        let path = QueryPath::parse("body.fee").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0], PathSegment::Field("body".into()));
        assert_eq!(path.segments[1], PathSegment::Field("fee".into()));
    }

    #[test]
    fn test_parse_with_index() {
        let path = QueryPath::parse("outputs.0.address").unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[0], PathSegment::Field("outputs".into()));
        assert_eq!(path.segments[1], PathSegment::Index(0));
        assert_eq!(path.segments[2], PathSegment::Field("address".into()));
    }

    #[test]
    fn test_parse_with_wildcard() {
        let path = QueryPath::parse("outputs.*.address").unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[0], PathSegment::Field("outputs".into()));
        assert_eq!(path.segments[1], PathSegment::Wildcard);
        assert_eq!(path.segments[2], PathSegment::Field("address".into()));
        assert!(path.has_wildcard());
    }

    #[test]
    fn test_parse_single_field() {
        let path = QueryPath::parse("fee").unwrap();
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0], PathSegment::Field("fee".into()));
    }

    #[test]
    fn test_parse_empty() {
        let path = QueryPath::parse("").unwrap();
        assert!(path.is_empty());
    }

    #[test]
    fn test_parse_consecutive_dots_error() {
        let result = QueryPath::parse("body..fee");
        assert!(result.is_err());
    }

    #[test]
    fn test_has_wildcard() {
        assert!(!QueryPath::parse("body.fee").unwrap().has_wildcard());
        assert!(QueryPath::parse("outputs.*").unwrap().has_wildcard());
    }

    #[test]
    fn test_parse_filter_gt() {
        let path = QueryPath::parse("outputs[value.coin > 1000000]").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0], PathSegment::Field("outputs".into()));
        if let PathSegment::Filter(f) = &path.segments[1] {
            assert_eq!(f.field, "value.coin");
            assert_eq!(f.op, FilterOp::Gt);
            assert_eq!(f.value, FilterValue::Number(1000000.0));
        } else {
            panic!("Expected Filter segment");
        }
        assert!(path.has_filter());
    }

    #[test]
    fn test_parse_filter_contains() {
        let path = QueryPath::parse("outputs[address.address ~ \"addr1\"]").unwrap();
        assert_eq!(path.segments.len(), 2);
        if let PathSegment::Filter(f) = &path.segments[1] {
            assert_eq!(f.field, "address.address");
            assert_eq!(f.op, FilterOp::Contains);
            assert_eq!(f.value, FilterValue::String("addr1".into()));
        } else {
            panic!("Expected Filter segment");
        }
    }

    #[test]
    fn test_parse_filter_not_null() {
        let path = QueryPath::parse("outputs[datum != null]").unwrap();
        if let PathSegment::Filter(f) = &path.segments[1] {
            assert_eq!(f.field, "datum");
            assert_eq!(f.op, FilterOp::Ne);
            assert_eq!(f.value, FilterValue::Null);
        } else {
            panic!("Expected Filter segment");
        }
    }

    #[test]
    fn test_parse_filter_with_continuation() {
        let path = QueryPath::parse("outputs[value.coin > 1000000].address").unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[0], PathSegment::Field("outputs".into()));
        assert!(matches!(path.segments[1], PathSegment::Filter(_)));
        assert_eq!(path.segments[2], PathSegment::Field("address".into()));
    }
}
