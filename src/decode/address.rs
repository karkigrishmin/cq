//! Standalone address decoding for Cardano addresses.

use crate::error::{Error, Result};
use cml_chain::address::Address;
use cml_chain::certs::Credential;
use cml_core::serialization::ToBytes;
use cml_crypto::RawBytesEncoding;
use serde_json::Value as JsonValue;

/// Decoded address with all components.
pub struct DecodedAddress {
    /// The original bech32 string.
    pub bech32: String,
    /// The address type.
    pub address_type: AddressType,
    /// Network (mainnet or testnet).
    pub network: Network,
    /// Payment credential (if applicable).
    pub payment_credential: Option<DecodedCredential>,
    /// Stake credential (if applicable).
    pub stake_credential: Option<DecodedCredential>,
    /// Pointer info for pointer addresses.
    pub pointer: Option<Pointer>,
}

/// Address type enumeration.
#[derive(Debug, Clone, Copy)]
pub enum AddressType {
    Base,
    Enterprise,
    Reward,
    Pointer,
    Byron,
}

impl AddressType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AddressType::Base => "base",
            AddressType::Enterprise => "enterprise",
            AddressType::Reward => "reward",
            AddressType::Pointer => "pointer",
            AddressType::Byron => "byron",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AddressType::Base => "Base (Shelley)",
            AddressType::Enterprise => "Enterprise (Shelley, no staking)",
            AddressType::Reward => "Reward/Stake",
            AddressType::Pointer => "Pointer (Shelley)",
            AddressType::Byron => "Byron (Legacy)",
        }
    }
}

/// Network enumeration.
#[derive(Debug, Clone, Copy)]
pub enum Network {
    Mainnet,
    Testnet,
    Unknown,
}

impl Network {
    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
            Network::Unknown => "unknown",
        }
    }
}

/// Decoded credential.
pub struct DecodedCredential {
    /// Type of credential (keyhash or scripthash).
    pub cred_type: CredentialType,
    /// Hash in hex.
    pub hash: String,
}

/// Credential type.
#[derive(Debug, Clone, Copy)]
pub enum CredentialType {
    KeyHash,
    ScriptHash,
}

impl CredentialType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CredentialType::KeyHash => "keyhash",
            CredentialType::ScriptHash => "scripthash",
        }
    }
}

/// Pointer info for pointer addresses.
pub struct Pointer {
    pub slot: u64,
    pub tx_index: u64,
    pub cert_index: u64,
}

/// Decode a bech32 Cardano address.
pub fn decode_address(addr_str: &str) -> Result<DecodedAddress> {
    let addr = Address::from_bech32(addr_str)
        .map_err(|e| Error::DecodeFailed(format!("Invalid address: {}", e)))?;

    let bech32 = addr_str.to_string();

    // Detect network from header byte (CIP-19)
    // Network ID is encoded in bit 0 of the header byte for Shelley addresses
    // - 0 = testnet (covers preprod, preview, and all other testnets)
    // - 1 = mainnet
    // Note: Cannot distinguish between different testnets from address alone
    let raw_bytes = addr.to_raw_bytes();
    let network = if !raw_bytes.is_empty() {
        let header = raw_bytes[0];
        match header & 0x01 {
            0 => Network::Testnet,
            1 => Network::Mainnet,
            _ => unreachable!(),
        }
    } else {
        Network::Unknown
    };

    match addr {
        Address::Base(base_addr) => Ok(DecodedAddress {
            bech32,
            address_type: AddressType::Base,
            network,
            payment_credential: Some(decode_credential(&base_addr.payment)),
            stake_credential: Some(decode_credential(&base_addr.stake)),
            pointer: None,
        }),
        Address::Enterprise(enterprise_addr) => Ok(DecodedAddress {
            bech32,
            address_type: AddressType::Enterprise,
            network,
            payment_credential: Some(decode_credential(&enterprise_addr.payment)),
            stake_credential: None,
            pointer: None,
        }),
        Address::Ptr(ptr_addr) => Ok(DecodedAddress {
            bech32,
            address_type: AddressType::Pointer,
            network,
            payment_credential: Some(decode_credential(&ptr_addr.payment)),
            stake_credential: None,
            pointer: Some(Pointer {
                slot: ptr_addr.stake.slot(),
                tx_index: ptr_addr.stake.tx_index(),
                cert_index: ptr_addr.stake.cert_index(),
            }),
        }),
        Address::Reward(reward_addr) => Ok(DecodedAddress {
            bech32,
            address_type: AddressType::Reward,
            network,
            payment_credential: None,
            stake_credential: Some(decode_credential(&reward_addr.payment)),
            pointer: None,
        }),
        Address::Byron(byron_addr) => Ok(DecodedAddress {
            bech32: hex::encode(byron_addr.to_bytes()),
            address_type: AddressType::Byron,
            network,
            payment_credential: None,
            stake_credential: None,
            pointer: None,
        }),
    }
}

