//! Pretty terminal output with colors and tables.

use crate::cli::Args;
use crate::error::{Error, Result};
use crate::query::{QueryResult, QueryValue};
use colored::Colorize;
use comfy_table::{Cell, ContentArrangement, Table, presets};
use serde_json::Value as JsonValue;

/// Format a query result as pretty terminal output.
pub fn format_pretty(result: &QueryResult, args: &Args) -> Result<String> {
    if args.no_color {
        colored::control::set_override(false);
    }

    match result {
        QueryResult::FullTransaction(json) => format_full_transaction(json, args),
        QueryResult::Single(value) => format_single_value(value, args),
        QueryResult::Multiple(values) => format_multiple_values(values, args),
    }
}

/// Format a full transaction.
fn format_full_transaction(json: &JsonValue, args: &Args) -> Result<String> {
    let mut output = String::new();

    // Header with hash
    let hash = json
        .get("hash")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let is_valid = json
        .get("is_valid")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    output.push_str(&format!("{}\n", "Transaction".bold().cyan()));
    output.push_str(&format!("  {} {}\n", "Hash:".dimmed(), hash.yellow()));
    output.push_str(&format!(
        "  {} {}\n\n",
        "Valid:".dimmed(),
        if is_valid {
            "true".green()
        } else {
            "false".red()
        }
    ));

    // Body section
    if let Some(body) = json.get("body") {
        output.push_str(&format!("{}\n", "Body".bold().cyan()));

        // Fee
        if let Some(fee) = body.get("fee").and_then(|v| v.as_u64()) {
            output.push_str(&format!(
                "  {} {}\n",
                "Fee:".dimmed(),
                format_lovelace(fee, args)
            ));
        }

        // TTL
        if let Some(ttl) = body.get("ttl").and_then(|v| v.as_u64()) {
            output.push_str(&format!("  {} {}\n", "TTL:".dimmed(), ttl));
        }

        // Validity interval start
        if let Some(start) = body.get("validity_interval_start").and_then(|v| v.as_u64()) {
            output.push_str(&format!("  {} {}\n", "Valid from:".dimmed(), start));
        }

        output.push('\n');

        // Inputs table
        if let Some(inputs) = body.get("inputs").and_then(|v| v.as_array()) {
            output.push_str(&format!("{} ({})\n", "Inputs".bold().cyan(), inputs.len()));
            output.push_str(&format_inputs_table(inputs)?);
            output.push('\n');
        }

        // Outputs table
        if let Some(outputs) = body.get("outputs").and_then(|v| v.as_array()) {
            output.push_str(&format!(
                "{} ({})\n",
                "Outputs".bold().cyan(),
                outputs.len()
            ));
            output.push_str(&format_outputs_table(outputs, args)?);
            output.push('\n');
        }

        // Mint
        if let Some(mint) = body.get("mint").and_then(|v| v.as_array()) {
            if !mint.is_empty() {
                output.push_str(&format!("{}\n", "Mint".bold().cyan()));
                output.push_str(&format_mint(mint)?);
                output.push('\n');
            }
        }

        // Collateral
        if let Some(collateral) = body.get("collateral_inputs").and_then(|v| v.as_array()) {
            if !collateral.is_empty() {
                output.push_str(&format!(
                    "{} ({})\n",
                    "Collateral".bold().cyan(),
                    collateral.len()
                ));
                output.push_str(&format_inputs_table(collateral)?);
                output.push('\n');
            }
        }

        // Total collateral
        if let Some(total) = body.get("total_collateral").and_then(|v| v.as_u64()) {
            output.push_str(&format!(
                "  {} {}\n",
                "Total collateral:".dimmed(),
                format_lovelace(total, args)
            ));
        }

        // Collateral return
        if body.get("collateral_return").is_some() {
            output.push_str(&format!(
                "  {} {}\n",
                "Collateral return:".dimmed(),
                "present".green()
            ));
        }

        // Script data hash
        if let Some(hash) = body.get("script_data_hash").and_then(|v| v.as_str()) {
            output.push_str(&format!(
                "  {} {}\n",
                "Script data hash:".dimmed(),
                truncate_hash(hash, 16)
            ));
        }

        // Required signers
        if let Some(signers) = body.get("required_signers").and_then(|v| v.as_array()) {
            if !signers.is_empty() {
                output.push_str(&format!("{}\n", "Required Signers".bold().cyan()));
                for signer in signers {
                    if let Some(s) = signer.as_str() {
                        output.push_str(&format!("  {}\n", truncate_hash(s, 16)));
                    }
                }
                output.push('\n');
            }
        }

        // Certificates
        if let Some(certs) = body.get("certs").and_then(|v| v.as_array()) {
            if !certs.is_empty() {
                output.push_str(&format!(
                    "{} ({})\n",
                    "Certificates".bold().cyan(),
                    certs.len()
                ));
                output.push_str(&format_certificates(certs)?);
                output.push('\n');
            }
        }

        // Withdrawals
        if let Some(withdrawals) = body.get("withdrawals").and_then(|v| v.as_array()) {
            if !withdrawals.is_empty() {
                output.push_str(&format!(
                    "{} ({})\n",
                    "Withdrawals".bold().cyan(),
                    withdrawals.len()
                ));
                output.push_str(&format_withdrawals(withdrawals, args)?);
                output.push('\n');
            }
        }
    }

    // Witness set
    if let Some(witnesses) = json.get("witness_set") {
        output.push_str(&format!("{}\n", "Witnesses".bold().cyan()));
        output.push_str(&format_witnesses(witnesses)?);
        output.push('\n');
    }

    // Auxiliary data
    if let Some(aux) = json.get("auxiliary_data") {
        output.push_str(&format!("{}\n", "Auxiliary Data".bold().cyan()));
        output.push_str(&format_auxiliary_data(aux)?);
    }

    Ok(output)
}

