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
//! - Standalone address decoding

pub mod cli;
pub mod decode;
pub mod error;
pub mod format;
pub mod input;
pub mod query;

pub use cli::{Args, Command};
pub use error::{Error, Result};

use decode::{decode_address, decode_transaction};
use format::format_output;
use input::read_input;
use query::execute_query;

/// Run cq with the given arguments.
pub fn run(args: &Args) -> Result<()> {
    // Handle subcommands first
    if let Some(ref command) = args.command {
        return run_command(command, args);
    }

    // Default behavior: transaction query mode
    run_transaction_mode(args)
}

/// Run a subcommand.
fn run_command(command: &Command, args: &Args) -> Result<()> {
    use std::io::IsTerminal;

    match command {
        Command::Address { address, json } => {
            let decoded = decode_address(address)?;

            if *json {
                let json_output = serde_json::to_string_pretty(&decoded.to_json())
                    .map_err(|e| Error::FormatError(format!("JSON error: {}", e)))?;
                println!("{}", json_output);
            } else {
                let use_color = !args.no_color && std::io::stdout().is_terminal();
                print!("{}", decoded.to_pretty(use_color));
            }

            Ok(())
        }
    }
}

/// Run transaction query mode (default).
fn run_transaction_mode(args: &Args) -> Result<()> {
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
