//! Update checker for cq.
//!
//! Queries crates.io to check if a newer version is available.

use crate::error::{Error, Result};

/// Current version of cq (from Cargo.toml).
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// crates.io API endpoint for cq crate info.
const CRATES_IO_API: &str = "https://crates.io/api/v1/crates/cq";

/// Check for updates and display the result.
pub fn check_for_updates() -> Result<()> {
    println!("cq v{}", CURRENT_VERSION);
    println!();

    // Fetch latest version from crates.io
    let latest = fetch_latest_version();

    match latest {
        Ok(version) => {
            if version == CURRENT_VERSION {
                println!("You are on the latest version.");
            } else if is_newer(&version, CURRENT_VERSION) {
                println!("Update available: {} -> {}", CURRENT_VERSION, version);
                println!();
                println!("Upgrade with:");
                println!("  cargo install cq --force");
                println!();
                println!("Or download from:");
                println!("  https://github.com/karkigrishmin/cq/releases/latest");
            } else {
                // Current version is newer (dev build?)
                println!("You are on the latest version.");
            }
        }
        Err(e) => {
            println!("Could not check for updates: {}", e);
        }
    }

    Ok(())
}

/// Fetch the latest version from crates.io.
fn fetch_latest_version() -> Result<String> {
    let response = ureq::get(CRATES_IO_API)
        .set("User-Agent", "cq-update-checker")
        .call()
        .map_err(|e| Error::NetworkError(format!("Failed to connect to crates.io: {}", e)))?;

    let body = response
        .into_string()
        .map_err(|e| Error::NetworkError(format!("Invalid response from crates.io: {}", e)))?;

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| Error::NetworkError(format!("Invalid JSON from crates.io: {}", e)))?;

    json["crate"]["max_version"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| Error::NetworkError("Could not parse version from crates.io".to_string()))
}

/// Check if `new_version` is newer than `current_version`.
/// Uses simple semver comparison.
fn is_newer(new_version: &str, current_version: &str) -> bool {
    let parse = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() >= 3 {
            Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            ))
        } else {
            None
        }
    };

    match (parse(new_version), parse(current_version)) {
        (Some((n_maj, n_min, n_pat)), Some((c_maj, c_min, c_pat))) => {
            (n_maj, n_min, n_pat) > (c_maj, c_min, c_pat)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.3.0", "0.2.0"));
        assert!(is_newer("0.2.1", "0.2.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
    }
}