/// Format inputs as a table.
fn format_inputs_table(inputs: &[JsonValue]) -> Result<String> {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("#").fg(comfy_table::Color::DarkGrey),
        Cell::new("Transaction ID").fg(comfy_table::Color::DarkGrey),
        Cell::new("Index").fg(comfy_table::Color::DarkGrey),
    ]);

    for (idx, input) in inputs.iter().enumerate() {
        let tx_id = input
            .get("transaction_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let index = input.get("index").and_then(|v| v.as_u64()).unwrap_or(0);

        table.add_row(vec![
            Cell::new(idx),
            Cell::new(truncate_hash(tx_id, 16)),
            Cell::new(index),
        ]);
    }

    Ok(format!("{}\n", table))
}

/// Format outputs as a table.
fn format_outputs_table(outputs: &[JsonValue], args: &Args) -> Result<String> {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("#").fg(comfy_table::Color::DarkGrey),
        Cell::new("Address").fg(comfy_table::Color::DarkGrey),
        Cell::new("Value").fg(comfy_table::Color::DarkGrey),
        Cell::new("Datum").fg(comfy_table::Color::DarkGrey),
    ]);

    for (idx, output) in outputs.iter().enumerate() {
        let address = output
            .get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        let value = output.get("value");
        let coin = value
            .and_then(|v| v.get("coin"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let multi_assets = value
            .and_then(|v| v.get("multi_assets"))
            .and_then(|v| v.as_array());

        let value_str = if let Some(assets) = multi_assets {
            if assets.is_empty() {
                format_lovelace(coin, args)
            } else {
                format!(
                    "{} + {} asset(s)",
                    format_lovelace(coin, args),
                    assets.len()
                )
            }
        } else {
            format_lovelace(coin, args)
        };

        let datum_str = match output.get("datum") {
            Some(datum) => {
                let datum_type = datum.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                match datum_type {
                    "hash" => {
                        let hash = datum.get("hash").and_then(|v| v.as_str()).unwrap_or("?");
                        format!("hash: {}", truncate_hash(hash, 8))
                    }
                    "inline" => {
                        let size = datum.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                        format!("<inline: {} B>", size)
                    }
                    _ => datum_type.to_string(),
                }
            }
            None => "-".dimmed().to_string(),
        };

        table.add_row(vec![
            Cell::new(idx),
            Cell::new(truncate_address(address, 24)),
            Cell::new(value_str),
            Cell::new(datum_str),
        ]);
    }

    Ok(format!("{}\n", table))
}

/// Format mint information.
fn format_mint(mint: &[JsonValue]) -> Result<String> {
    let mut output = String::new();

    for entry in mint {
        let policy_id = entry
            .get("policy_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        output.push_str(&format!(
            "  {} {}\n",
            "Policy:".dimmed(),
            truncate_hash(policy_id, 16)
        ));

        if let Some(assets) = entry.get("assets").and_then(|v| v.as_array()) {
            for asset in assets {
                let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let amount = asset.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);

                let name_display = if name.is_empty() {
                    "(empty)".dimmed().to_string()
                } else {
                    // Try to decode as UTF-8
                    hex::decode(name)
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                        .unwrap_or_else(|| truncate_hash(name, 16))
                };

                let amount_color = if amount > 0 {
                    format!("+{}", amount).green()
                } else {
                    format!("{}", amount).red()
                };

                output.push_str(&format!("    {} {}\n", name_display, amount_color));
            }
        }
    }

    Ok(output)
}

