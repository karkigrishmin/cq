//! CBOR decoding module with CML integration.

mod transaction;

pub use transaction::{DecodedTransaction, decode_transaction};
