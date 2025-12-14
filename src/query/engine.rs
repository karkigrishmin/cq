//! Query execution engine.

use crate::decode::DecodedTransaction;
use crate::error::{Error, Result};
use crate::query::path::{PathSegment, QueryPath};
use crate::query::shortcuts::{expand_shortcut, is_hash_query};
use cml_crypto::RawBytesEncoding;
use serde::Serialize;
use serde_json::Value as JsonValue;

/// Result of a query execution.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum QueryResult {
    /// The full transaction.
    FullTransaction(JsonValue),
    /// A single value.
    Single(QueryValue),
    /// Multiple values (from wildcard expansion).
    Multiple(Vec<QueryValue>),
}

/// A queryable value.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum QueryValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<QueryValue>),
    Object(serde_json::Map<String, JsonValue>),
}

impl From<JsonValue> for QueryValue {
    fn from(value: JsonValue) -> Self {
        match value {
            JsonValue::Null => QueryValue::Null,
            JsonValue::Bool(b) => QueryValue::Bool(b),
            JsonValue::Number(n) => QueryValue::Number(n),
            JsonValue::String(s) => QueryValue::String(s),
            JsonValue::Array(arr) => {
                QueryValue::Array(arr.into_iter().map(QueryValue::from).collect())
            }
            JsonValue::Object(map) => QueryValue::Object(map),
        }
    }
}

impl From<QueryValue> for JsonValue {
    fn from(value: QueryValue) -> Self {
        match value {
            QueryValue::Null => JsonValue::Null,
            QueryValue::Bool(b) => JsonValue::Bool(b),
            QueryValue::Number(n) => JsonValue::Number(n),
            QueryValue::String(s) => JsonValue::String(s),
            QueryValue::Array(arr) => {
                JsonValue::Array(arr.into_iter().map(JsonValue::from).collect())
            }
            QueryValue::Object(map) => JsonValue::Object(map),
        }
    }
}

/// Execute a query against a decoded transaction.
pub fn execute_query(tx: &DecodedTransaction, query: &str) -> Result<QueryResult> {
    // Expand shortcuts first
    let expanded = expand_shortcut(query);

    // Handle special computed fields
    if is_hash_query(&expanded) {
        let hash_hex = hex::encode(tx.hash.to_raw_bytes());
        return Ok(QueryResult::Single(QueryValue::String(hash_hex)));
    }

    // Parse the query path
    let path = QueryPath::parse(&expanded)?;

    // Convert transaction to JSON for querying
    let tx_json = transaction_to_json(tx)?;

    // If path is empty, return full transaction
    if path.is_empty() {
        return Ok(QueryResult::FullTransaction(tx_json));
    }

    // Execute the path query
    if path.has_wildcard() {
        let results = execute_path_with_wildcards(&tx_json, &path.segments)?;
        Ok(QueryResult::Multiple(results))
    } else {
        let result = execute_path(&tx_json, &path.segments)?;
        Ok(QueryResult::Single(result))
    }
}