/// Format certificates.
fn format_certificates(certs: &[JsonValue]) -> Result<String> {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("#").fg(comfy_table::Color::DarkGrey),
        Cell::new("Type").fg(comfy_table::Color::DarkGrey),
        Cell::new("Details").fg(comfy_table::Color::DarkGrey),
    ]);

    for (idx, cert) in certs.iter().enumerate() {
        let cert_type = cert
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let details = format_certificate_details(cert);

        table.add_row(vec![
            Cell::new(idx),
            Cell::new(format_cert_type(cert_type)),
            Cell::new(details),
        ]);
    }

    Ok(format!("{}\n", table))
}

/// Format certificate type for display (more readable).
fn format_cert_type(cert_type: &str) -> String {
    match cert_type {
        "stake_registration" => "Stake Registration".to_string(),
        "stake_deregistration" => "Stake Deregistration".to_string(),
        "stake_delegation" => "Stake Delegation".to_string(),
        "pool_registration" => "Pool Registration".to_string(),
        "pool_retirement" => "Pool Retirement".to_string(),
        "reg_cert" => "Registration (Conway)".to_string(),
        "unreg_cert" => "Deregistration (Conway)".to_string(),
        "vote_deleg_cert" => "Vote Delegation".to_string(),
        "stake_vote_deleg_cert" => "Stake+Vote Delegation".to_string(),
        "stake_reg_deleg_cert" => "Stake Reg+Delegation".to_string(),
        "vote_reg_deleg_cert" => "Vote Reg+Delegation".to_string(),
        "stake_vote_reg_deleg_cert" => "Stake+Vote Reg+Del".to_string(),
        "auth_committee_hot_cert" => "Auth Committee Hot".to_string(),
        "resign_committee_cold_cert" => "Resign Committee Cold".to_string(),
        "reg_drep_cert" => "Register DRep".to_string(),
        "unreg_drep_cert" => "Unregister DRep".to_string(),
        "update_drep_cert" => "Update DRep".to_string(),
        _ => cert_type.to_string(),
    }
}

/// Format certificate details based on type.
fn format_certificate_details(cert: &JsonValue) -> String {
    let cert_type = cert.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match cert_type {
        "stake_delegation" => {
            let pool = cert
                .get("pool_keyhash")
                .and_then(|v| v.as_str())
                .map(|h| truncate_hash(h, 12))
                .unwrap_or_else(|| "?".to_string());
            format!("pool: {}", pool)
        }
        "pool_registration" => {
            let pool = cert
                .get("pool_keyhash")
                .and_then(|v| v.as_str())
                .map(|h| truncate_hash(h, 12))
                .unwrap_or_else(|| "?".to_string());
            let margin = cert.get("margin").and_then(|v| v.as_str()).unwrap_or("?");
            format!("{}, margin: {}", pool, margin)
        }
        "pool_retirement" => {
            let epoch = cert.get("epoch").and_then(|v| v.as_u64()).unwrap_or(0);
            format!("epoch: {}", epoch)
        }
        "vote_deleg_cert" | "stake_vote_deleg_cert" => {
            if let Some(drep) = cert.get("drep") {
                format_drep_details(drep)
            } else {
                "-".to_string()
            }
        }
        "reg_cert"
        | "unreg_cert"
        | "stake_reg_deleg_cert"
        | "vote_reg_deleg_cert"
        | "stake_vote_reg_deleg_cert"
        | "reg_drep_cert"
        | "unreg_drep_cert" => {
            if let Some(deposit) = cert.get("deposit").and_then(|v| v.as_u64()) {
                format!(
                    "deposit: {} lovelace",
                    format_number_with_separators(deposit)
                )
            } else {
                "-".to_string()
            }
        }
        _ => {
            // For other types, show stake credential hash if present
            if let Some(cred) = cert.get("stake_credential") {
                if let Some(hash) = cred.get("hash").and_then(|v| v.as_str()) {
                    return truncate_hash(hash, 16);
                }
            }
            "-".to_string()
        }
    }
}

