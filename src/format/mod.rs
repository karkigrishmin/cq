//! Output formatting module.

mod json;
mod pretty;
mod raw;

use crate::cli::Args;
use crate::error::Result;
use crate::query::QueryResult;

pub use json::format_json;
pub use pretty::format_pretty;
pub use raw::format_raw;

/// Format a query result according to the output flags.
pub fn format_output(result: &QueryResult, args: &Args) -> Result<String> {
    if args.json {
        format_json(result)
    } else if args.raw {
        format_raw(result)
    } else {
        format_pretty(result, args)
    }
}