/// Convert a decoded transaction to a JSON value for querying.
fn transaction_to_json(tx: &DecodedTransaction) -> Result<JsonValue> {
    use cml_chain::PolicyId;
    use cml_chain::assets::AssetName;
    use cml_core::serialization::Serialize as CmlSerialize;

    let body = &tx.tx.body;
    let witness_set = &tx.tx.witness_set;

    // Build inputs
    let inputs: Vec<JsonValue> = body
        .inputs
        .iter()
        .map(|input| {
            serde_json::json!({
                "transaction_id": hex::encode(input.transaction_id.to_raw_bytes()),
                "index": input.index
            })
        })
        .collect();

    // Build outputs
    let outputs: Vec<JsonValue> = body.outputs.iter().map(output_to_json).collect();

    // Build mint if present
    let mint = body.mint.as_ref().map(|m| {
        m.iter()
            .map(|(policy_id, assets): (&PolicyId, _)| {
                let assets_json: Vec<JsonValue> = assets
                    .iter()
                    .map(|(name, amount): (&AssetName, &i64)| {
                        serde_json::json!({
                            "name": hex::encode(name.to_raw_bytes()),
                            "amount": *amount
                        })
                    })
                    .collect();
                serde_json::json!({
                    "policy_id": hex::encode(policy_id.to_raw_bytes()),
                    "assets": assets_json
                })
            })
            .collect::<Vec<_>>()
    });

    // Build collateral inputs if present
    let collateral_inputs = body.collateral_inputs.as_ref().map(|inputs| {
        inputs
            .iter()
            .map(|input| {
                serde_json::json!({
                    "transaction_id": hex::encode(input.transaction_id.to_raw_bytes()),
                    "index": input.index
                })
            })
            .collect::<Vec<_>>()
    });

    // Build required signers if present
    let required_signers = body.required_signers.as_ref().map(|signers| {
        signers
            .iter()
            .map(|s| hex::encode(s.to_raw_bytes()))
            .collect::<Vec<_>>()
    });

    // Build body JSON
    let mut body_json = serde_json::json!({
        "inputs": inputs,
        "outputs": outputs,
        "fee": body.fee
    });

    if let Some(ttl) = body.ttl {
        body_json["ttl"] = serde_json::json!(ttl);
    }
    if let Some(validity_start) = body.validity_interval_start {
        body_json["validity_interval_start"] = serde_json::json!(validity_start);
    }
    if let Some(m) = mint {
        body_json["mint"] = serde_json::json!(m);
    }
    if let Some(c) = collateral_inputs {
        body_json["collateral_inputs"] = serde_json::json!(c);
    }
    if let Some(r) = required_signers {
        body_json["required_signers"] = serde_json::json!(r);
    }
    if let Some(ref network_id) = body.network_id {
        // NetworkId stores network value directly
        body_json["network_id"] = serde_json::json!(network_id.network);
    }

    // Build certificates if present
    if let Some(ref certs) = body.certs {
        let certs_json: Vec<JsonValue> = certs.iter().map(certificate_to_json).collect();
        if !certs_json.is_empty() {
            body_json["certs"] = serde_json::json!(certs_json);
        }
    }

    // Build withdrawals if present
    if let Some(ref withdrawals) = body.withdrawals {
        let withdrawals_json: Vec<JsonValue> = withdrawals
            .iter()
            .map(|(reward_addr, coin)| {
                // RewardAddress needs to be converted to Address first for bech32
                // Clone since to_address() takes ownership
                let addr = reward_addr.clone().to_address();
                let addr_str = addr
                    .to_bech32(None)
                    .unwrap_or_else(|_| hex::encode(addr.to_raw_bytes()));
                serde_json::json!({
                    "reward_address": addr_str,
                    "amount": coin
                })
            })
            .collect();
        if !withdrawals_json.is_empty() {
            body_json["withdrawals"] = serde_json::json!(withdrawals_json);
        }
    }

    // Add script_data_hash if present
    if let Some(ref script_data_hash) = body.script_data_hash {
        body_json["script_data_hash"] =
            serde_json::json!(hex::encode(script_data_hash.to_raw_bytes()));
    }

    // Add collateral_return if present
    if let Some(ref collateral_return) = body.collateral_return {
        body_json["collateral_return"] = output_to_json(collateral_return);
    }

    // Add total_collateral if present
    if let Some(total_collateral) = body.total_collateral {
        body_json["total_collateral"] = serde_json::json!(total_collateral);
    }

    // Build witness set summary
    let mut witness_json = serde_json::json!({});

    if let Some(vkeys) = &witness_set.vkeywitnesses {
        witness_json["vkeywitnesses"] = serde_json::json!(vkeys.len());
    }
    if let Some(native) = &witness_set.native_scripts {
        witness_json["native_scripts"] = serde_json::json!(native.len());
    }
    if let Some(v1) = &witness_set.plutus_v1_scripts {
        let scripts: Vec<JsonValue> = v1
            .iter()
            .map(|s| {
                let bytes = s.to_cbor_bytes();
                serde_json::json!({
                    "hash": hex::encode(s.hash().to_raw_bytes()),
                    "size": bytes.len()
                })
            })
            .collect();
        witness_json["plutus_v1_scripts"] = serde_json::json!(scripts);
    }
    if let Some(v2) = &witness_set.plutus_v2_scripts {
        let scripts: Vec<JsonValue> = v2
            .iter()
            .map(|s| {
                let bytes = s.to_cbor_bytes();
                serde_json::json!({
                    "hash": hex::encode(s.hash().to_raw_bytes()),
                    "size": bytes.len()
                })
            })
            .collect();
        witness_json["plutus_v2_scripts"] = serde_json::json!(scripts);
    }
    if let Some(v3) = &witness_set.plutus_v3_scripts {
        let scripts: Vec<JsonValue> = v3
            .iter()
            .map(|s| {
                let bytes = s.to_cbor_bytes();
                serde_json::json!({
                    "hash": hex::encode(s.hash().to_raw_bytes()),
                    "size": bytes.len()
                })
            })
            .collect();
        witness_json["plutus_v3_scripts"] = serde_json::json!(scripts);
    }
    if let Some(data) = &witness_set.plutus_datums {
        witness_json["plutus_data"] = serde_json::json!(data.len());
    }
    if witness_set.redeemers.is_some() {
        // Redeemers present (can't easily get count without iteration)
        witness_json["redeemers"] = serde_json::json!("present");
    }

    // Build auxiliary data if present
    let auxiliary_data = tx.tx.auxiliary_data.as_ref().map(|aux| {
        let mut aux_json = serde_json::json!({});

        if let Some(metadata) = aux.metadata() {
            let labels: Vec<JsonValue> = metadata
                .entries
                .iter()
                .map(|(label, value)| {
                    serde_json::json!({
                        "label": label,
                        "value": metadata_value_to_json(value)
                    })
                })
                .collect();
            aux_json["metadata"] = serde_json::json!({ "labels": labels });
        }

        if let Some(native) = aux.native_scripts() {
            aux_json["native_scripts"] = serde_json::json!(native.len());
        }

        if let Some(v1) = aux.plutus_v1_scripts() {
            aux_json["plutus_v1_scripts"] = serde_json::json!(v1.len());
        }

        if let Some(v2) = aux.plutus_v2_scripts() {
            aux_json["plutus_v2_scripts"] = serde_json::json!(v2.len());
        }

        // Note: plutus_v3_scripts not available in AuxiliaryData accessor methods

        aux_json
    });

    // Build final transaction JSON
    let mut tx_json = serde_json::json!({
        "hash": hex::encode(tx.hash.to_raw_bytes()),
        "body": body_json,
        "witness_set": witness_json,
        "is_valid": tx.tx.is_valid
    });

    if let Some(aux) = auxiliary_data {
        tx_json["auxiliary_data"] = aux;
    }

    Ok(tx_json)
}

