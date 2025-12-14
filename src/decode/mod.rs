//! CBOR decoding module with CML integration.

mod address;
mod transaction;

pub use address::{decode_address, DecodedAddress};
pub use transaction::{DecodedTransaction, decode_transaction};