/// Format DRep details for display.
fn format_drep_details(drep: &JsonValue) -> String {
    let drep_type = drep.get("type").and_then(|v| v.as_str()).unwrap_or("?");
    match drep_type {
        "key" | "script" => {
            let hash = drep
                .get("hash")
                .and_then(|v| v.as_str())
                .map(|h| truncate_hash(h, 12))
                .unwrap_or_else(|| "?".to_string());
            format!("drep: {} ({})", hash, drep_type)
        }
        "always_abstain" => "drep: always_abstain".to_string(),
        "always_no_confidence" => "drep: always_no_confidence".to_string(),
        _ => format!("drep: {}", drep_type),
    }
}

/// Format withdrawals.
fn format_withdrawals(withdrawals: &[JsonValue], args: &Args) -> Result<String> {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("#").fg(comfy_table::Color::DarkGrey),
        Cell::new("Reward Address").fg(comfy_table::Color::DarkGrey),
        Cell::new("Amount").fg(comfy_table::Color::DarkGrey),
    ]);

    for (idx, withdrawal) in withdrawals.iter().enumerate() {
        let reward_addr = withdrawal
            .get("reward_address")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let amount = withdrawal
            .get("amount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        table.add_row(vec![
            Cell::new(idx),
            Cell::new(truncate_address(reward_addr, 32)),
            Cell::new(format_lovelace(amount, args)),
        ]);
    }

    Ok(format!("{}\n", table))
}

/// Format witness set summary.
fn format_witnesses(witnesses: &JsonValue) -> Result<String> {
    let mut output = String::new();

    if let Some(count) = witnesses.get("vkeywitnesses").and_then(|v| v.as_u64()) {
        output.push_str(&format!("  {} {}\n", "VKey signatures:".dimmed(), count));
    }

    if let Some(count) = witnesses.get("native_scripts").and_then(|v| v.as_u64()) {
        output.push_str(&format!("  {} {}\n", "Native scripts:".dimmed(), count));
    }

    for (version, label) in [
        ("plutus_v1_scripts", "Plutus V1"),
        ("plutus_v2_scripts", "Plutus V2"),
        ("plutus_v3_scripts", "Plutus V3"),
    ] {
        if let Some(scripts) = witnesses.get(version).and_then(|v| v.as_array()) {
            output.push_str(&format!(
                "  {} {}:\n",
                format!("{} scripts:", label).dimmed(),
                scripts.len()
            ));
            for script in scripts {
                let hash = script.get("hash").and_then(|v| v.as_str()).unwrap_or("?");
                let size = script.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                output.push_str(&format!("    {} <{} B>\n", truncate_hash(hash, 12), size));
            }
        }
    }

    if let Some(count) = witnesses.get("plutus_data").and_then(|v| v.as_u64()) {
        output.push_str(&format!("  {} {}\n", "Plutus data:".dimmed(), count));
    }

    if let Some(count) = witnesses.get("redeemers").and_then(|v| v.as_u64()) {
        output.push_str(&format!("  {} {}\n", "Redeemers:".dimmed(), count));
    }

    if output.is_empty() {
        output.push_str(&format!("  {}\n", "(empty)".dimmed()));
    }

    Ok(output)
}

/// Format auxiliary data.
fn format_auxiliary_data(aux: &JsonValue) -> Result<String> {
    let mut output = String::new();

    if let Some(metadata) = aux.get("metadata") {
        if let Some(labels) = metadata.get("labels").and_then(|v| v.as_array()) {
            output.push_str(&format!(
                "  {} {} label(s)\n",
                "Metadata:".dimmed(),
                labels.len()
            ));
            for label_entry in labels.iter().take(5) {
                let label = label_entry
                    .get("label")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                output.push_str(&format!("    Label {}\n", label.to_string().yellow()));
            }
            if labels.len() > 5 {
                output.push_str(&format!(
                    "    {} more...\n",
                    format!("... and {}", labels.len() - 5).dimmed()
                ));
            }
        }
    }

    if let Some(count) = aux.get("native_scripts").and_then(|v| v.as_u64()) {
        output.push_str(&format!("  {} {}\n", "Native scripts:".dimmed(), count));
    }

    for (version, label) in [
        ("plutus_v1_scripts", "Plutus V1 scripts"),
        ("plutus_v2_scripts", "Plutus V2 scripts"),
        ("plutus_v3_scripts", "Plutus V3 scripts"),
    ] {
        if let Some(count) = aux.get(version).and_then(|v| v.as_u64()) {
            output.push_str(&format!("  {} {}\n", format!("{}:", label).dimmed(), count));
        }
    }

    if output.is_empty() {
        output.push_str(&format!("  {}\n", "(empty)".dimmed()));
    }

    Ok(output)
}

