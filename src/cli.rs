//! CLI argument parsing for cq.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// CBOR Query Tool for Cardano transactions.
///
/// Inspect and query Cardano transactions encoded in CBOR format.
/// Think of it as "jq for Cardano CBOR."
#[derive(Parser, Debug)]
#[command(
    name = "cq",
    version,
    about = "CBOR Query Tool for Cardano transactions",
    after_help = r#"EXAMPLES:
    cq tx.cbor                     Show full transaction (pretty)
    cq 84a400...                   Auto-detect hex input
    cat tx.cbor | cq               Read from stdin
    cq fee tx.cbor                 Query specific field
    cq fee tx.cbor --ada           Show fee in ADA
    cq outputs.0.address tx.cbor   Nested field access
    cq outputs.*.address tx.cbor   Wildcard (all addresses)
    cq tx.cbor --json              JSON output
    cq tx.cbor --check             Validate only (exit code)
    cq addr addr1q8mnd...          Decode any Cardano address

QUERY SHORTCUTS:
    fee        → body.fee
    inputs     → body.inputs
    outputs    → body.outputs
    metadata   → auxiliary_data.metadata
    witnesses  → witness_set
    hash       → (computed transaction hash)"#
)]
pub struct Args {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Query path or input (file path / hex string).
    /// If one argument: treated as input.
    /// If two arguments: first is query, second is input.
    #[arg(value_name = "QUERY_OR_INPUT")]
    pub first: Option<String>,

    /// Input file or hex string when query is provided.
    #[arg(value_name = "INPUT")]
    pub second: Option<String>,

    /// Output as JSON.
    #[arg(long, short = 'j')]
    pub json: bool,

    /// Output raw CBOR diagnostic notation.
    #[arg(long, short = 'r')]
    pub raw: bool,

    /// Display ADA amounts instead of lovelace.
    #[arg(long, short = 'a')]
    pub ada: bool,

    /// Validate only (exit code indicates result: 0=valid, 1=invalid).
    #[arg(long, short = 'c')]
    pub check: bool,

    /// Disable colored output.
    #[arg(long)]
    pub no_color: bool,
}

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Decode and display a Cardano address.
    ///
    /// Parses a bech32 Cardano address and shows its components including
    /// type (base, enterprise, reward, pointer, byron), network,
    /// and payment/stake credentials.
    #[command(name = "addr")]
    Address {
        /// The bech32 address to decode (e.g., addr1..., stake1..., addr_test1...).
        address: String,

        /// Output as JSON.
        #[arg(long, short = 'j')]
        json: bool,
    },

    /// Check for updates and show upgrade instructions.
    ///
    /// Queries crates.io for the latest version and displays
    /// upgrade instructions if a newer version is available.
    #[command(name = "update")]
    Update,
}

/// Specifies how to obtain input bytes.
#[derive(Debug, Clone)]
pub enum InputSpec {
    /// Read from stdin.
    Stdin,
    /// Read from a file path.
    File(PathBuf),
    /// Parse hex string directly.
    Hex(String),
}

impl Args {
    /// Resolve the query and input from positional arguments.
    ///
    /// Returns (optional query path, input specification).
    pub fn resolve(&self) -> (Option<&str>, InputSpec) {
        match (&self.first, &self.second) {
            // No arguments: read from stdin, no query
            (None, None) => (None, InputSpec::Stdin),

            // One argument: could be query (with stdin) or input
            (Some(first), None) => {
                if Self::looks_like_query(first) {
                    (Some(first.as_str()), InputSpec::Stdin)
                } else {
                    (None, InputSpec::detect(first))
                }
            }

            // Two arguments: first is query, second is input
            (Some(query), Some(input)) => (Some(query.as_str()), InputSpec::detect(input)),

            // This case shouldn't happen with clap
            (None, Some(_)) => unreachable!(),
        }
    }

    /// Heuristic to determine if a string looks like a query path.
    fn looks_like_query(s: &str) -> bool {
        // Known shortcuts
        let shortcuts = [
            "fee",
            "inputs",
            "outputs",
            "metadata",
            "witnesses",
            "hash",
            "ttl",
            "mint",
            "certs",
            "withdrawals",
            "collateral",
        ];

        if shortcuts.contains(&s) {
            return true;
        }

        // Exclude common file extensions before checking for dots
        let file_extensions = [".cbor", ".bin", ".hex", ".raw", ".tx", ".json"];
        for ext in file_extensions {
            if s.ends_with(ext) {
                return false;
            }
        }

        // Dot notation or wildcard patterns
        if s.contains('.') || s.contains('*') {
            return true;
        }

        // Starts with "body." or "witness_set." etc.
        if s.starts_with("body") || s.starts_with("auxiliary") || s.starts_with("witness") {
            return true;
        }

        false
    }
}

impl InputSpec {
    /// Detect input type from a string argument.
    pub fn detect(s: &str) -> Self {
        // Strip optional 0x prefix for hex detection
        let hex_candidate = s.strip_prefix("0x").unwrap_or(s);

        // Check if it looks like hex:
        // - All characters are hex digits
        // - Reasonable length (at least 8 chars for minimal CBOR)
        // - Starts with valid CBOR transaction tag (84 for 4-element array)
        if hex_candidate.len() >= 8
            && hex_candidate.chars().all(|c| c.is_ascii_hexdigit())
            && hex_candidate.starts_with("84")
        {
            return InputSpec::Hex(hex_candidate.to_string());
        }

        // Otherwise treat as file path
        InputSpec::File(PathBuf::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_query_shortcuts() {
        assert!(Args::looks_like_query("fee"));
        assert!(Args::looks_like_query("inputs"));
        assert!(Args::looks_like_query("outputs"));
        assert!(Args::looks_like_query("hash"));
    }

    #[test]
    fn test_looks_like_query_dot_notation() {
        assert!(Args::looks_like_query("body.fee"));
        assert!(Args::looks_like_query("outputs.0.address"));
        assert!(Args::looks_like_query("outputs.*.value"));
    }

    #[test]
    fn test_looks_like_query_file_paths() {
        assert!(!Args::looks_like_query("tx.cbor"));
        assert!(!Args::looks_like_query("/path/to/file.cbor"));
        assert!(!Args::looks_like_query("transaction.bin"));
    }

    #[test]
    fn test_input_spec_detect_hex() {
        match InputSpec::detect("84a4000081") {
            InputSpec::Hex(s) => assert_eq!(s, "84a4000081"),
            _ => panic!("Expected Hex"),
        }
    }

    #[test]
    fn test_input_spec_detect_hex_with_prefix() {
        match InputSpec::detect("0x84a400abc123") {
            InputSpec::Hex(s) => assert_eq!(s, "84a400abc123"),
            _ => panic!("Expected Hex"),
        }
    }

    #[test]
    fn test_input_spec_detect_file() {
        match InputSpec::detect("tx.cbor") {
            InputSpec::File(p) => assert_eq!(p, PathBuf::from("tx.cbor")),
            _ => panic!("Expected File"),
        }
    }
}