/// Convert a transaction output to JSON.
fn output_to_json(output: &cml_chain::transaction::TransactionOutput) -> JsonValue {
    use cml_chain::transaction::TransactionOutput;
    use cml_core::serialization::Serialize as CmlSerialize;

    match output {
        TransactionOutput::AlonzoFormatTxOut(alonzo) => {
            let mut json = serde_json::json!({
                "address": format_address(&alonzo.address),
                "value": value_to_json(&alonzo.amount)
            });

            if let Some(datum_hash) = &alonzo.datum_hash {
                json["datum"] = serde_json::json!({
                    "type": "hash",
                    "hash": hex::encode(datum_hash.to_raw_bytes())
                });
            }

            json
        }
        TransactionOutput::ConwayFormatTxOut(conway) => {
            let mut json = serde_json::json!({
                "address": format_address(&conway.address),
                "value": value_to_json(&conway.amount)
            });

            if let Some(datum_option) = &conway.datum_option {
                use cml_chain::transaction::DatumOption;
                match datum_option {
                    DatumOption::Hash { datum_hash, .. } => {
                        json["datum"] = serde_json::json!({
                            "type": "hash",
                            "hash": hex::encode(datum_hash.to_raw_bytes())
                        });
                    }
                    DatumOption::Datum { datum, .. } => {
                        let bytes = datum.to_cbor_bytes();
                        json["datum"] = serde_json::json!({
                            "type": "inline",
                            "bytes": hex::encode(&bytes),
                            "size": bytes.len()
                        });
                    }
                }
            }

            if let Some(script_ref) = &conway.script_reference {
                let bytes = script_ref.to_cbor_bytes();
                json["script_ref"] = serde_json::json!({
                    "size": bytes.len(),
                    "bytes": hex::encode(&bytes)
                });
            }

            json
        }
    }
}

/// Format an address to bech32.
fn format_address(addr: &cml_chain::address::Address) -> String {
    // Try to get bech32 representation
    addr.to_bech32(None).unwrap_or_else(|_| {
        // Fallback to hex if bech32 fails
        hex::encode(addr.to_raw_bytes())
    })
}

