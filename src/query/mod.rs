//! Query engine module for dot-notation queries.

mod engine;
mod path;
mod shortcuts;

pub use engine::{QueryResult, QueryValue, execute_query};
pub use path::{PathSegment, QueryPath};
pub use shortcuts::expand_shortcut;
