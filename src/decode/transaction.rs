//! Transaction decoding with CML.

use crate::error::{Error, Result};
use cml_chain::auxdata::AuxiliaryData;
use cml_chain::transaction::{Transaction, TransactionBody, TransactionWitnessSet};
use cml_core::serialization::Deserialize;
use cml_crypto::TransactionHash;

/// A decoded Cardano transaction with preserved original bytes.
#[derive(Debug)]
pub struct DecodedTransaction {
    /// The parsed CML transaction.
    pub tx: Transaction,
    /// Original CBOR bytes (preserved for hash computation).
    pub original_bytes: Vec<u8>,
    /// Computed transaction hash.
    pub hash: TransactionHash,
}

impl DecodedTransaction {
    /// Access the transaction body.
    pub fn body(&self) -> &TransactionBody {
        &self.tx.body
    }

    /// Access the witness set.
    pub fn witness_set(&self) -> &TransactionWitnessSet {
        &self.tx.witness_set
    }

    /// Access auxiliary data (metadata).
    pub fn auxiliary_data(&self) -> Option<&AuxiliaryData> {
        self.tx.auxiliary_data.as_ref()
    }

    /// Check if the transaction is marked as valid.
    pub fn is_valid(&self) -> bool {
        self.tx.is_valid
    }
}

/// Decode a transaction from CBOR bytes.
pub fn decode_transaction(bytes: &[u8]) -> Result<DecodedTransaction> {
    // Use CML to deserialize the transaction
    let tx = Transaction::from_cbor_bytes(bytes).map_err(|e| Error::DecodeFailed(e.to_string()))?;

    // Compute transaction hash from body
    // CML's TransactionBody::hash() computes blake2b_256 of the body bytes
    let hash = tx.body.hash();

    Ok(DecodedTransaction {
        tx,
        original_bytes: bytes.to_vec(),
        hash,
    })
}

#[cfg(test)]
mod tests {
    // Tests will be added once we have real transaction fixtures
}