/// Convert a value (coin + multi-assets) to JSON.
fn value_to_json(value: &cml_chain::assets::Value) -> JsonValue {
    use cml_chain::PolicyId;
    use cml_chain::assets::AssetName;

    let coin = value.coin;

    let multi_assets: Vec<JsonValue> = value
        .multiasset
        .iter()
        .map(|(policy_id, assets): (&PolicyId, _)| {
            let assets_json: Vec<JsonValue> = assets
                .iter()
                .map(|(name, amount): (&AssetName, &u64)| {
                    serde_json::json!({
                        "name": hex::encode(name.to_raw_bytes()),
                        "amount": *amount
                    })
                })
                .collect();
            serde_json::json!({
                "policy_id": hex::encode(policy_id.to_raw_bytes()),
                "assets": assets_json
            })
        })
        .collect();

    if multi_assets.is_empty() {
        serde_json::json!({ "coin": coin })
    } else {
        serde_json::json!({
            "coin": coin,
            "multi_assets": multi_assets
        })
    }
}

/// Convert metadata value to JSON.
fn metadata_value_to_json(value: &cml_chain::auxdata::TransactionMetadatum) -> JsonValue {
    use cml_chain::auxdata::TransactionMetadatum;

    match value {
        TransactionMetadatum::Int(i) => {
            // CML Int can be positive or negative
            serde_json::json!(i.to_string())
        }
        TransactionMetadatum::Bytes { bytes, .. } => {
            serde_json::json!({
                "bytes": hex::encode(bytes)
            })
        }
        TransactionMetadatum::Text { text, .. } => {
            serde_json::json!(text)
        }
        TransactionMetadatum::List { elements, .. } => {
            let arr: Vec<JsonValue> = elements.iter().map(metadata_value_to_json).collect();
            serde_json::json!(arr)
        }
        TransactionMetadatum::Map(map_entries) => {
            let map: Vec<JsonValue> = map_entries
                .entries
                .iter()
                .map(|(k, v)| {
                    serde_json::json!({
                        "key": metadata_value_to_json(k),
                        "value": metadata_value_to_json(v)
                    })
                })
                .collect();
            serde_json::json!(map)
        }
    }
}