/// Decode a credential to our format.
fn decode_credential(cred: &Credential) -> DecodedCredential {
    match cred {
        Credential::PubKey { hash, .. } => DecodedCredential {
            cred_type: CredentialType::KeyHash,
            hash: hex::encode(hash.to_raw_bytes()),
        },
        Credential::Script { hash, .. } => DecodedCredential {
            cred_type: CredentialType::ScriptHash,
            hash: hex::encode(hash.to_raw_bytes()),
        },
    }
}

impl DecodedAddress {
    /// Convert to JSON.
    pub fn to_json(&self) -> JsonValue {
        let mut json = serde_json::json!({
            "address": self.bech32,
            "type": self.address_type.as_str(),
            "network": self.network.as_str()
        });

        if let Some(ref payment) = self.payment_credential {
            json["payment_credential"] = serde_json::json!({
                "type": payment.cred_type.as_str(),
                "hash": payment.hash
            });
        }

        if let Some(ref stake) = self.stake_credential {
            json["stake_credential"] = serde_json::json!({
                "type": stake.cred_type.as_str(),
                "hash": stake.hash
            });
        }

        if let Some(ref ptr) = self.pointer {
            json["pointer"] = serde_json::json!({
                "slot": ptr.slot,
                "tx_index": ptr.tx_index,
                "cert_index": ptr.cert_index
            });
        }

        json
    }

    /// Format as pretty string for terminal output.
    pub fn to_pretty(&self, use_color: bool) -> String {
        use colored::Colorize;

        let mut output = String::new();

        // Title
        if use_color {
            output.push_str(&format!("{}\n", "Address Details".bold().cyan()));
        } else {
            output.push_str("Address Details\n");
        }

        // Bech32
        if use_color {
            output.push_str(&format!("  {}: {}\n", "Address".bold(), self.bech32));
        } else {
            output.push_str(&format!("  Address: {}\n", self.bech32));
        }

        // Type
        if use_color {
            output.push_str(&format!(
                "  {}: {}\n",
                "Type".bold(),
                self.address_type.description().green()
            ));
        } else {
            output.push_str(&format!("  Type: {}\n", self.address_type.description()));
        }

        // Network
        let network_str = self.network.as_str();
        if use_color {
            let colored_network = if matches!(self.network, Network::Mainnet) {
                network_str.yellow()
            } else {
                network_str.blue()
            };
            output.push_str(&format!("  {}: {}\n", "Network".bold(), colored_network));
        } else {
            output.push_str(&format!("  Network: {}\n", network_str));
        }

        // Payment credential
        if let Some(ref payment) = self.payment_credential {
            if use_color {
                output.push_str(&format!(
                    "  {}: {} {}\n",
                    "Payment".bold(),
                    payment.cred_type.as_str().cyan(),
                    payment.hash.dimmed()
                ));
            } else {
                output.push_str(&format!(
                    "  Payment: {} {}\n",
                    payment.cred_type.as_str(),
                    payment.hash
                ));
            }
        }

        // Stake credential
        if let Some(ref stake) = self.stake_credential {
            if use_color {
                output.push_str(&format!(
                    "  {}: {} {}\n",
                    "Stake".bold(),
                    stake.cred_type.as_str().cyan(),
                    stake.hash.dimmed()
                ));
            } else {
                output.push_str(&format!(
                    "  Stake: {} {}\n",
                    stake.cred_type.as_str(),
                    stake.hash
                ));
            }
        }

        // Pointer
        if let Some(ref ptr) = self.pointer {
            if use_color {
                output.push_str(&format!(
                    "  {}: slot={}, tx={}, cert={}\n",
                    "Pointer".bold(),
                    ptr.slot,
                    ptr.tx_index,
                    ptr.cert_index
                ));
            } else {
                output.push_str(&format!(
                    "  Pointer: slot={}, tx={}, cert={}\n",
                    ptr.slot, ptr.tx_index, ptr.cert_index
                ));
            }
        }

        output
    }
}
