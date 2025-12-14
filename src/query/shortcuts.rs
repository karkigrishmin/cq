//! Query shortcut expansion.

/// Get the expansion for a shortcut prefix.
fn shortcut_expansion(shortcut: &str) -> Option<&'static str> {
    match shortcut {
        "fee" => Some("body.fee"),
        "inputs" => Some("body.inputs"),
        "outputs" => Some("body.outputs"),
        "metadata" => Some("auxiliary_data.metadata"),
        "witnesses" => Some("witness_set"),
        "hash" => Some("__hash__"),
        "ttl" => Some("body.ttl"),
        "mint" => Some("body.mint"),
        "certs" => Some("body.certs"),
        "withdrawals" => Some("body.withdrawals"),
        "collateral" => Some("body.collateral_inputs"),
        "required_signers" => Some("body.required_signers"),
        "network_id" => Some("body.network_id"),
        "validity_start" => Some("body.validity_interval_start"),
        "script_data_hash" => Some("body.script_data_hash"),
        "collateral_return" => Some("body.collateral_return"),
        "total_collateral" => Some("body.total_collateral"),
        _ => None,
    }
}

/// Expand a query shortcut to its full path.
///
/// Handles both exact matches and prefixes:
/// - `outputs` → `body.outputs`
/// - `outputs.0.address` → `body.outputs.0.address`
///
/// # Shortcuts
///
/// - `fee` → `body.fee`
/// - `inputs` → `body.inputs`
/// - `outputs` → `body.outputs`
/// - `metadata` → `auxiliary_data.metadata`
/// - `witnesses` → `witness_set`
/// - `hash` → `__hash__` (special computed field)
/// - `ttl` → `body.ttl`
/// - `mint` → `body.mint`
/// - `certs` → `body.certs`
/// - `withdrawals` → `body.withdrawals`
/// - `collateral` → `body.collateral_inputs`
pub fn expand_shortcut(query: &str) -> String {
    // Check for exact match first
    if let Some(expanded) = shortcut_expansion(query) {
        return expanded.to_string();
    }

    // Check if query starts with a shortcut followed by a dot
    if let Some(dot_pos) = query.find('.') {
        let prefix = &query[..dot_pos];
        let rest = &query[dot_pos..]; // includes the dot

        if let Some(expanded_prefix) = shortcut_expansion(prefix) {
            return format!("{}{}", expanded_prefix, rest);
        }
    }

    // No shortcut found, return as-is
    query.to_string()
}

/// Check if a query is the special hash computed field.
pub fn is_hash_query(expanded: &str) -> bool {
    expanded == "__hash__"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_shortcuts() {
        assert_eq!(expand_shortcut("fee"), "body.fee");
        assert_eq!(expand_shortcut("inputs"), "body.inputs");
        assert_eq!(expand_shortcut("outputs"), "body.outputs");
        assert_eq!(expand_shortcut("metadata"), "auxiliary_data.metadata");
        assert_eq!(expand_shortcut("witnesses"), "witness_set");
        assert_eq!(expand_shortcut("hash"), "__hash__");
    }

    #[test]
    fn test_expand_shortcuts_nested() {
        assert_eq!(
            expand_shortcut("outputs.0.address"),
            "body.outputs.0.address"
        );
        assert_eq!(
            expand_shortcut("inputs.0.transaction_id"),
            "body.inputs.0.transaction_id"
        );
        assert_eq!(expand_shortcut("outputs.*.value"), "body.outputs.*.value");
    }

    #[test]
    fn test_passthrough() {
        assert_eq!(expand_shortcut("body.fee"), "body.fee");
        assert_eq!(
            expand_shortcut("body.outputs.0.address"),
            "body.outputs.0.address"
        );
        assert_eq!(expand_shortcut("unknown"), "unknown");
        assert_eq!(expand_shortcut("unknown.field"), "unknown.field");
    }

    #[test]
    fn test_is_hash_query() {
        assert!(is_hash_query("__hash__"));
        assert!(!is_hash_query("hash"));
        assert!(!is_hash_query("body.fee"));
    }
}
