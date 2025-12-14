//! cq - CBOR Query Tool for Cardano transactions.

use clap::Parser;
use colored::Colorize;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Parse command line arguments
    let args = cq::Args::parse();

    // Disable colors if requested
    if args.no_color {
        colored::control::set_override(false);
    }

    // Run the main logic
    match cq::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Print error message
            eprintln!("{}: {}", "error".red(), e);

            // Return appropriate exit code
            ExitCode::from(e.exit_code() as u8)
        }
    }
}