/// Format a single query value.
fn format_single_value(value: &QueryValue, args: &Args) -> Result<String> {
    match value {
        QueryValue::Null => Ok("null".dimmed().to_string()),
        QueryValue::Bool(b) => Ok(if *b {
            "true".green().to_string()
        } else {
            "false".red().to_string()
        }),
        QueryValue::Number(n) => {
            // Format number, converting to ADA if requested
            if let Some(num) = n.as_u64() {
                if args.ada {
                    Ok(format_lovelace(num, args))
                } else {
                    Ok(format_number_with_separators(num))
                }
            } else {
                Ok(n.to_string())
            }
        }
        QueryValue::String(s) => {
            // Check if it looks like an address
            if s.starts_with("addr") {
                Ok(s.clone())
            } else if s.chars().all(|c| c.is_ascii_hexdigit()) && s.len() >= 16 {
                // Looks like a hash - show truncated
                Ok(truncate_hash(s, 24))
            } else {
                Ok(s.clone())
            }
        }
        QueryValue::Array(arr) => {
            let items: Result<Vec<String>> =
                arr.iter().map(|v| format_single_value(v, args)).collect();
            Ok(format!("[{}]", items?.join(", ")))
        }
        QueryValue::Object(_) => {
            // For objects, fall back to JSON
            serde_json::to_string_pretty(value).map_err(|e| Error::FormatError(e.to_string()))
        }
    }
}

/// Format multiple query values (from wildcard).
fn format_multiple_values(values: &[QueryValue], args: &Args) -> Result<String> {
    let formatted: Result<Vec<String>> = values
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let formatted = format_single_value(v, args)?;
            Ok(format!("[{}] {}", idx.to_string().dimmed(), formatted))
        })
        .collect();

    Ok(formatted?.join("\n"))
}

/// Format lovelace amount, optionally as ADA.
fn format_lovelace(lovelace: u64, args: &Args) -> String {
    if args.ada {
        let ada = lovelace as f64 / 1_000_000.0;
        format!("{:.6} ADA", ada)
    } else {
        format!("{} lovelace", format_number_with_separators(lovelace))
    }
}

/// Format a number with thousand separators.
fn format_number_with_separators(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Truncate a hash for display.
fn truncate_hash(hash: &str, max_len: usize) -> String {
    if hash.len() <= max_len {
        hash.to_string()
    } else {
        let half = (max_len - 3) / 2;
        format!("{}...{}", &hash[..half], &hash[hash.len() - half..])
    }
}

/// Truncate an address for display.
fn truncate_address(addr: &str, max_len: usize) -> String {
    if addr.len() <= max_len {
        addr.to_string()
    } else {
        // Keep the prefix (addr1, addr_test1) visible
        let prefix_end = addr.find('1').map(|i| i + 1).unwrap_or(5);
        let suffix_len = 8;
        let prefix_len = max_len - suffix_len - 3;

        if prefix_end < prefix_len {
            format!(
                "{}...{}",
                &addr[..prefix_len.max(prefix_end)],
                &addr[addr.len() - suffix_len..]
            )
        } else {
            format!(
                "{}...{}",
                &addr[..prefix_len],
                &addr[addr.len() - suffix_len..]
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_with_separators() {
        assert_eq!(format_number_with_separators(1000), "1,000");
        assert_eq!(format_number_with_separators(1000000), "1,000,000");
        assert_eq!(format_number_with_separators(123), "123");
    }

    #[test]
    fn test_truncate_hash() {
        let hash = "0123456789abcdef0123456789abcdef";
        assert_eq!(truncate_hash(hash, 16), "012345...abcdef");
    }

    #[test]
    fn test_truncate_address() {
        let addr = "addr1qxck47d8fy6vk2jqsf3r9k2l7vr5h9d8wkz3r9k2l7vr5h9d8wkz";
        let truncated = truncate_address(addr, 24);
        assert!(truncated.len() <= 27); // 24 + "..."
        assert!(truncated.starts_with("addr1"));
    }

    #[test]
    fn test_format_lovelace_as_ada() {
        let args = Args {
            command: None,
            first: None,
            second: None,
            json: false,
            raw: false,
            ada: true,
            check: false,
            no_color: true,
        };
        assert_eq!(format_lovelace(2_500_000, &args), "2.500000 ADA");
    }

    #[test]
    fn test_format_lovelace_as_lovelace() {
        let args = Args {
            command: None,
            first: None,
            second: None,
            json: false,
            raw: false,
            ada: false,
            check: false,
            no_color: true,
        };
        assert_eq!(format_lovelace(2_500_000, &args), "2,500,000 lovelace");
    }
}
