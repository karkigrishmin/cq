//! Query path parsing.

use crate::error::{Error, Result};

/// A segment in a query path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    /// Named field access (e.g., "body", "fee").
    Field(String),
    /// Array index access (e.g., "0", "1").
    Index(usize),
    /// Wildcard for all array elements (e.g., "*").
    Wildcard,
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
    pub fn parse(input: &str) -> Result<Self> {
        if input.is_empty() {
            return Ok(QueryPath { segments: vec![] });
        }

        let segments = input
            .split('.')
            .map(Self::parse_segment)
            .collect::<Result<Vec<_>>>()?;

        Ok(QueryPath { segments })
    }

    /// Parse a single path segment.
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

    /// Check if this path contains any wildcards.
    pub fn has_wildcard(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, PathSegment::Wildcard))
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
}
