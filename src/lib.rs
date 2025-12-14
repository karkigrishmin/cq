//! cq - CBOR Query Tool for Cardano transactions.
//!
//! A CLI tool for inspecting and querying Cardano transactions encoded in CBOR format.
//! Think of it as "jq for Cardano CBOR."
//!
//! # Features
//!
//! - Parse CBOR transactions from file, hex string, or stdin
//! - Auto-detect input format
//! - Query specific fields with dot notation (e.g., `outputs.0.address`)
//! - Support wildcards (e.g., `outputs.*.address`)
//! - Format addresses as bech32
//! - Pretty terminal output with colors
//! - JSON output for piping
//! - Validation mode with exit codes

pub mod cli;
pub mod decode;
pub mod error;
pub mod format;
pub mod input;
pub mod query;

pub use cli::Args;
pub use error::{Error, Result};

use decode::decode_transaction;
use format::format_output;
use input::read_input;
use query::execute_query;

/// Run cq with the given arguments.
pub fn run(args: &Args) -> Result<()> {
    // Resolve query and input from positional arguments
    let (query_opt, input_spec) = args.resolve();

    // Read input bytes
    let bytes = read_input(&input_spec)?;

    // Decode the transaction
    let tx = decode_transaction(&bytes)?;

    // Check mode: just validate and exit
    if args.check {
        // Transaction decoded successfully
        return Ok(());
    }

    // Execute query - use empty string for full transaction
    let query = query_opt.unwrap_or("");
    let result = execute_query(&tx, query)?;

    // Format and print output
    let output = format_output(&result, args)?;
    println!("{}", output);

    Ok(())
}