/// Convert a certificate to JSON.
fn certificate_to_json(cert: &cml_chain::certs::Certificate) -> JsonValue {
    use cml_chain::certs::Certificate;

    match cert {
        Certificate::StakeRegistration(reg) => {
            serde_json::json!({
                "type": "stake_registration",
                "stake_credential": stake_credential_to_json(&reg.stake_credential)
            })
        }
        Certificate::StakeDeregistration(dereg) => {
            serde_json::json!({
                "type": "stake_deregistration",
                "stake_credential": stake_credential_to_json(&dereg.stake_credential)
            })
        }
        Certificate::StakeDelegation(deleg) => {
            serde_json::json!({
                "type": "stake_delegation",
                "stake_credential": stake_credential_to_json(&deleg.stake_credential),
                "pool_keyhash": hex::encode(deleg.pool.to_raw_bytes())
            })
        }
        Certificate::PoolRegistration(pool_reg) => {
            serde_json::json!({
                "type": "pool_registration",
                "pool_keyhash": hex::encode(pool_reg.pool_params.operator.to_raw_bytes()),
                "vrf_keyhash": hex::encode(pool_reg.pool_params.vrf_keyhash.to_raw_bytes()),
                "pledge": pool_reg.pool_params.pledge,
                "cost": pool_reg.pool_params.cost,
                "margin": format!("{}/{}", pool_reg.pool_params.margin.start, pool_reg.pool_params.margin.end)
            })
        }
        Certificate::PoolRetirement(pool_ret) => {
            serde_json::json!({
                "type": "pool_retirement",
                "pool_keyhash": hex::encode(pool_ret.pool.to_raw_bytes()),
                "epoch": pool_ret.epoch
            })
        }
        Certificate::RegCert(reg) => {
            serde_json::json!({
                "type": "reg_cert",
                "stake_credential": stake_credential_to_json(&reg.stake_credential),
                "deposit": reg.deposit
            })
        }
        Certificate::UnregCert(unreg) => {
            serde_json::json!({
                "type": "unreg_cert",
                "stake_credential": stake_credential_to_json(&unreg.stake_credential),
                "deposit": unreg.deposit
            })
        }
        Certificate::VoteDelegCert(vote_deleg) => {
            serde_json::json!({
                "type": "vote_deleg_cert",
                "stake_credential": stake_credential_to_json(&vote_deleg.stake_credential),
                "drep": drep_to_json(&vote_deleg.d_rep)
            })
        }
        Certificate::StakeVoteDelegCert(stake_vote) => {
            serde_json::json!({
                "type": "stake_vote_deleg_cert",
                "stake_credential": stake_credential_to_json(&stake_vote.stake_credential),
                "pool_keyhash": hex::encode(stake_vote.pool.to_raw_bytes()),
                "drep": drep_to_json(&stake_vote.d_rep)
            })
        }
        Certificate::StakeRegDelegCert(stake_reg) => {
            serde_json::json!({
                "type": "stake_reg_deleg_cert",
                "stake_credential": stake_credential_to_json(&stake_reg.stake_credential),
                "pool_keyhash": hex::encode(stake_reg.pool.to_raw_bytes()),
                "deposit": stake_reg.deposit
            })
        }
        Certificate::VoteRegDelegCert(vote_reg) => {
            serde_json::json!({
                "type": "vote_reg_deleg_cert",
                "stake_credential": stake_credential_to_json(&vote_reg.stake_credential),
                "drep": drep_to_json(&vote_reg.d_rep),
                "deposit": vote_reg.deposit
            })
        }
        Certificate::StakeVoteRegDelegCert(stake_vote_reg) => {
            serde_json::json!({
                "type": "stake_vote_reg_deleg_cert",
                "stake_credential": stake_credential_to_json(&stake_vote_reg.stake_credential),
                "pool_keyhash": hex::encode(stake_vote_reg.pool.to_raw_bytes()),
                "drep": drep_to_json(&stake_vote_reg.d_rep),
                "deposit": stake_vote_reg.deposit
            })
        }
        Certificate::AuthCommitteeHotCert(auth) => {
            serde_json::json!({
                "type": "auth_committee_hot_cert",
                "committee_cold_credential": credential_to_json(&auth.committee_cold_credential),
                "committee_hot_credential": credential_to_json(&auth.committee_hot_credential)
            })
        }
        Certificate::ResignCommitteeColdCert(resign) => {
            serde_json::json!({
                "type": "resign_committee_cold_cert",
                "committee_cold_credential": credential_to_json(&resign.committee_cold_credential)
            })
        }
        Certificate::RegDrepCert(reg_drep) => {
            serde_json::json!({
                "type": "reg_drep_cert",
                "drep_credential": credential_to_json(&reg_drep.drep_credential),
                "deposit": reg_drep.deposit
            })
        }
        Certificate::UnregDrepCert(unreg_drep) => {
            serde_json::json!({
                "type": "unreg_drep_cert",
                "drep_credential": credential_to_json(&unreg_drep.drep_credential),
                "deposit": unreg_drep.deposit
            })
        }
        Certificate::UpdateDrepCert(update_drep) => {
            serde_json::json!({
                "type": "update_drep_cert",
                "drep_credential": credential_to_json(&update_drep.drep_credential)
            })
        }
    }
}

/// Convert stake credential to JSON.
fn stake_credential_to_json(cred: &cml_chain::certs::StakeCredential) -> JsonValue {
    credential_to_json(cred)
}

/// Convert credential to JSON.
fn credential_to_json(cred: &cml_chain::certs::Credential) -> JsonValue {
    use cml_chain::certs::Credential;
    match cred {
        Credential::PubKey { hash, .. } => {
            serde_json::json!({
                "type": "pubkey",
                "hash": hex::encode(hash.to_raw_bytes())
            })
        }
        Credential::Script { hash, .. } => {
            serde_json::json!({
                "type": "script",
                "hash": hex::encode(hash.to_raw_bytes())
            })
        }
    }
}

/// Convert DRep to JSON.
fn drep_to_json(drep: &cml_chain::certs::DRep) -> JsonValue {
    use cml_chain::certs::DRep;
    match drep {
        DRep::Key { pool, .. } => {
            serde_json::json!({
                "type": "key",
                "hash": hex::encode(pool.to_raw_bytes())
            })
        }
        DRep::Script { script_hash, .. } => {
            serde_json::json!({
                "type": "script",
                "hash": hex::encode(script_hash.to_raw_bytes())
            })
        }
        DRep::AlwaysAbstain { .. } => {
            serde_json::json!({ "type": "always_abstain" })
        }
        DRep::AlwaysNoConfidence { .. } => {
            serde_json::json!({ "type": "always_no_confidence" })
        }
    }
}

