//! CBOR decoding module with CML integration.

mod address;
mod transaction;

pub use address::{DecodedAddress, decode_address};
pub use transaction::{DecodedTransaction, decode_transaction};