/// Execute a path query without wildcards.
fn execute_path(value: &JsonValue, segments: &[PathSegment]) -> Result<QueryValue> {
    let mut current = value.clone();

    for segment in segments {
        current = match segment {
            PathSegment::Field(name) => current
                .get(name)
                .cloned()
                .ok_or_else(|| Error::FieldNotFound(name.clone()))?,
            PathSegment::Index(idx) => current
                .get(*idx)
                .cloned()
                .ok_or(Error::IndexOutOfBounds(*idx))?,
            PathSegment::Wildcard => {
                return Err(Error::InvalidQuery(
                    "Unexpected wildcard in non-wildcard path".to_string(),
                ));
            }
        };
    }

    Ok(QueryValue::from(current))
}

/// Execute a path query with wildcards, returning all matching values.
fn execute_path_with_wildcards(
    value: &JsonValue,
    segments: &[PathSegment],
) -> Result<Vec<QueryValue>> {
    execute_path_recursive(value, segments)
}

/// Recursively execute path with wildcard expansion.
fn execute_path_recursive(value: &JsonValue, segments: &[PathSegment]) -> Result<Vec<QueryValue>> {
    if segments.is_empty() {
        return Ok(vec![QueryValue::from(value.clone())]);
    }

    let (current_segment, rest) = segments.split_first().unwrap();

    match current_segment {
        PathSegment::Field(name) => {
            let next = value
                .get(name)
                .ok_or_else(|| Error::FieldNotFound(name.clone()))?;
            execute_path_recursive(next, rest)
        }
        PathSegment::Index(idx) => {
            let next = value.get(*idx).ok_or(Error::IndexOutOfBounds(*idx))?;
            execute_path_recursive(next, rest)
        }
        PathSegment::Wildcard => {
            let arr = value
                .as_array()
                .ok_or_else(|| Error::InvalidQuery("Wildcard on non-array".to_string()))?;

            let mut results = Vec::new();
            for item in arr {
                let sub_results = execute_path_recursive(item, rest)?;
                results.extend(sub_results);
            }
            Ok(results)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_path_simple() {
        let json = serde_json::json!({
            "body": {
                "fee": 200000,
                "inputs": []
            }
        });

        let segments = vec![
            PathSegment::Field("body".into()),
            PathSegment::Field("fee".into()),
        ];

        let result = execute_path(&json, &segments).unwrap();
        match result {
            QueryValue::Number(n) => assert_eq!(n.as_u64(), Some(200000)),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_execute_path_with_index() {
        let json = serde_json::json!({
            "outputs": [
                { "address": "addr1..." },
                { "address": "addr2..." }
            ]
        });

        let segments = vec![
            PathSegment::Field("outputs".into()),
            PathSegment::Index(0),
            PathSegment::Field("address".into()),
        ];

        let result = execute_path(&json, &segments).unwrap();
        match result {
            QueryValue::String(s) => assert_eq!(s, "addr1..."),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_execute_path_with_wildcard() {
        let json = serde_json::json!({
            "outputs": [
                { "address": "addr1" },
                { "address": "addr2" },
                { "address": "addr3" }
            ]
        });

        let segments = vec![
            PathSegment::Field("outputs".into()),
            PathSegment::Wildcard,
            PathSegment::Field("address".into()),
        ];

        let results = execute_path_with_wildcards(&json, &segments).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_field_not_found() {
        let json = serde_json::json!({ "body": {} });
        let segments = vec![
            PathSegment::Field("body".into()),
            PathSegment::Field("nonexistent".into()),
        ];

        let result = execute_path(&json, &segments);
        assert!(matches!(result, Err(Error::FieldNotFound(_))));
    }

    #[test]
    fn test_index_out_of_bounds() {
        let json = serde_json::json!({ "arr": [1, 2] });
        let segments = vec![PathSegment::Field("arr".into()), PathSegment::Index(10)];

        let result = execute_path(&json, &segments);
        assert!(matches!(result, Err(Error::IndexOutOfBounds(10))));
    }
}
